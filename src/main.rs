mod annotations;
mod app;
mod crdt;
mod pdf;
mod protocol;
mod renderer;
mod tablet_server;

use anyhow::Result;
use tracing::info;
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    // Initialise logging. Set RUST_LOG=debug for verbose output.
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let args: Vec<String> = std::env::args().collect();
    let enable_tablet = args.iter().any(|a| a == "--tablet");

    let pdf_path: Option<String> = args
        .iter()
        .skip(1)
        .find(|a| !a.starts_with("..."))
        .cloned();

    info!("inkstone starting (tablet server={})", enable_tablet);

    app::run(app::AppConfig {
        pdf_path,
        enable_tablet,
    })
}
