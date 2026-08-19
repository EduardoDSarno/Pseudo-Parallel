mod accounts;
mod app;
mod coin;
mod config;
mod heatmap;
mod hyperliquid;
mod liquidation;
mod market;
mod position;
mod price_data;
mod volatility;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = simple_logger::init_with_level(log::Level::Info);
    app::run().await
}
