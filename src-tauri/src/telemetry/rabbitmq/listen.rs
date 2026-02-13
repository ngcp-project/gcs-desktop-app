use lapin::{
    options::*, types::FieldTable, Channel, Consumer, Queue, Result as LapinResult,
};

// Declare a queue for the consumer
pub async fn queue_declare(channel: &Channel, queue_name: &str) -> LapinResult<Queue> {
    channel
        .queue_declare(
            queue_name,
            QueueDeclareOptions {
                durable: true,
                auto_delete: false,
                exclusive: false,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
}

// Create a consumer for a specific queue
pub async fn create_consumer(channel: &Channel, queue_name: &str) -> LapinResult<Consumer> {
    // Generate unique consumer tag using queue name and timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let consumer_tag = format!("consumer_{}_{}", queue_name, timestamp);

    println!("Creating consumer with tag: {}", consumer_tag);

    channel
        .basic_consume(
            queue_name,
            &consumer_tag,
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
}
