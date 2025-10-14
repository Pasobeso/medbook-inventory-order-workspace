use medbook_ordersservice::{app_state, consumers, routes};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing_subscriber::filter::LevelFilter::INFO)
        .init();

    dotenvy::dotenv().ok();
    tracing::info!(".env files loaded");

    let ip = format!("0.0.0.0:{}", std::env::var("PORT")?);
    tracing::info!("Starting server on {}...", ip);

    let app_state = app_state::AppState::init().await?;

    consumers::start(
        "orders.order_reserved".into(),
        consumers::orders::order_reserved,
        app_state.clone(),
    );

    consumers::start(
        "orders.order_rejected".into(),
        consumers::orders::order_rejected,
        app_state.clone(),
    );

    let app = axum::Router::new()
        .nest("/orders", routes::orders::routes())
        .route("/health-check", axum::routing::get(|| async { "OK" }))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}
