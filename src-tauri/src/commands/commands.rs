use lapin::{
    options::{BasicConsumeOptions, BasicPublishOptions, QueueDeclareOptions},
    types::FieldTable,
    BasicProperties, Connection, ConnectionProperties,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::Arc;
use taurpc::{procedures, resolvers};
use tokio::sync::Mutex;

#[derive(Debug, Deserialize, Serialize, Clone, Type)]
pub struct GeoCoordinate {
    pub lat: f64,
    pub long: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Type)]
pub struct CommandsStruct {
    pub vehicle_id: String,
    pub commandID: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coordinates: Option<Vec<GeoCoordinate>>,
}

type SharedCommands = Arc<Mutex<CommandsStruct>>;

#[procedures(export_to = "../src/lib/bindings.ts", path = "commands")]
pub trait CommandsApi {
    async fn send_emergency_stop(vehicle_id: String) -> Result<(), String>;
    async fn send_mission_update(vehicle_id: String, mission_id: String) -> Result<(), String>;
    async fn send_zone_update(
        vehicle_id: String,
        zone_id: String,
        coordinates: Vec<GeoCoordinate>,
    ) -> Result<(), String>;
}

#[derive(Clone)]
pub struct CommandsApiImpl {
    state: SharedCommands,
}

// Implementation of our own "RPC", using rust:
// Basic idea:
// - create_consumer for commands ack ??idk
// -

impl Default for CommandsApiImpl {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(CommandsStruct {
                vehicle_id: "default".to_string(),
                commandID: 0,
                coordinates: None,
            })),
        }
    }
}

#[resolvers]
impl CommandsApi for CommandsApiImpl {
    async fn send_emergency_stop(self, vehicle_id: String) -> Result<(), String> {
        let mut state = self.state.lock().await;
        state.vehicle_id = vehicle_id; // This will be "ALL" for all vehicles or specific vehicle name
        state.commandID = 1; // Emergency stop command ID
        state.coordinates = None;
        self.publish_command_to_rabbitmq(&state).await?;
        Ok(())
    }

    async fn send_mission_update(
        self,
        vehicle_id: String,
        mission_id: String,
    ) -> Result<(), String> {
        let mut state = self.state.lock().await;
        state.vehicle_id = vehicle_id;
        state.commandID = mission_id.parse().unwrap_or(0);
        state.coordinates = None;
        self.publish_command_to_rabbitmq(&state).await?;
        Ok(())
    }

    async fn send_zone_update(
        self,
        vehicle_id: String,
        zone_id: String,
        coordinates: Vec<GeoCoordinate>,
    ) -> Result<(), String> {
        let mut state = self.state.lock().await;
        state.vehicle_id = vehicle_id;
        state.commandID = zone_id.parse().unwrap_or(0);
        state.coordinates = Some(coordinates);
        self.publish_command_to_rabbitmq(&state).await?;
        Ok(())
    }
}

impl CommandsApiImpl {
    async fn publish_command_to_rabbitmq(&self, command: &CommandsStruct) -> Result<(), String> {
        // 1) Use %2f to select the "/" vhost
        let addr = std::env::var("AMQP_ADDR")
            .unwrap_or_else(|_| "amqp://admin:admin@localhost:5672/%2f".into());
        println!("Connecting to RabbitMQ at {}", addr);
        let conn = Connection::connect(&addr, ConnectionProperties::default())
            .await
            .map_err(|e| format!("Failed to connect to RabbitMQ: {}", e))?;
        println!("Connected");

        // 2) Open channel
        let channel = conn
            .create_channel()
            .await
            .map_err(|e| format!("Failed to create channel: {}", e))?;
        println!("Created channel");

        // 3) Declare queue (durable)
        let queue = channel
            .queue_declare(
                "vehicle_commands",
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await
            .map_err(|e| format!("Failed to declare queue: {}", e))?;
        println!(
            "Queue 'vehicle_commands' ready count = {}",
            queue.message_count()
        );

        // 4) Serialize & publish to default exchange
        let payload = serde_json::to_vec(command)
            .map_err(|e| format!("Failed to serialize command: {}", e))?;
        println!("Serialized command: {:?}", command);

        let confirm = channel
            .basic_publish(
                "",
                "vehicle_commands",
                BasicPublishOptions {
                    mandatory: true,
                    ..Default::default()
                },
                &payload,
                BasicProperties::default().with_delivery_mode(2), // Make message persistent
            )
            .await
            .map_err(|e| format!("Failed to publish: {}", e))?;
        confirm
            .await
            .map_err(|e| format!("Publish confirm failed: {}", e))?;
        println!("Published command via default exchange");

        // 5) Close
        conn.close(0, "")
            .await
            .map_err(|e| format!("Failed to close connection: {}", e))?;
        println!("Closed connection");

        Ok(())
    }

    // Command Ack struct
    async fn receive_command_to_rabbitmq(&self, command: &CommandsStruct) -> Result<(), String> {
        // 1) Use %2f to select the "/" vhost
        let addr = std::env::var("AMQP_ADDR")
            .unwrap_or_else(|_| "amqp://admin:admin@localhost:5672/%2f".into());
        println!("Connecting to RabbitMQ at {}", addr);
        let conn = Connection::connect(&addr, ConnectionProperties::default())
            .await
            .map_err(|e| format!("Failed to connect to RabbitMQ: {}", e))?;
        println!("Connected");

        // 2) Open channel
        let channel = conn
            .create_channel()
            .await
            .map_err(|e| format!("Failed to create channel: {}", e))?;
        println!("Created channel");

        // 3) Declare queue (durable)
        let queue_name = "vehicle_commands";
        channel
            .queue_declare(
                queue_name,
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await
            .map_err(|e| format!("Failed to declare queue: {}", e))?;
        println!("Queue {} declared", queue_name);

        let consumer = channel
            .basic_consume(
                queue_name,
                "consumer_tag",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;

        while let Some(delivery) = consumer.next().await {
            if let Ok(delivery) = delivery {
                let message_body = std::str::from_utf8(&delivery.data).unwrap_or("Invalid UTF-8");
                println!("Received message: {}", message_body);

                // Acknowledge the message to remove it from the queue
                channel.basic_ack(delivery.delivery_tag, false).await?;
            } else {
                // Handle cases where the delivery itself is an error
                eprintln!("Error in message delivery");
            }
        }

        Ok(())
    }
}
