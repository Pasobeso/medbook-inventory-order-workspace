use std::time::Duration;

use anyhow::Result;
use axum::Router;
use tracing::{error, info};
use tracing_subscriber::filter::LevelFilter;

use crate::app_state::AppState;

mod app_error;
mod app_state;
mod consumers;
mod db;
mod models;
mod outbox;
mod routes;
mod schema;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .init();

    dotenvy::dotenv().ok();
    info!(".env files loaded");

    let ip = format!("0.0.0.0:{}", std::env::var("PORT")?);
    info!("Starting server on {}...", ip);

    let app_state = AppState::init().await?;

    consumers::start(
        "inventory.order_requested".into(),
        consumers::inventory::reserve_stock,
        app_state.clone(),
    );

    let outbox_app_state = app_state.clone();
    tokio::spawn(async move {
        loop {
            if let Err(e) = outbox::start(outbox_app_state.clone()).await {
                error!("Error occured in outbox loop: {:?}", e);
                error!("Retrying in 5 seconds...");
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    });

    let app = Router::new()
        .nest("/products", routes::products::routes())
        .nest("/inventory", routes::inventory::routes())
        .route("/health-check", axum::routing::get(|| async { "OK" }))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}
