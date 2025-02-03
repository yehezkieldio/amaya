use clap::Parser;
use clap::Subcommand;
use inquire::Select;

use crate::provider::AmarisRegistry;
use crate::utils::AmarisInitialConfigHandler;

#[derive(Parser)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(about = env!("CARGO_PKG_DESCRIPTION"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct CLI {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Installs a specific configuration.
    ///
    /// Optionally, you can provide a config file with `--config` or `-c`
    Install {
        #[arg(short, long)]
        config: Option<String>,
    },
    /// Lists all available configurations.
    List,
    /// Removes a specific configuration.
    ///
    /// This requires a config file to be specified via `--config` or `-c`
    Remove {
        #[arg(short, long)]
        config: String,
    },
    /// Runs diagnostic commands to check the system's state.
    Doctor,
    /// Populates the configuration root directory with default configurations.
    Init,
}

impl Commands {
    pub async fn execute(&self, registry: &AmarisRegistry) -> anyhow::Result<()> {
        match self {
            Commands::Install { config } => {
                let config_name = match config {
                    Some(name) => name.clone(),
                    None => {
                        let configs: Vec<(&str, &str)> = registry.available_configs();
                        let options: Vec<_> = configs.iter().map(|(_, desc)| *desc).collect();
                        let selection =
                            Select::new("Select configuration to install:", options).prompt()?;

                        configs
                            .iter()
                            .find(|(_, desc)| *desc == selection)
                            .map(|(name, _)| name.to_string())
                            .unwrap()
                    }
                };

                if let Some(provider) = registry.get_provider(&config_name) {
                    provider.check_prerequisites().await?;
                    provider.install().await?;
                }
            }
            Commands::List => {
                println!("Available configurations:");
                for (name, description) in registry.available_configs() {
                    println!("- {}: {}", name, description);
                }
            }
            Commands::Remove { config } => {
                if let Some(provider) = registry.get_provider(config) {
                    provider.remove().await?;
                }
            }
            Commands::Doctor => {
                for (name, _) in registry.available_configs() {
                    if let Some(provider) = registry.get_provider(&name) {
                        match provider.check_prerequisites().await {
                            Ok(_) => println!("All prerequisites met for {}", name),
                            Err(e) => println!("All prerequisites are not met for {}\n{}", name, e),
                        }
                    }
                }
            }
            Commands::Init => {
                AmarisInitialConfigHandler::ensure_dirs().await?;
                AmarisInitialConfigHandler::create_initial_config().await?;
            }
        }
        Ok(())
    }
}
