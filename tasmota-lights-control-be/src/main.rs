mod adapters;
mod application;
mod bootstrap;
mod config;
mod domain;
mod error;
mod http;
mod ports;

#[tokio::main]
async fn main() {
    bootstrap::run().await;
}
