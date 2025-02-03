use async_trait::async_trait;
use std::fs;
use std::path::Path;
use which::which;

use crate::error::ConfigError;
use crate::registry::AmarisProvider;

pub struct BiomeProvider;

impl BiomeProvider {
    pub fn configuration() -> serde_json::Value {
        serde_json::json!({
            "$schema": "https://biomejs.dev/schemas/1.9.4/schema.json",
            "extends": ["ultracite"],
            "vcs": {
                "enabled": true,
                "clientKind": "git",
                "useIgnoreFile": true,
                "defaultBranch": "master"
            },
            "organizeImports": {
                "enabled": true
            },
            "files": {
                "ignore": ["node_modules"]
            },
            "formatter": {
                "enabled": true,
                "formatWithErrors": false,
                "indentStyle": "space",
                "indentWidth": 4,
                "lineWidth": 120
            },
            "linter": {
                "enabled": true,
                "rules": {
                    "recommended": true,
                    "style": {
                        "noNonNullAssertion": "off",
                        "useForOf": "error",
                        "useNodejsImportProtocol": "error",
                        "useNumberNamespace": "error",
                        "noInferrableTypes": "warn"
                    },
                    "correctness": {
                        "noUnusedImports": "warn",
                        "noUnusedVariables": "info",
                        "noUnusedFunctionParameters": "info",
                        "useHookAtTopLevel": "off"
                    },
                    "complexity": {
                        "noStaticOnlyClass": "off",
                        "noThisInStatic": "off",
                        "noForEach": "error",
                        "noUselessSwitchCase": "error",
                        "useFlatMap": "error"
                    },
                    "suspicious": {
                        "noConsole": "off",
                        "noConsoleLog": "off"
                    },
                    "nursery": {
                        "useConsistentMemberAccessibility": "off",
                        "noNestedTernary": "off"
                    },
                    "performance": {
                        "useTopLevelRegex": "off"
                    }
                }
            },
            "javascript": {
                "formatter": {
                    "quoteStyle": "double",
                    "indentWidth": 4,
                    "lineWidth": 120
                },
                "globals": ["Bun"]
            },
            "json": {
                "formatter": {
                    "indentWidth": 4,
                    "indentStyle": "space"
                }
            }
        })
    }
}

#[async_trait]
impl AmarisProvider for BiomeProvider {
    fn name(&self) -> &'static str {
        "biome"
    }

    fn description(&self) -> &'static str {
        "Biome configuration for JavaScript/TypeScript projects"
    }

    async fn check_prerequisites(&self) -> Result<(), ConfigError> {
        which("bun").map_err(|_| {
            ConfigError::MissingPrerequisite("bun is required but not found".to_string())
        })?;

        if !Path::new("package.json").exists() {
            return Err(ConfigError::MissingPrerequisite(
                "package.json not found!".to_string(),
            ));
        }

        Ok(())
    }

    async fn install(&self) -> Result<(), ConfigError> {
        println!("Starting installation of Biome configuration...");
        println!("Installing packages with bun (installing @biomejs/biome and ultracite)...");

        let output = tokio::process::Command::new("bun")
            .args(&["install", "--dev", "@biomejs/biome", "ultracite"])
            .output()
            .await
            .map_err(|e| ConfigError::DependencyError(e.to_string()))?;

        if !output.status.success() {
            println!("Error during package installation:");
            return Err(ConfigError::DependencyError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        println!("Packages installed successfully!");
        println!("Generating biome.json configuration file...");

        let biome_config = BiomeProvider::configuration();

        fs::write("biome.json", serde_json::to_string_pretty(&biome_config)?)
            .map_err(|e| ConfigError::FileWriteError(e.to_string()))?;

        println!("Biome configuration file generated successfully!");
        println!("Biome configuration installed successfully!");

        Ok(())
    }

    async fn remove(&self) -> Result<(), ConfigError> {
        println!("Removing Biome configuration...");
        println!("Removing biome.json configuration file...");

        for file in &["biome.json"] {
            if Path::new(file).exists() {
                fs::remove_file(file).map_err(|e| ConfigError::FileWriteError(e.to_string()))?;
            }
        }

        let output = tokio::process::Command::new("bun")
            .args(&["remove", "@biomejs/biome", "ultracite"])
            .output()
            .await
            .map_err(|e| ConfigError::DependencyError(e.to_string()))?;

        if !output.status.success() {
            return Err(ConfigError::DependencyError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(())
    }
}
