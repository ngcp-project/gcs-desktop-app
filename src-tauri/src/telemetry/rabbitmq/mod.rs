mod heartbeat;
mod listen;
mod process;

// Re-export public types
pub use heartbeat::VehicleHeartbeat;

use crate::telemetry::types::VehicleTelemetryData;
use lapin::{Channel, Connection, ConnectionProperties, Result as LapinResult};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tauri::AppHandle;
use taurpc;
use tokio::sync::Mutex;
use tokio_amqp::*;

// Constants
const RABBITMQ_ADDR: &str = "amqp://admin:admin@localhost:5672/%2f";
const DATABASE_URL: &str = "postgres://ngcp:ngcp@localhost:5433/ngcpdb";
const VALID_VEHICLE_IDS: [&str; 4] = ["eru", "fra", "mea", "mra"];
const DEFAULT_HEARTBEAT_TIMEOUT_SECS: u64 = 10; // 30 seconds timeout
const DEFAULT_HEARTBEAT_CHECK_INTERVAL_SECS: u64 = 1; // Check every 10 seconds

#[derive(Clone)]
pub struct RabbitMQAPIImpl {
    connection: Arc<Mutex<Connection>>,
    state: Arc<Mutex<VehicleTelemetryData>>,
    channel: Channel,
    db: PgPool,
    app_handle: Option<AppHandle>,
    // Heartbeat tracking
    vehicle_heartbeats: Arc<Mutex<HashMap<String, VehicleHeartbeat>>>,
    heartbeat_timeout: Duration,
    heartbeat_check_interval: Duration,
}

impl RabbitMQAPIImpl {
    pub async fn new() -> LapinResult<Self> {
        let connection =
            Connection::connect(RABBITMQ_ADDR, ConnectionProperties::default().with_tokio())
                .await?;

        let connection = Arc::new(Mutex::new(connection));
        let channel = connection.lock().await.create_channel().await?;

        // Initialize heartbeat tracking for all valid vehicles
        let mut vehicle_heartbeats = HashMap::new();
        for vehicle_id in VALID_VEHICLE_IDS.iter() {
            vehicle_heartbeats.insert(vehicle_id.to_string(), VehicleHeartbeat::new());
        }

        let database_connection = PgPoolOptions::new()
            .max_connections(5)
            .connect(DATABASE_URL)
            .await
            .expect("Failed to connect to the database");
        let db = database_connection;

        let consumer = Self {
            connection,
            channel,
            db,
            state: Arc::new(Mutex::new(VehicleTelemetryData::default())),
            app_handle: None,
            vehicle_heartbeats: Arc::new(Mutex::new(vehicle_heartbeats)),
            heartbeat_timeout: Duration::from_secs(DEFAULT_HEARTBEAT_TIMEOUT_SECS),
            heartbeat_check_interval: Duration::from_secs(DEFAULT_HEARTBEAT_CHECK_INTERVAL_SECS),
        };

        Ok(consumer)
    }

    // Method to set the app handle after initialization
    pub fn with_app_handle(mut self, app_handle: AppHandle) -> Self {
        self.app_handle = Some(app_handle);
        self
    }

    // Method to configure heartbeat settings
    pub fn with_heartbeat_config(mut self, timeout_secs: u64, check_interval_secs: u64) -> Self {
        self.heartbeat_timeout = Duration::from_secs(timeout_secs);
        self.heartbeat_check_interval = Duration::from_secs(check_interval_secs);
        self
    }

    // Initialize all consumers and start heartbeat monitoring
    pub async fn init_consumers(&self) -> LapinResult<()> {
        // Start heartbeat monitor
        heartbeat::start_heartbeat_monitor(
            self.vehicle_heartbeats.clone(),
            self.state.clone(),
            self.app_handle.clone(),
            self.heartbeat_timeout,
            self.heartbeat_check_interval,
        )
        .await;

        for vehicle_id in VALID_VEHICLE_IDS.iter() {
            let queue_name = format!("telemetry_{}", vehicle_id);
            println!("Initializing consumer for queue: {}", queue_name);

            // Declare queue first
            listen::queue_declare(&self.channel, &queue_name).await?;

            tokio::spawn({
                let consumer = self.clone();
                let queue = queue_name.clone();
                async move {
                    if let Err(e) = consumer.start_consuming(&queue).await {
                        eprintln!("Failed to consume from queue {}: {}", queue, e);
                    }
                }
            });
        }

        Ok(())
    }

    // Start consuming from a specific queue
    pub async fn start_consuming(&self, queue_name: &str) -> LapinResult<()> {
        let consumer = listen::create_consumer(&self.channel, queue_name).await?;
        process::process_telemetry(
            consumer,
            self.state.clone(),
            self.db.clone(),
            self.app_handle.clone(),
            self.vehicle_heartbeats.clone(),
            self.heartbeat_timeout,
        )
        .await?;
        Ok(())
    }

    // Get heartbeat status for all vehicles
    pub async fn get_heartbeat_status(&self) -> HashMap<String, VehicleHeartbeat> {
        heartbeat::get_heartbeat_status(self.vehicle_heartbeats.clone()).await
    }

    // Check if a specific vehicle is connected
    pub async fn is_vehicle_connected(&self, vehicle_id: &str) -> bool {
        heartbeat::is_vehicle_connected(
            vehicle_id,
            self.vehicle_heartbeats.clone(),
            self.heartbeat_timeout,
        )
        .await
    }
}

// TauRPC trait definition
#[taurpc::procedures(
    event_trigger = TelemetryEventTrigger,
    export_to = "../src/lib/bindings.ts",
    path = "telemetry"
)]
pub trait RabbitMQAPI {
    #[taurpc(event)]
    async fn on_updated(new_data: VehicleTelemetryData);

    // State Management
    async fn get_default_data() -> VehicleTelemetryData;
    async fn get_telemetry() -> VehicleTelemetryData;

    // Heartbeat Management
    // async fn get_heartbeat_status() -> HashMap<String, VehicleHeartbeat>;
    // async fn is_vehicle_connected(vehicle_id: String) -> bool;
}

// Implementation of the TauRPC trait for our API
#[taurpc::resolvers]
impl RabbitMQAPI for RabbitMQAPIImpl {
    async fn get_default_data(self) -> VehicleTelemetryData {
        Self::new().await.unwrap().state.lock().await.clone()
    }

    async fn get_telemetry(self) -> VehicleTelemetryData {
        self.state.lock().await.clone()
    }

    // async fn get_heartbeat_status(self) -> HashMap<String, VehicleHeartbeat> {
    //     self.get_heartbeat_status().await
    // }

    // async fn is_vehicle_connected(self, vehicle_id: String) -> bool {
    //     self.is_vehicle_connected(&vehicle_id).await
    // }
}
