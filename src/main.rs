pub mod args;
pub mod error;
pub mod providers;
pub mod registry;

use args::CLI;
use clap::Parser;
use providers::biome::BiomeProvider;
use registry::AmarisRegistry;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli: CLI = CLI::parse();

    let mut registry: AmarisRegistry = AmarisRegistry::new();
    registry.register(BiomeProvider);

    match cli.command.execute(&registry).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
