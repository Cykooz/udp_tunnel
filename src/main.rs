use std::process;

use tokio::signal;
#[cfg(feature = "tracing")]
pub use tracing::{error, info};
use udp_tunnel::{Settings, run_tunnel};
#[cfg(not(feature = "tracing"))]
use udp_tunnel::{error, info};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let settings = Settings::new();

    #[cfg(feature = "tracing")]
    {
        use tracing_subscriber::fmt::Subscriber;
        use tracing_subscriber::util::SubscriberInitExt;
        let builder = Subscriber::builder()
            .with_max_level(settings.log_level)
            .with_target(false)
            .with_ansi(!settings.without_ansi_color)
            .without_time();

        let subscriber = builder.finish();
        subscriber.try_init()?;
    };

    let run_handle = tokio::spawn(run_tunnel(
        settings.listen,
        settings.primary,
        settings.secondary,
        settings.keepalive_timeout,
    ));

    tokio::select! {
        r = run_handle => {r?},
        v = signal::ctrl_c() => {
            // Wait for the Ctrl+C signal
            match v {
                Ok(()) => {
                    info!("Received Ctrl+C, starting shutdown...");
                    Ok(())
                }
                Err(err) => {
                    error!("Unable to listen for shutdown signal: {err}");
                    // Exit with an error code if the signal handler couldn't be installed
                    process::exit(1);
                }
            }
        },
    }
}
