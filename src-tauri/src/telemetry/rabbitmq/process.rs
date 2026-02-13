use crate::telemetry::geos;
use crate::telemetry::geos::*;
use crate::telemetry::sql::*;
use crate::telemetry::types::{TelemetryData, VehicleTelemetryData};
use futures_util::stream::StreamExt;
use lapin::{options::*, Consumer, Result as LapinResult};
use serde_json::json;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

use super::heartbeat::{is_vehicle_connected, update_vehicle_heartbeat, VehicleHeartbeat};
use super::TelemetryEventTrigger;

// Process telemetry data from the consumer
pub async fn process_telemetry(
    mut consumer: Consumer,
    state: Arc<Mutex<VehicleTelemetryData>>,
    db: PgPool,
    app_handle: Option<AppHandle>,
    vehicle_heartbeats: Arc<Mutex<HashMap<String, VehicleHeartbeat>>>,
    heartbeat_timeout: Duration,
) -> LapinResult<()> {
    let mut failure_count = 0;

    while let Some(delivery) = consumer.next().await {
        if let Ok(delivery) = delivery {
            match serde_json::from_slice::<TelemetryData>(&delivery.data) {
                Ok(mut data) => {
                    failure_count = 0; // reset on success

                    // Update heartbeat for this vehicle
                    update_vehicle_heartbeat(
                        &data.vehicle_id,
                        vehicle_heartbeats.clone(),
                        state.clone(),
                    )
                    .await;

                    // Existing signal strength check
                    if data.signal_strength < -70 {
                        data.vehicle_status = "Bad Connection".to_string();
                    }

                    // Existing geo-fencing check
                    let point = geos::Coordinate {
                        latitude: data.current_position.latitude,
                        longitude: data.current_position.longitude,
                    };

                    if is_near_keep_out_zone(&data.vehicle_id, &point, 1000.0) {
                        data.vehicle_status = "Approaching restricted area".to_string();
                    }

                    // If vehicle was marked as disconnected but we're receiving data,
                    // and no other critical status is set, mark as connected
                    if data.vehicle_status.is_empty() || data.vehicle_status == "Disconnected" {
                        if is_vehicle_connected(
                            &data.vehicle_id,
                            vehicle_heartbeats.clone(),
                            heartbeat_timeout,
                        )
                        .await
                        {
                            data.vehicle_status = "Connected".to_string();
                        }
                    }

                    let vehicle_id = data.vehicle_id.clone();
                    state
                        .lock()
                        .await
                        .update_vehicle_telemetry_state(vehicle_id.clone(), data.clone());

                    // Create payload for the event
                    let payload = json!({
                        "vehicle_id": vehicle_id,
                        "telemetry": data.clone(),
                        "timestamp": std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs()
                    });

                    // Emit the telemetry update using TelemetryEventTrigger
                    if let Some(app_handle) = &app_handle {
                        let vehicle_telemetry: VehicleTelemetryData = state.lock().await.clone();
                        match TelemetryEventTrigger::new(app_handle.clone())
                            .on_updated(vehicle_telemetry)
                        {
                            Ok(_) => {
                                println!(
                                    "Successfully emitted telemetry update via event trigger for vehicle: {}",
                                    vehicle_id
                                );
                            }
                            Err(e) => {
                                println!(
                                    "Failed to emit telemetry update via event trigger: {}",
                                    e
                                );

                                // Fallback to regular app_handle emit
                                if let Err(e) = app_handle.emit("telemetry_update", &payload) {
                                    println!("Failed to emit telemetry update: {}", e);
                                }
                            }
                        }
                    } else {
                        println!("Warning: No app_handle available to emit telemetry updates");
                    }

                    println!("Received telemetry data from {}: {:?}", vehicle_id, payload);
                    println!("Vehicle {} status: {:?}", vehicle_id, data.vehicle_status);
                    delivery.ack(BasicAckOptions::default()).await?;

                    // Insert telemetry data into the database
                    let current_position_str = serde_json::to_string(&data.current_position).unwrap();
                    let request_coordinate_str =
                        serde_json::to_string(&data.request_coordinate).unwrap();

                    if let Err(e) = insert_telemetry(
                        db.clone(),
                        data.vehicle_id.clone(),
                        data.signal_strength,
                        data.pitch,
                        data.yaw,
                        data.roll,
                        data.speed,
                        data.altitude,
                        data.battery_life,
                        current_position_str,
                        data.vehicle_status.clone(),
                        request_coordinate_str,
                    )
                    .await
                    {
                        eprintln!("Failed to insert telemetry data: {}", e);
                    }
                }
                Err(e) => {
                    failure_count += 1;
                    println!(
                        "Failed to parse Telemetry data (attempt {}): {}",
                        failure_count, e
                    );
                    println!("Raw payload: {:?}", String::from_utf8_lossy(&delivery.data));
                    delivery.reject(BasicRejectOptions::default()).await?;

                    if failure_count >= 3 {
                        let error_payload = json!({
                            "error": "Failed to establish a connection after 3 invalid messages",
                            "consecutive_failures": failure_count
                        });

                        if let Some(app_handle) = &app_handle {
                            app_handle.emit("telemetry_error", error_payload).ok();
                        }

                        return Err(lapin::Error::InvalidChannelState(
                            lapin::ChannelState::Closed,
                        ));
                    }
                }
            }
        }
    }

    Ok(())
}
