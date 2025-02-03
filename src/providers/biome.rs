use async_trait::async_trait;
use std::path::PathBuf;
use which::which;

use crate::configurator::AmarisConfigurator;
use crate::error::ConfigError;
use crate::registry::AmarisProvider;

pub struct BiomeProvider;

impl BiomeProvider {
    pub fn get_configuration_path() -> PathBuf {
        PathBuf::from("biome.json")
    }

    pub fn get_configuration() -> serde_json::Value {
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

    pub fn get_vscode_configuration() -> serde_json::Value {
        serde_json::json!({
           "typescript.tsdk": "node_modules/typescript/lib",
            "typescript.enablePromptUseWorkspaceTsdk": true,
            "editor.defaultFormatter": "biomejs.biome",
            "editor.codeActionsOnSave": {
                "quickfix.biome": "explicit",
                "source.organizeImports.biome": "explicit"
            },
            "files.exclude": {
                "**/node_modules": true
            }
        })
    }

    pub fn get_packages() -> Vec<&'static str> {
        vec!["@biomejs/biome", "ultracite"]
    }

    pub async fn install_packages(&self) -> Result<(), ConfigError> {
        let packages = BiomeProvider::get_packages();

        for package in packages {
            AmarisConfigurator::run_command("bun", &["install", "--dev", package]).await?;
        }

        Ok(())
    }

    pub async fn remove_packages(&self) -> Result<(), ConfigError> {
        let packages = BiomeProvider::get_packages();

        for package in packages {
            AmarisConfigurator::run_command("bun", &["remove", "--dev", package]).await?;
        }

        Ok(())
    }

    pub async fn write_configuration(&self) -> Result<(), ConfigError> {
        AmarisConfigurator::write_file(
            BiomeProvider::get_configuration_path(),
            &serde_json::to_string_pretty(&BiomeProvider::get_configuration())?,
        )
        .await?;

        Ok(())
    }

    pub async fn remove_configuration(&self) -> Result<(), ConfigError> {
        AmarisConfigurator::remove_file(BiomeProvider::get_configuration_path()).await?;

        Ok(())
    }

    pub async fn update_vscode_settings() -> Result<(), ConfigError> {
        let settings = BiomeProvider::get_vscode_configuration();
        let workspace_settings = AmarisConfigurator::read_vscode_settings().await?;

        let mut updated_settings = workspace_settings.clone();

        for (key, value) in settings.as_object().unwrap() {
            updated_settings[key] = value.clone();
        }

        AmarisConfigurator::write_vscode_settings(&updated_settings).await?;

        Ok(())
    }

    pub async fn remove_vscode_settings() -> Result<(), ConfigError> {
        let workspace_settings = AmarisConfigurator::read_vscode_settings().await?;
        let settings = BiomeProvider::get_vscode_configuration();

        let mut updated_settings = workspace_settings.clone();

        for (key, _) in settings.as_object().unwrap() {
            updated_settings.as_object_mut().unwrap().remove(key);
        }

        AmarisConfigurator::write_vscode_settings(&updated_settings).await?;

        Ok(())
    }

    pub async fn update_package_json() -> Result<(), ConfigError> {
        let package_json = AmarisConfigurator::read_package_json().await?;

        let mut updated_package_json = package_json.clone();

        updated_package_json["scripts"]["biome"] = serde_json::json!("biome check .");
        updated_package_json["scripts"]["biome:fix"] = serde_json::json!("biome check . --write");

        AmarisConfigurator::write_package_json(&updated_package_json).await?;

        Ok(())
    }

    pub async fn remove_package_json() -> Result<(), ConfigError> {
        let package_json = AmarisConfigurator::read_package_json().await?;

        let mut updated_package_json = package_json.clone();

        updated_package_json["scripts"]
            .as_object_mut()
            .unwrap()
            .remove("biome");
        updated_package_json["scripts"]
            .as_object_mut()
            .unwrap()
            .remove("biome:fix");

        AmarisConfigurator::write_package_json(&updated_package_json).await?;

        Ok(())
    }
}

#[async_trait]
impl AmarisProvider for BiomeProvider {
    fn name(&self) -> &'static str {
        "biome"
    }

    fn description(&self) -> &'static str {
        "Biome"
    }

    async fn check_prerequisites(&self) -> Result<(), ConfigError> {
        which("bun").map_err(|_| {
            ConfigError::MissingPrerequisite("bun is required but not found".to_string())
        })?;

        if !AmarisConfigurator::get_package_json_path().exists() {
            return Err(ConfigError::MissingPrerequisite(
                "package.json not found!".to_string(),
            ));
        }

        Ok(())
    }

    async fn install(&self) -> Result<(), ConfigError> {
        println!("Installing Biome packages...");
        BiomeProvider::install_packages(&self).await?;

        println!("Writing Biome configuration...");
        BiomeProvider::write_configuration(&self).await?;

        println!("Updating VS Code settings...");
        BiomeProvider::update_vscode_settings().await?;

        println!("Updating package.json...");
        BiomeProvider::update_package_json().await?;

        println!("Biome installed successfully!");

        Ok(())
    }

    async fn remove(&self) -> Result<(), ConfigError> {
        println!("Removing Biome packages...");
        BiomeProvider::remove_packages(&self).await?;

        println!("Removing Biome configuration...");
        BiomeProvider::remove_configuration(&self).await?;

        println!("Removing VS Code settings...");
        BiomeProvider::remove_vscode_settings().await?;

        println!("Removing package.json scripts...");
        BiomeProvider::remove_package_json().await?;

        Ok(())
    }
}
