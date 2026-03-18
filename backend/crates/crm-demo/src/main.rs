#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("crm-demo server starting");
}
