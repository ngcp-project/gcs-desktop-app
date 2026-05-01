use crate::telemetry::types::VehicleTelemetryData;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use tokio::time::interval;

use super::TelemetryEventTrigger;

#[derive(Clone, Debug)]
pub struct VehicleHeartbeat {
    pub last_seen: Instant,
    pub is_connected: bool,
    pub consecutive_failures: u32,
}

impl VehicleHeartbeat {
    pub fn new() -> Self {
        Self {
            last_seen: Instant::now(),
            is_connected: true,
            consecutive_failures: 0,
        }
    }

    pub fn update(&mut self) {
        self.last_seen = Instant::now();
        self.is_connected = true;
        self.consecutive_failures = 0;
    }

    pub fn is_timeout(&self, timeout_duration: Duration) -> bool {
        self.last_seen.elapsed() > timeout_duration
    }

    pub fn mark_disconnected(&mut self) {
        self.is_connected = false;
        self.consecutive_failures += 1;
    }
}

// Start the heartbeat monitoring task
pub async fn start_heartbeat_monitor(
    heartbeats: Arc<Mutex<HashMap<String, VehicleHeartbeat>>>,
    state: Arc<Mutex<VehicleTelemetryData>>,
    app_handle: Option<AppHandle>,
    timeout: Duration,
    check_interval: Duration,
) {
    tokio::spawn(async move {
        let mut interval_timer = interval(check_interval);

        loop {
            interval_timer.tick().await;

            let mut heartbeats_guard = heartbeats.lock().await;
            let mut state_guard = state.lock().await;
            let mut status_changed = false;

            for (vehicle_id, heartbeat) in heartbeats_guard.iter_mut() {
                if heartbeat.is_timeout(timeout) && heartbeat.is_connected {
                    println!("Vehicle {} heartbeat timeout detected", vehicle_id);
                    heartbeat.mark_disconnected();

                    // Update vehicle status in telemetry data based on vehicle_id
                    match vehicle_id.as_str() {
                        "eru" => {
                            state_guard.ERU.vehicle_status = "Disconnected".to_string();
                            status_changed = true;
                        }
                        "mea" => {
                            state_guard.MEA.vehicle_status = "Disconnected".to_string();
                            status_changed = true;
                        }
                        "mra" => {
                            state_guard.MRA.vehicle_status = "Disconnected".to_string();
                            status_changed = true;
                        }
                        _ => {
                            println!("Unknown vehicle_id: {}", vehicle_id);
                        }
                    }

                    if status_changed {
                        println!(
                            "Vehicle {} marked as disconnected after {} seconds of no data",
                            vehicle_id,
                            timeout.as_secs()
                        );
                    }
                }
            }

            // If any status changed, emit update
            if status_changed {
                if let Some(app_handle) = &app_handle {
                    let vehicle_telemetry = state_guard.clone();
                    drop(state_guard); // Release the lock before emitting
                    drop(heartbeats_guard); // Release the lock before emitting

                    // Try to emit via TelemetryEventTrigger first
                    match TelemetryEventTrigger::new(app_handle.clone())
                        .on_updated(vehicle_telemetry.clone())
                    {
                        Ok(_) => {
                            println!("Successfully emitted heartbeat status update via event trigger");
                        }
                        Err(e) => {
                            println!(
                                "Failed to emit heartbeat status update via event trigger: {}",
                                e
                            );

                            // Fallback to regular app_handle emit
                            let payload = json!({
                                "type": "heartbeat_update",
                                "telemetry": vehicle_telemetry
                            });
                            if let Err(e) = app_handle.emit("telemetry_update", &payload) {
                                println!("Failed to emit heartbeat status update: {}", e);
                            }
                        }
                    }
                }
            }
        }
    });
}

// Update heartbeat for a vehicle
pub async fn update_vehicle_heartbeat(
    vehicle_id: &str,
    heartbeats: Arc<Mutex<HashMap<String, VehicleHeartbeat>>>,
    state: Arc<Mutex<VehicleTelemetryData>>,
) {
    let mut heartbeats_guard = heartbeats.lock().await;
    if let Some(heartbeat) = heartbeats_guard.get_mut(vehicle_id) {
        let was_disconnected = !heartbeat.is_connected;
        heartbeat.update();

        if was_disconnected {
            println!(
                "Vehicle {} reconnected after being disconnected",
                vehicle_id
            );

            // Update vehicle status back to normal if it was disconnected
            let mut state_guard = state.lock().await;
            match vehicle_id {
                "eru" => {
                    if state_guard.ERU.vehicle_status == "Disconnected" {
                        state_guard.ERU.vehicle_status = "Connected".to_string();
                    }
                }
                "mea" => {
                    if state_guard.MEA.vehicle_status == "Disconnected" {
                        state_guard.MEA.vehicle_status = "Connected".to_string();
                    }
                }
                "mra" => {
                    if state_guard.MRA.vehicle_status == "Disconnected" {
                        state_guard.MRA.vehicle_status = "Connected".to_string();
                    }
                }
                _ => {
                    println!("Unknown vehicle_id for reconnection: {}", vehicle_id);
                }
            }
        }
    }
}

// Get heartbeat status for all vehicles
pub async fn get_heartbeat_status(
    heartbeats: Arc<Mutex<HashMap<String, VehicleHeartbeat>>>,
) -> HashMap<String, VehicleHeartbeat> {
    heartbeats.lock().await.clone()
}

// Check if a specific vehicle is connected
pub async fn is_vehicle_connected(
    vehicle_id: &str,
    heartbeats: Arc<Mutex<HashMap<String, VehicleHeartbeat>>>,
    timeout: Duration,
) -> bool {
    let heartbeats_guard = heartbeats.lock().await;
    heartbeats_guard
        .get(vehicle_id)
        .map(|h| h.is_connected && !h.is_timeout(timeout))
        .unwrap_or(false)
}
