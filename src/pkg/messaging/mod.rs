// Consumer and Producer modules
mod consumer;
mod producer;
mod util;

// Task module - contains interfaces/traits for task handling
pub mod task;

// Re-export consumer types
pub use consumer::{ConsumerConfig, MessageConsumer, create_consumer};

// Re-export Kafka utilities
pub use util::kafka_util::ensure_topics_exist;

// Re-export producer types
pub use producer::{MessageProducer, ProducerConfig, create_producer};

// Re-export task types
pub use task::{TaskEvent, TaskHandler, TaskPriority};
