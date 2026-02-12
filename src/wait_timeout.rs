use std::fmt::Display;
use std::time::Duration;

use thiserror::Error;
use tokio::time;

#[derive(Error, Debug)]
#[error("Timeout ({duration} ms) during {context}")]
pub struct Timeout {
    pub duration: u64,
    pub context: String,
}

pub async fn wait<F, C, CF>(
    millis: u64,
    future: F,
    context: CF,
) -> anyhow::Result<F::Output, Timeout>
where
    F: IntoFuture,
    C: Display + Send + Sync + 'static,
    CF: FnOnce() -> C,
{
    if millis == 0 {
        Ok(future.await)
    } else {
        time::timeout(Duration::from_millis(millis), future)
            .await
            .map_err(|_| Timeout {
                duration: millis,
                context: context().to_string(),
            })
    }
}
