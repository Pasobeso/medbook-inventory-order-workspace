use anyhow::{Context, Result};
use diesel::SelectableHelper;
use diesel_async::RunQueryDsl;
use futures::future::BoxFuture;
use lapin::{message::Delivery, options::BasicAckOptions};
use medbook_events::DeliveryOrderSuccessEvent;
use tracing::info;

use crate::{
    app_state::AppState,
    models::{CreateDeliveryEntity, DeliveryEntity},
    schema::delivery,
};

pub fn order_success(delivery: Delivery, state: AppState) -> BoxFuture<'static, Result<()>> {
    Box::pin(async move {
        let payload: DeliveryOrderSuccessEvent = serde_json::from_str(str::from_utf8(&delivery.data)?)?;
        info!("Received event: {:?}", payload);

        let conn = &mut state
            .db_pool
            .get()
            .await
            .context("Failed to obtain a DB connection pool")?;

        let deliv = diesel::insert_into(delivery::table)
            .values(CreateDeliveryEntity {
                order_id: payload.order_id,
                status: "PREPARING".into(),
            })
            .returning(DeliveryEntity::as_returning())
            .get_result(conn)
            .await
            .context("Failed to create delivery")?;

        info!("Delivery #{} has been created", deliv.id);

        delivery.ack(BasicAckOptions::default()).await?;

        Ok(())
    })
}
