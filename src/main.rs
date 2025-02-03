pub mod args;
pub mod error;
pub mod provider;
pub mod utils;

use args::CLI;
use clap::Parser;
use provider::{AmarisRegistry, DynamicProvider};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli: CLI = CLI::parse();

    let mut registry: AmarisRegistry = AmarisRegistry::new();

    let providers = DynamicProvider::load_all(None).await?;
    for provider in providers {
        registry.register(provider);
    }

    match cli.command.execute(&registry).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
