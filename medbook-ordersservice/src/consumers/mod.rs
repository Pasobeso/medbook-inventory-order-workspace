use anyhow::Result;
use futures::future::BoxFuture;
use futures_lite::StreamExt;
use lapin::message::Delivery;
use std::time::Duration;
use tracing::info;

use crate::app_state::AppState;
pub mod orders;

type ConsumerFn = fn(Delivery, AppState) -> BoxFuture<'static, Result<()>>;

pub fn start(queue_name: String, consumer_fn: ConsumerFn, state: AppState) {
    tokio::spawn(async move {
        loop {
            let state = state.clone();
            let queue_name = &queue_name;

            let future = Box::pin(async move {
                let channel = state.rmq_client.create_channel().await?;
                channel.create_queue(queue_name).await?;
                let mut consumer = channel.create_consumer(queue_name, queue_name).await?;

                info!("Consumer {} created", queue_name);

                while let Some(delivery) = consumer.next().await {
                    let delivery = delivery?;
                    consumer_fn(delivery, state.clone()).await?
                }

                Ok::<_, anyhow::Error>(())
            });

            match future.await {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("Error occured in consumer \"{}\": {:?}", queue_name, e);
                    tracing::error!("Retrying in 5 seconds...");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    });
}
