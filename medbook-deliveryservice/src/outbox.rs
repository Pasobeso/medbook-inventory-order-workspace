use std::time::Duration;

use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use tracing::{error, info};

use crate::{app_state::AppState, models::OutboxEntity, schema::outbox};

pub fn init(state: AppState) {
    info!("Outbox initialized");
    tokio::spawn(async move {
        loop {
            if let Err(e) = start(state.clone()).await {
                error!("Error occured in outbox loop: {:?}", e);
                error!("Retrying in 5 seconds...");
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    });
}

async fn start(state: AppState) -> anyhow::Result<()> {
    let conn = &mut state.db_pool.get().await?;
    let channel = state.rmq_client.create_channel().await?;

    loop {
        info!("Processing outbox...");

        let events: Vec<OutboxEntity> = outbox::table
            .filter(outbox::status.eq("PENDING"))
            .select(OutboxEntity::as_select())
            .get_results(conn)
            .await?;

        if events.len() == 0 {
            info!("No events to process, sleeping for 5 seconds...");
            tokio::time::sleep(Duration::from_secs(5)).await;
        } else {
            for event in events {
                let queue = channel.create_queue(&event.event_type).await?;
                match queue.publish_plain(&event.payload).await {
                    Ok(_) => {
                        diesel::update(outbox::table.filter(outbox::id.eq(event.id)))
                            .set(outbox::status.eq("PROCESSED"))
                            .execute(conn)
                            .await?;
                        info!(
                            "Outbox event #{} ({}) has been published",
                            event.id, event.event_type
                        )
                    }
                    Err(e) => {
                        tracing::error!(
                            "An error occured while publishing outbox event #{} ({}): {}",
                            event.id,
                            event.event_type,
                            e
                        )
                    }
                };
            }
        }
    }
}
