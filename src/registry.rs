use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use crate::error::ConfigError;

#[async_trait]
pub trait AmarisProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    async fn check_prerequisites(&self) -> Result<(), ConfigError>;
    async fn install(&self) -> Result<(), ConfigError>;
    async fn remove(&self) -> Result<(), ConfigError>;
}

pub struct AmarisRegistry {
    providers: HashMap<String, Arc<dyn AmarisProvider>>,
}

impl AmarisRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn register<P: AmarisProvider + 'static>(&mut self, provider: P) {
        self.providers
            .insert(provider.name().to_string(), Arc::new(provider));
    }

    pub fn get_provider(&self, name: &str) -> Option<Arc<dyn AmarisProvider>> {
        self.providers.get(name).cloned()
    }

    pub fn available_configs(&self) -> Vec<(&str, &str)> {
        self.providers
            .values()
            .map(|p| (p.name(), p.description()))
            .collect()
    }
}
