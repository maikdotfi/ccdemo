use anyhow::{Context, Result};
use serde::Serialize;
use std::env;
use tokio_postgres::{Client, Config as PgConfig, NoTls, Row};
use log::info;

pub async fn connect(database_url: &str) -> Result<Client> {
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

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Postgres connection error: {e}");
        }
    });

    // Ensure table exists (safe to run repeatedly)
    info!("DB connected; ensuring words table exists");
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

#[derive(Debug, Clone, Serialize)]
pub struct Word {
    pub id: i32,
    pub word: String,
}

pub async fn list_words_page(cursor: Option<i32>, limit: i64) -> Result<(Vec<Word>, Option<i32>)> {
    // Rebuild from env every call for simplicity; low-traffic UI.
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://127.0.0.1:5432/wordsdb".into());
    let client = connect(&database_url).await?;

    info!("Query words page: cursor={:?}, limit={}", cursor, limit);
    let rows: Vec<Row> = if let Some(c) = cursor {
        client
            .query(
                "SELECT id, word FROM words WHERE id > $1 ORDER BY id ASC LIMIT $2",
                &[&c, &limit],
            )
            .await
            .context("Failed to fetch paginated words after cursor")?
    } else {
        client
            .query(
                "SELECT id, word FROM words ORDER BY id ASC LIMIT $1",
                &[&limit],
            )
            .await
            .context("Failed to fetch first page of words")?
    };

    let mut items: Vec<Word> = Vec::with_capacity(rows.len());
    let mut next_cursor: Option<i32> = None;
    for row in rows {
        let id: i32 = row.get(0);
        let word: String = row.get(1);
        next_cursor = Some(id);
        items.push(Word { id, word });
    }
    if items.len() < (limit as usize) {
        next_cursor = None;
    }
    info!("Query returned {} items; next_cursor={:?}", items.len(), next_cursor);
    Ok((items, next_cursor))
}
