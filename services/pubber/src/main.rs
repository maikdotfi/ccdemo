use anyhow::Result;
use common::{init_logging, log_effective_config, read_words_from_file, AppConfig};
use log::info;
use std::time::Duration;
use std::{env};
#[cfg(not(feature = "gcp"))]
use std::thread;

#[cfg(not(feature = "gcp"))]
fn main() -> Result<()> {
    init_logging();
    let cfg = AppConfig::from_env()?;
    log_effective_config(&cfg);

    let delay_ms: u64 = env::var("PUBLISH_DELAY_MS").ok().and_then(|s| s.parse().ok()).unwrap_or(0);
    let words = read_words_from_file(&cfg.input_file)?;
    info!("Loaded {} words from {}", words.len(), cfg.input_file);

    for word in words {
        info!("[MOCK] Publish -> topic={} payload='{}'", cfg.topic_id, word);
        if delay_ms > 0 {
            thread::sleep(Duration::from_millis(delay_ms));
        }
    }
    info!("Publishing complete. Idling...");
    loop {
        thread::sleep(Duration::from_secs(60));
    }
}

#[cfg(feature = "gcp")]
#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    use google_cloud_googleapis::pubsub::v1::PubsubMessage;
    use google_cloud_pubsub::client::{Client, ClientConfig};

    init_logging();
    let cfg = AppConfig::from_env()?;
    log_effective_config(&cfg);

    let delay_ms: u64 = env::var("PUBLISH_DELAY_MS").ok().and_then(|s| s.parse().ok()).unwrap_or(0);
    let words = read_words_from_file(&cfg.input_file)?;
    info!("Loaded {} words from {}", words.len(), cfg.input_file);

    // Create pubsub client using ADC/emulator via explicit ClientConfig (compatible with crate version)
    let config = ClientConfig::default().with_auth().await?;
    let client = Client::new(config).await?;

    let topic = client.topic(&cfg.topic_id);
    let publisher = topic.new_publisher(None);

    for word in words {
        info!("[GCP] Publish -> topic={} payload='{}'", cfg.topic_id, word);
        let mut msg = PubsubMessage::default();
        msg.data = word.clone().into();
        // Set ordering_key
        msg.ordering_key = "order".into();
        // Enqueue publish and wait for server-generated ID
        let awaiter = publisher.publish(msg).await;
        let _ = awaiter.get().await?;
        if delay_ms > 0 { tokio::time::sleep(Duration::from_millis(delay_ms)).await; }
    }

    // Gracefully shutdown publisher
    let mut publisher = publisher;
    publisher.shutdown().await;

    info!("Publishing complete. Idling...");
    loop {
        tokio::time::sleep(Duration::from_secs(3600)).await;
    }
}
