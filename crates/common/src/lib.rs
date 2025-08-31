use anyhow::{anyhow, Context, Result};
use log::{info, warn};
use std::env;
use std::fs;

pub fn init_logging() {
    // Idempotent init: subsequent calls are no-ops
    let _ = env_logger::builder()
        .format_timestamp_secs()
        .try_init();
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub project_id: Option<String>,
    pub topic_id: String,
    pub subscription_id: Option<String>,
    pub input_file: String,
    pub database_url: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        let project_id = env::var("PROJECT_ID").ok();
        let topic_id = env::var("TOPIC_ID").unwrap_or_else(|_| "ccdemo-topic".to_string());
        let subscription_id = env::var("SUBSCRIPTION_ID").ok();
        let input_file = env::var("INPUT_FILE").unwrap_or_else(|_| "rick.txt".to_string());
        let database_url = env::var("DATABASE_URL").ok();

        if topic_id.trim().is_empty() {
            return Err(anyhow!("TOPIC_ID must not be empty"));
        }

        Ok(Self {
            project_id,
            topic_id,
            subscription_id,
            input_file,
            database_url,
        })
    }
}

pub fn read_words_from_file(path: &str) -> Result<Vec<String>> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("Failed to read input file: {}", path))?;
    let words = contents
        .split_whitespace()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    Ok(words)
}

// Placeholder for DB persistence — to be implemented with Cloud SQL later.
pub fn persist_word_mock(word: &str) -> Result<()> {
    info!("Persist (mock): {}", word);
    Ok(())
}

// Helper used by subber mock to decide an alternate source file for simulation.
pub fn mock_source_file_from_env() -> String {
    env::var("MOCK_SOURCE_FILE").unwrap_or_else(|_| env::var("INPUT_FILE").unwrap_or_else(|_| "rick.txt".to_string()))
}

pub fn log_effective_config(cfg: &AppConfig) {
    let project = cfg
        .project_id
        .as_deref()
        .unwrap_or("<unset: uses ADC or default>");
    let sub = cfg
        .subscription_id
        .as_deref()
        .unwrap_or("<unset>");
    warn!(
        "Config: project_id={}, topic_id={}, subscription_id={}, input_file={}, database_url={}",
        project, cfg.topic_id, sub, cfg.input_file, cfg.database_url.as_deref().unwrap_or("<unset>")
    );
}
