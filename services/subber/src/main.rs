use anyhow::{anyhow, Result};
use common::{init_logging, log_effective_config, AppConfig};
use futures::StreamExt;
use log::{warn};

mod pg;
use pg::{connect_and_migrate, insert_word};

#[cfg(all(feature = "mock", not(feature = "gcp")))]
#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    init_logging();
    let cfg = AppConfig::from_env()?;
    log_effective_config(&cfg);
    // In mock mode without GCP, just ensure DB is reachable
    // and the table exists, then exit.
    if let Some(url) = &cfg.database_url {
        let _client = connect_and_migrate(url).await?;
        warn!("Subber mock mode: migration complete; no messages consumed without GCP.");
    } else {
        warn!("Subber mock mode: DATABASE_URL not set; nothing to do.");
    }
    Ok(())
}

#[cfg(feature = "gcp")]
#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    use google_cloud_pubsub::client::{Client, ClientConfig};

    init_logging();
    let cfg = AppConfig::from_env()?;
    log_effective_config(&cfg);

    let sub_id = cfg
        .subscription_id
        .clone()
        .ok_or_else(|| anyhow!("SUBSCRIPTION_ID must be set for gcp mode"))?;

    let client = connect_and_migrate(
        cfg.database_url
            .as_deref()
            .ok_or_else(|| anyhow!("DATABASE_URL must be set for subber"))?,
    )
    .await?;

    let config = ClientConfig::default().with_auth().await?;
    let gcp = Client::new(config).await?;
    let subscription = gcp.subscription(&sub_id);

    let mut stream = subscription.subscribe(None).await?;
    while let Some(message) = stream.next().await {
        let data = String::from_utf8(message.message.data.to_vec()).unwrap_or_default();
        if !data.is_empty() {
            insert_word(&client, &data).await?;
        } else {
            warn!("Received empty message; skipping");
        }
        message.ack().await?;
    }
    Ok(())
}
