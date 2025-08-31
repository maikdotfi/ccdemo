use anyhow::{Context, Result};
use std::env;
use tokio_postgres::{Client, Config as PgConfig, NoTls};

pub async fn connect_and_migrate(database_url: &str) -> Result<Client> {
    // Start from DATABASE_URL (may omit credentials), then override with DB_* envs.
    let mut cfg = match database_url.parse::<PgConfig>() {
        Ok(c) => c,
        Err(_) => PgConfig::new(),
    };

    if let Ok(host) = env::var("DB_HOST") {
        cfg.host(&host);
    }
    if let Ok(port_s) = env::var("DB_PORT") {
        if let Ok(port) = port_s.parse::<u16>() {
            cfg.port(port);
        }
    }
    if let Ok(dbname) = env::var("DB_NAME") {
        cfg.dbname(&dbname);
    }
    if let Ok(user) = env::var("DB_USER") {
        cfg.user(&user);
    }
    if let Ok(pass) = env::var("DB_PASS") {
        // No URL encoding required; sent directly to the driver.
        cfg.password(pass);
    }

    if cfg.get_hosts().is_empty() {
        cfg.host("127.0.0.1");
    }
    if cfg.get_ports().is_empty() {
        cfg.port(5432);
    }

    let (client, connection) = cfg
        .connect(NoTls)
        .await
        .with_context(|| "Failed to connect to Postgres")?;

    // Spawn the connection task to drive the I/O.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Postgres connection error: {e}");
        }
    });

    client
        .batch_execute(
            r#"
            CREATE TABLE IF NOT EXISTS words (
                id SERIAL PRIMARY KEY,
                word TEXT NOT NULL
            );
        "#,
        )
        .await
        .context("Failed to run migration for words table")?;

    Ok(client)
}

pub async fn insert_word(client: &Client, word: &str) -> Result<()> {
    client
        .execute("INSERT INTO words (word) VALUES ($1)", &[&word])
        .await
        .context("Failed to insert word")?;
    Ok(())
}
