use medbook_paymentservice::{app_state, consumers, outbox, routes};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing_subscriber::filter::LevelFilter::INFO)
        .init();

    dotenvy::dotenv().ok();
    tracing::info!(".env files loaded");

    let ip = format!("0.0.0.0:{}", std::env::var("PORT")?);
    tracing::info!("Starting PaymentService on {}...", ip);

    let app_state = app_state::AppState::init().await?;

    consumers::init(
        "payments.pay_request".into(),
        consumers::payments::pay_request,
        app_state.clone(),
    );

    // consumers::init(
    //     "orders.order_rejected".into(),
    //     consumers::orders::order_rejected,
    //     app_state.clone(),
    // );

    outbox::init(app_state.clone());

    let app = axum::Router::new()
        .nest("/payments", routes::payments::routes())
        .route("/health-check", axum::routing::get(|| async { "OK" }))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}
