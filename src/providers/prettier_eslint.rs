use async_trait::async_trait;
use std::path::PathBuf;
use which::which;

use crate::configurator::AmarisConfigurator;
use crate::error::ConfigError;
use crate::registry::AmarisProvider;

use super::biome::BiomeProvider;

pub struct PrettierEslintProvider;

impl PrettierEslintProvider {
    pub fn get_prettier_configuration_path() -> PathBuf {
        PathBuf::from(".prettierrc.json")
    }

    pub fn get_prettier_configuration() -> serde_json::Value {
        serde_json::json!({
            "semi": true,
            "trailingComma": "es5",
            "tabWidth": 4,
            "bracketSpacing": true,
            "singleQuote": false,
            "arrowParens": "always",
            "quoteProps": "consistent",
            "printWidth": 120,
            "plugins": ["@ianvs/prettier-plugin-sort-imports"],
            "importOrder": [
                "",
                "^react$",
                "^next(-[^/]+)?(/.*)?$",
                "",
                "<TYPES>",
                "<TYPES>^[.]",
                "",
                "<BUILTIN_MODULES>",
                "",
                "<THIRD_PARTY_MODULES>",
                "",
                "^#/(.*)$",
                "",
                "^[./]",
                "",
                "^(?!.*[.]css$)[./].*$",
                ".css$"
            ],
            "importOrderTypeScriptVersion": "5.4.5",
            "overrides": [
                {
                    "files": ["**/.vscode/*.json", "**/tsconfig.json", "**/tsconfig.*.json"],
                    "options": {
                        "parser": "jsonc"
                    }
                }
            ]
        })
    }

    pub fn get_vscode_configuration() -> serde_json::Value {
        serde_json::json!({
            "typescript.tsdk": "node_modules/typescript/lib",
            "typescript.enablePromptUseWorkspaceTsdk": true,
            "editor.defaultFormatter": "esbenp.prettier-vscode",
            "editor.codeActionsOnSave": {
                "source.fixAll.eslint": "explicit",
            },
            "eslint.rules.customizations": [
                {
                    "rule": "*",
                    "severity": "warn",
                },
            ],
            "files.exclude": {
                "**/node_modules": true
            },
        })
    }

    pub fn get_packages() -> Vec<&'static str> {
        vec!["prettier", "@ianvs/prettier-plugin-sort-imports"]
    }

    pub async fn install_packages(&self) -> Result<(), ConfigError> {
        let packages = PrettierEslintProvider::get_packages();

        for package in packages {
            AmarisConfigurator::run_command("bun", &["install", "--dev", package]).await?;
        }

        Ok(())
    }

    pub async fn remove_packages(&self) -> Result<(), ConfigError> {
        let packages = PrettierEslintProvider::get_packages();

        for package in packages {
            AmarisConfigurator::run_command("bun", &["remove", "--dev", package]).await?;
        }

        Ok(())
    }

    pub async fn write_configuration(&self) -> Result<(), ConfigError> {
        AmarisConfigurator::write_file(
            PrettierEslintProvider::get_prettier_configuration_path(),
            &serde_json::to_string_pretty(&PrettierEslintProvider::get_prettier_configuration())?,
        )
        .await?;

        Ok(())
    }

    pub async fn remove_configuration(&self) -> Result<(), ConfigError> {
        AmarisConfigurator::remove_file(PrettierEslintProvider::get_prettier_configuration_path())
            .await?;

        Ok(())
    }

    pub async fn update_vscode_settings() -> Result<(), ConfigError> {
        let settings = PrettierEslintProvider::get_vscode_configuration();
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
        let settings = PrettierEslintProvider::get_vscode_configuration();

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

        updated_package_json["scripts"]["format"] = serde_json::json!("prettier --write .");
        updated_package_json["scripts"]["format:check"] = serde_json::json!("prettier --check .");

        AmarisConfigurator::write_package_json(&updated_package_json).await?;

        Ok(())
    }

    pub async fn remove_package_json() -> Result<(), ConfigError> {
        let package_json = AmarisConfigurator::read_package_json().await?;

        let mut updated_package_json = package_json.clone();

        updated_package_json["scripts"]
            .as_object_mut()
            .unwrap()
            .remove("format");
        updated_package_json["scripts"]
            .as_object_mut()
            .unwrap()
            .remove("format:check");

        AmarisConfigurator::write_package_json(&updated_package_json).await?;

        Ok(())
    }
}

#[async_trait]
impl AmarisProvider for PrettierEslintProvider {
    fn name(&self) -> &'static str {
        "prettier_eslint"
    }

    fn description(&self) -> &'static str {
        "Prettier + ESLint"
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

        if AmarisConfigurator::check_if_dependency_exists(&BiomeProvider::get_packages()).await? {
            return Err(ConfigError::ConflictError(
                "Biome is already installed in this project and cannot be used with Prettier + ESLint".to_string(),
            ));
        }

        Ok(())
    }

    async fn install(&self) -> Result<(), ConfigError> {
        println!("Installing Prettier + ESLint packages...");
        PrettierEslintProvider::install_packages(&self).await?;

        println!("Writing Prettier + ESLint configuration...");
        PrettierEslintProvider::write_configuration(&self).await?;

        println!("Updating VS Code settings...");
        PrettierEslintProvider::update_vscode_settings().await?;

        println!("Updating package.json...");
        PrettierEslintProvider::update_package_json().await?;

        println!("Prettier + ESLint installed successfully!");

        Ok(())
    }

    async fn remove(&self) -> Result<(), ConfigError> {
        println!("Removing Prettier + ESLint packages...");
        PrettierEslintProvider::remove_packages(&self).await?;

        println!("Removing Prettier + ESLint configuration...");
        PrettierEslintProvider::remove_configuration(&self).await?;

        println!("Removing VS Code settings...");
        PrettierEslintProvider::remove_vscode_settings().await?;

        println!("Removing package.json scripts...");
        PrettierEslintProvider::remove_package_json().await?;

        println!("Prettier + ESLint removed successfully!");

        Ok(())
    }
}
