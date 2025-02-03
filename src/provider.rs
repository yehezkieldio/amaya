use std::{collections::HashMap, path::PathBuf};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::{
    fs::{File, create_dir_all},
    io::AsyncReadExt,
};
use which::which;

use crate::{
    error::ConfigError,
    utils::{
        AmarisConfigurationHandler, AmarisInstaller, AmarisPackageJsonHandler, AmarisPathHandler,
    },
};

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigEntry {
    pub file_location: String,
    pub file_name: String,
    pub source_from: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ScriptEntry {
    pub name: String,
    pub script: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DynamicProvider {
    pub name: String,
    pub description: String,
    pub package_manager: String,
    pub packages: Vec<String>,
    pub configuration: Vec<ConfigEntry>,
    pub scripts: Vec<ScriptEntry>,
}

impl DynamicProvider {
    pub async fn load_all(dir: Option<&PathBuf>) -> Result<Vec<Self>, ConfigError> {
        let dir = match dir {
            Some(d) => d.clone(),
            None => AmarisPathHandler::ensure_provider_dir().await?,
        };

        let mut providers = vec![];
        let mut entries = tokio::fs::read_dir(&dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                let provider = tokio::fs::read(path).await?;
                let provider: DynamicProvider = serde_json::from_slice(&provider)?;
                providers.push(provider);
            }
        }

        Ok(providers)
    }
}

#[async_trait]
pub trait AmarisProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    async fn check_prerequisites(&self) -> Result<(), ConfigError>;
    async fn install(&self) -> Result<(), ConfigError>;
    async fn remove(&self) -> Result<(), ConfigError>;
}

struct DynamicProviderImpl {
    name: String,
    description: String,
    provider: DynamicProvider,
}

#[async_trait]
impl AmarisProvider for DynamicProviderImpl {
    fn name(&self) -> &'static str {
        Box::leak(self.name.clone().into_boxed_str())
    }

    fn description(&self) -> &'static str {
        Box::leak(self.description.clone().into_boxed_str())
    }

    async fn check_prerequisites(&self) -> Result<(), ConfigError> {
        which(&self.provider.package_manager).map_err(|_| {
            ConfigError::MissingPrerequisite("Package manager not found".to_string())
        })?;

        if !AmarisPackageJsonHandler::get_default_path().exists() {
            return Err(ConfigError::MissingPrerequisite(
                "package.json not found!".to_string(),
            ));
        }

        Ok(())
    }

    async fn install(&self) -> Result<(), ConfigError> {
        let configurations = &self.provider.configuration;

        println!("Installing packages...");
        AmarisInstaller::install(&self.provider.package_manager, &self.provider.packages).await?;

        println!("Writing configurations...");
        AmarisConfigurationHandler::write_configs(self.name.clone(), configurations).await?;

        println!("Writing scripts...");
        AmarisPackageJsonHandler::write_scripts(&self.provider.scripts).await?;

        println!("Done!");

        Ok(())
    }

    async fn remove(&self) -> Result<(), ConfigError> {
        let configurations = &self.provider.configuration;

        println!("Removing packages...");
        AmarisInstaller::remove(&self.provider.package_manager, &self.provider.packages).await?;

        println!("Removing configurations...");
        AmarisConfigurationHandler::remove_configs(configurations).await?;

        println!("Removing scripts...");
        AmarisPackageJsonHandler::remove_scripts(&self.provider.scripts).await?;

        println!("Done!");

        Ok(())
    }
}

pub struct AmarisRegistry {
    providers: HashMap<String, Box<dyn AmarisProvider>>,
}

impl AmarisRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn register(&mut self, provider: DynamicProvider) {
        let dynamic_provider = DynamicProviderImpl {
            name: provider.name.clone(),
            description: provider.description.clone(),
            provider,
        };
        self.providers.insert(
            dynamic_provider.name.clone(),
            Box::new(dynamic_provider) as Box<dyn AmarisProvider>,
        );
    }

    pub fn available_configs(&self) -> Vec<(&str, &str)> {
        self.providers
            .values()
            .map(|p| (p.name(), p.description()))
            .collect()
    }

    pub fn get_provider(&self, name: &str) -> Option<&Box<dyn AmarisProvider>> {
        self.providers.get(name)
    }
}

pub struct AmarisVisualStudioCodeHandler;

impl AmarisVisualStudioCodeHandler {
    pub fn get_default_path() -> PathBuf {
        PathBuf::from(".vscode/settings.json")
    }

    pub async fn read() -> Result<Value, ConfigError> {
        let settings_path = AmarisVisualStudioCodeHandler::get_default_path();

        if !settings_path.exists() {
            create_dir_all(settings_path.parent().unwrap())
                .await
                .map_err(|e| ConfigError::FileWriteError(e.to_string()))?;

            // Return empty object if file doesn't exist
            return Ok(serde_json::json!({}));
        }

        let mut file: File = tokio::fs::File::open(settings_path)
            .await
            .map_err(|e| ConfigError::FileWriteError(e.to_string()))?;

        let mut contents = String::new();

        let _ = file
            .read_to_string(&mut contents)
            .await
            .map_err(|e| ConfigError::FileWriteError(e.to_string()));

        serde_json::from_str(&contents).map_err(|e| ConfigError::ValidationError(e.to_string()))
    }
}
