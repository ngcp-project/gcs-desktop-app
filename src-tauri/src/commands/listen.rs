use futures_util::stream::StreamExt;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions, QueueDeclareOptions},
    types::FieldTable,
    Channel, Connection, ConnectionProperties, Consumer,
};
use serde_json::{Value, from_slice, to_string_pretty};

/// Start the command consumer as a long-lived background task.
/// Spawns the actual consumer loop which maintains reconnection on failure.
pub async fn start_command_consumer() -> Result<(), String> {
    tokio::spawn(async {
        loop {
            if let Err(e) = consumer_loop().await {
                eprintln!(
                    "[Consumer] Consumer loop error: {}. Reconnecting in 5s...",
                    e
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    });
    Ok(())
}

/// The consumer loop: connects to RabbitMQ, declares the queue, and processes messages.
/// Returns an error if connection fails or the stream ends unexpectedly.
async fn consumer_loop() -> Result<(), String> {
    // 1) Establish connection and create channel
    let channel = connect_to_rabbitmq().await?;
    println!("[Consumer] Connected to RabbitMQ");

    declare_queue(&channel).await?;

    // 2) Create queue and start consuming
    let consumer = create_consumer(&channel).await?;
    println!("[Consumer] Consumer started, waiting for messages...");

    // 3) Process incoming messages
    process_messages(consumer, &channel).await?;

    println!("[Consumer] Consumer stream ended, reconnecting...");
    Ok(())
}

/// Connect to RabbitMQ at the configured address.
async fn connect_to_rabbitmq() -> Result<Channel, String> {
    let addr = std::env::var("AMQP_ADDR")
        .unwrap_or_else(|_| "amqp://admin:admin@localhost:5672/%2f".into());
    println!("[Consumer] Connecting to RabbitMQ at {}", addr);

    let connection = Connection::connect(&addr, ConnectionProperties::default())
        .await
        .map_err(|e| format!("Failed to connect to RabbitMQ: {}", e));

    connection
        .unwrap()
        .create_channel()
        .await
        .map_err(|e| format!("Failed to create channel: {}", e))
}

/// Declare the command_ack queue as durable before consuming.
async fn declare_queue(channel: &Channel) -> Result<lapin::Queue, String> {
    let queue_name = "command_ack";
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
        .map_err(|e| format!("Failed to declare queue: {}", e))
}

/// Create a consumer for the command_ack queue.
async fn create_consumer(channel: &Channel) -> Result<Consumer, String> {
    let queue_name = "command_ack";
    channel
        .basic_consume(
            queue_name,
            " ",
            BasicConsumeOptions {
                no_ack: false, // We will manually ack
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
        .map_err(|e| format!("Failed to start consuming: {}", e))
}

async fn process_messages(consumer: Consumer, channel: &Channel) -> Result<(), String> {
    let mut consumer = Box::pin(consumer);

    while let Some(delivery_result) = consumer.next().await {
        match delivery_result {
            Ok(delivery) => {
                // Parse and handle the message
                handle_message(&delivery.data, channel, delivery.delivery_tag).await;
            }
            Err(e) => {
                eprintln!("[Consumer] Delivery error: {}. Reconnecting...", e);
                return Err(format!("Consumer stream error: {}", e));
            }
        }
    }

    Ok(())
}

/// Handle a single incoming command message.
/// Parses the message, processes it, and acknowledges it.
async fn handle_message(data: &[u8], channel: &Channel, delivery_tag: u64) {
    let message_body = std::str::from_utf8(data).unwrap_or("<Invalid UTF-8>");
    println!("[Consumer] Received message: {}", message_body);

    let formatted_body = match from_slice::<Value>(data) {
        Ok(json) => to_string_pretty(&json).unwrap_or_else(|_| "Error formatting JSON".to_string()),
        Err(_) => String::from_utf8_lossy(data).to_string(),
    };

    println!("--- New Message ---");
    println!("{}", formatted_body);
    println!("-------------------");
    
    // TODO: Parse the message into a CommandAckStruct
    // Acknowledge the message to remove it from the queue

    if let Err(e) = channel
        .basic_ack(delivery_tag, BasicAckOptions::default())
        .await
    {
        eprintln!(
            "[Consumer] Failed to ack message (tag={}): {}",
            delivery_tag, e
        );
    } else {
        println!("[Consumer] Message acknowledged (tag={})", delivery_tag);
    }
}
