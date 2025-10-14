use anyhow::Result;
use axum::Router;
use medbook_inventoryservice::{app_state::AppState, consumers, outbox, routes};
use tracing::info;
use tracing_subscriber::filter::LevelFilter;

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

    consumers::init(
        "inventory.order_requested".into(),
        consumers::inventory::reserve_stock,
        app_state.clone(),
    );

    outbox::init(app_state.clone());

    let app = Router::new()
        .nest("/products", routes::products::routes())
        .nest("/inventory", routes::inventory::routes())
        .route("/health-check", axum::routing::get(|| async { "OK" }))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}
