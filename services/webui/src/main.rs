use anyhow::Result;
use axum::{routing::get, Router, response::Html};
use common::{init_logging, log_effective_config, AppConfig};
use serde::Deserialize;
use std::net::SocketAddr;
use std::time::Instant;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use log::info;

mod pg;
use pg::{connect, list_words_page};

async fn index_html() -> Html<String> {
    info!("GET / -> index_html");
    let html = r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>ccdemo — Words</title>
    <style>
      body { font-family: system-ui, -apple-system, Segoe UI, Roboto, sans-serif; margin: 2rem; }
      h1 { margin-bottom: .5rem; }
      .muted { color: #666; }
      code { background: #f4f4f4; padding: 2px 4px; border-radius: 3px; }
      .bar { display:flex; gap:1rem; align-items:center; margin-bottom:1rem; flex-direction:column; }
      #list { max-width: 48rem; margin: 0 auto; }
      table { width: 100%; border-collapse: collapse; }
      thead th { text-align: left; font-weight: 600; border-bottom: 2px solid #ddd; padding: .5rem .6rem; position: sticky; top: 0; background: #fff; }
      tbody td { border-bottom: 1px solid #eee; padding: .45rem .6rem; }
      td.id { width: 6rem; color: #555; }
      tr.loading td { text-align: center; color: #666; }
      button { padding:.4rem .7rem; }
    </style>
    <script src="https://unpkg.com/htmx.org@1.9.12"></script>
  </head>
  <body>
    <div class="bar">
      <h1>Words</h1>
    </div>
    <div id="list">
      <table>
        <thead>
          <tr>
            <th>ID</th>
            <th>Word</th>
          </tr>
        </thead>
        <tbody id="rows" hx-get="/api/words" hx-trigger="load" hx-swap="innerHTML"></tbody>
      </table>
    </div>
  </body>
</html>"#;
    Html(html.to_string())
}

#[derive(Deserialize)]
struct WordsQuery { cursor: Option<i32>, limit: Option<i64> }

async fn api_words(axum::extract::Query(q): axum::extract::Query<WordsQuery>) -> Result<Html<String>, axum::http::StatusCode> {
    let lim: i64 = q.limit.unwrap_or(1).clamp(1, 200) as i64;
    info!("GET /api/words cursor={:?} limit={}", q.cursor, lim);
    match list_words_page(q.cursor, lim).await {
        Ok((items, next_cursor)) => {
            info!("/api/words -> {} items, next_cursor={:?}", items.len(), next_cursor);
            let mut html = String::new();
            for itm in items {
                html.push_str(&format!(
                    "<tr><td class='id'>{}</td><td class='word'>{}</td></tr>",
                    itm.id, itm.word
                ));
            }
            if let Some(cursor) = next_cursor {
                let next_url = format!("/api/words?cursor={}&limit={}", cursor, lim);
                html.push_str(&format!(
                    "<tr class='loading' hx-get='{}' hx-trigger='revealed' hx-swap='outerHTML'><td colspan='2'>Loading...</td></tr>",
                    next_url
                ));
            }
            Ok(Html(html))
        }
        Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn log_requests(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let start = Instant::now();
    let res = next.run(req).await;
    let status = res.status();
    let elapsed = start.elapsed();
    info!("{} {} -> {} ({} ms)", method, uri, status.as_u16(), elapsed.as_millis());
    res
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    init_logging();
    let cfg = AppConfig::from_env()?;
    log_effective_config(&cfg);

    if let Some(url) = &cfg.database_url {
        let _ = connect(url).await?;
    }

    let app = Router::new()
        .route("/", get(index_html))
        .route("/api/words", get(api_words))
        .layer(axum::middleware::from_fn(log_requests));

    let port: u16 = std::env::var("PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8080);
    let addr: SocketAddr = ([0, 0, 0, 0], port).into();
    info!("webui starting on http://{}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}
