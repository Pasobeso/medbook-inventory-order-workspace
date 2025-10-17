use medbook_deliveryservice::{app_state, consumers, outbox, routes};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing_subscriber::filter::LevelFilter::INFO)
        .init();

    dotenvy::dotenv().ok();
    tracing::info!(".env files loaded");

    let ip = format!("0.0.0.0:{}", std::env::var("PORT")?);
    tracing::info!("Starting DeliveryService on {}...", ip);

    let app_state = app_state::AppState::init().await?;

    consumers::init(
        "delivery.order_success".into(),
        consumers::delivery::order_success,
        app_state.clone(),
    );

    outbox::init(app_state.clone());

    let app = axum::Router::new()
        .nest("/delivery", routes::delivery::routes())
        .route("/health-check", axum::routing::get(|| async { "OK" }))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}
