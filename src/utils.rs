use std::path::PathBuf;

use serde_json::Value;
use tokio::{
    fs::{File, create_dir_all},
    io::AsyncReadExt,
};

use crate::{
    error::ConfigError,
    provider::{ConfigEntry, DynamicProvider, ScriptEntry},
};

pub const PROVIDER_DIR_NAME: &str = "providers";
pub const CONFIG_DIR_NAME: &str = "configs";
pub const APP_CONFIG_DIR: &str = ".amaya";

fn merge_json_values(target: &mut Value, source: &Value) {
    match (target, source) {
        (Value::Object(target_map), Value::Object(source_map)) => {
            for (key, source_value) in source_map {
                match target_map.get_mut(key) {
                    Some(target_value) => {
                        if target_value.is_object() && source_value.is_object() {
                            merge_json_values(target_value, source_value);
                        } else if target_value != source_value {
                            *target_value = source_value.clone();
                        }
                    }
                    None => {
                        target_map.insert(key.clone(), source_value.clone());
                    }
                }
            }
        }
        (target, source) => *target = source.clone(),
    }
}

pub struct AmarisPathHandler;

impl AmarisPathHandler {
    fn get_root_config_path() -> Result<PathBuf, ConfigError> {
        let home = dirs::home_dir()
            .ok_or_else(|| ConfigError::PathError("Could not find home directory".into()))?;

        Ok(home.join(APP_CONFIG_DIR))
    }

    fn get_default_provider_path() -> Result<PathBuf, ConfigError> {
        let home = dirs::home_dir()
            .ok_or_else(|| ConfigError::PathError("Could not find home directory".into()))?;

        Ok(home.join(APP_CONFIG_DIR).join(PROVIDER_DIR_NAME))
    }

    pub async fn ensure_provider_dir() -> Result<PathBuf, ConfigError> {
        let provider_path = Self::get_default_provider_path()?;

        if !provider_path.exists() {
            tokio::fs::create_dir_all(&provider_path).await?;
        }

        Ok(provider_path)
    }

    fn get_default_config_path() -> Result<PathBuf, ConfigError> {
        let home = dirs::home_dir()
            .ok_or_else(|| ConfigError::PathError("Could not find home directory".into()))?;

        Ok(home.join(APP_CONFIG_DIR).join(CONFIG_DIR_NAME))
    }

    pub async fn ensure_config_dir() -> Result<PathBuf, ConfigError> {
        let config_path = Self::get_default_config_path()?;

        if !config_path.exists() {
            tokio::fs::create_dir_all(&config_path).await?;
        }

        Ok(config_path)
    }
}

pub struct AmarisFileHandler;

impl AmarisFileHandler {
    pub async fn write_file(path: PathBuf, content: &str) -> Result<(), ConfigError> {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .map_err(|e| ConfigError::FileWriteError(e.to_string()))?;
            }
        }

        tokio::fs::write(&path, content)
            .await
            .map_err(|e| ConfigError::FileWriteError(e.to_string()))?;

        Ok(())
    }

    pub async fn remove_file(path: PathBuf) -> Result<(), ConfigError> {
        if !path.exists() {
            return Ok(());
        }

        tokio::fs::remove_file(&path)
            .await
            .map_err(|e| ConfigError::FileWriteError(e.to_string()))?;

        Ok(())
    }

    pub async fn load_file(path: &PathBuf) -> Result<String, ConfigError> {
        tokio::fs::read_to_string(&path)
            .await
            .map_err(|e| ConfigError::FileReadError(e.to_string()))
    }
}

pub struct AmarisConfigurationHandler;

impl AmarisConfigurationHandler {
    pub async fn write_configs(
        name: String,
        configs: &Vec<ConfigEntry>,
    ) -> Result<(), ConfigError> {
        for config in configs {
            let source_path: PathBuf = AmarisPathHandler::get_default_config_path()?
                .join(&name)
                .join(&config.source_from);

            let content: String = AmarisFileHandler::load_file(&source_path).await?;
            let path: PathBuf = PathBuf::from(&config.file_location);

            AmarisFileHandler::write_file(path, &content).await?;
        }

        Ok(())
    }

    pub async fn remove_configs(configs: &Vec<ConfigEntry>) -> Result<(), ConfigError> {
        for config in configs {
            let path: PathBuf = PathBuf::from(&config.file_name);

            if path.to_str().unwrap() == "settings.json".to_string() {
                AmarisVisualStudioCodeHandler::write(&serde_json::json!({})).await?;
            } else {
                AmarisFileHandler::remove_file(path).await?;
            }
        }

        Ok(())
    }
}

pub struct AmarisInstaller;

impl AmarisInstaller {
    pub async fn install(
        manager: &str,
        packages: &Vec<std::string::String>,
    ) -> Result<(), ConfigError> {
        for package in packages {
            Self::run_command(manager, &["install", "--dev", package]).await?;
        }

        Ok(())
    }

    pub async fn remove(
        manager: &str,
        packages: &Vec<std::string::String>,
    ) -> Result<(), ConfigError> {
        for package in packages {
            Self::run_command(manager, &["remove", package]).await?;
        }

        Ok(())
    }

    async fn run_command(cmd: &str, args: &[&str]) -> Result<(), ConfigError> {
        let output = tokio::process::Command::new(cmd)
            .args(args)
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

    pub async fn write(settings: &Value) -> Result<(), ConfigError> {
        let settings_path = AmarisVisualStudioCodeHandler::get_default_path();

        create_dir_all(settings_path.parent().unwrap())
            .await
            .map_err(|e| ConfigError::FileWriteError(e.to_string()))?;

        tokio::fs::write(
            settings_path,
            serde_json::to_string_pretty(settings).unwrap(),
        )
        .await
        .map_err(|e| ConfigError::FileWriteError(e.to_string()))?;

        Ok(())
    }

    pub async fn update(update: impl FnOnce(&mut Value)) -> Result<(), ConfigError> {
        let mut settings = AmarisVisualStudioCodeHandler::read().await?;

        let mut original = settings.clone();

        update(&mut settings);
        merge_json_values(&mut original, &settings);

        AmarisVisualStudioCodeHandler::write(&original).await
    }
}

pub struct AmarisPackageJsonHandler;

impl AmarisPackageJsonHandler {
    pub fn get_default_path() -> PathBuf {
        PathBuf::from("package.json")
    }

    pub async fn read() -> Result<Value, ConfigError> {
        let package_json_path = AmarisPackageJsonHandler::get_default_path();

        if !package_json_path.exists() {
            return Ok(serde_json::json!({}));
        }

        let mut file: File = tokio::fs::File::open(package_json_path)
            .await
            .map_err(|e| ConfigError::FileWriteError(e.to_string()))?;

        let mut contents = String::new();

        let _ = file
            .read_to_string(&mut contents)
            .await
            .map_err(|e| ConfigError::FileWriteError(e.to_string()));

        serde_json::from_str(&contents).map_err(|e| ConfigError::ValidationError(e.to_string()))
    }

    pub async fn write(package_json: &Value) -> Result<(), ConfigError> {
        let package_json_path = AmarisPackageJsonHandler::get_default_path();

        tokio::fs::write(
            package_json_path,
            serde_json::to_string_pretty(package_json).unwrap(),
        )
        .await
        .map_err(|e| ConfigError::FileWriteError(e.to_string()))?;

        Ok(())
    }

    pub async fn update(update: impl FnOnce(&mut Value)) -> Result<(), ConfigError> {
        let mut package_json = AmarisPackageJsonHandler::read().await?;

        let mut original = package_json.clone();

        update(&mut package_json);
        merge_json_values(&mut original, &package_json);

        AmarisPackageJsonHandler::write(&original).await
    }

    pub async fn add_script(name: &str, content: &str, append: bool) -> Result<(), ConfigError> {
        AmarisPackageJsonHandler::update(|package_json| {
            // Ensure scripts object exists
            if !package_json.get("scripts").is_some() {
                package_json["scripts"] = serde_json::json!({});
            }

            let scripts = package_json["scripts"].as_object_mut().unwrap();

            match scripts.get(name) {
                Some(existing) if append => {
                    // Append to existing script
                    let existing_content = existing.as_str().unwrap_or_default();
                    let new_content = format!("{} && {}", existing_content, content);
                    scripts[name] = serde_json::json!(new_content);
                }
                Some(_) if !append => {
                    // Overwrite existing script
                    scripts[name] = serde_json::json!(content);
                }
                None => {
                    // Add new script
                    scripts[name] = serde_json::json!(content);
                }
                _ => {}
            }
        })
        .await
    }

    pub async fn remove_script(name: &str) -> Result<(), ConfigError> {
        AmarisPackageJsonHandler::update(|package_json| {
            if let Some(scripts) = package_json.get_mut("scripts") {
                if let Some(obj) = scripts.as_object_mut() {
                    obj.remove(name);
                }
            }
        })
        .await
    }

    pub async fn get_script(name: &str) -> Result<Option<String>, ConfigError> {
        let package_json = AmarisPackageJsonHandler::read().await?;

        Ok(package_json
            .get("scripts")
            .and_then(|scripts| scripts.get(name))
            .and_then(|script| script.as_str())
            .map(String::from))
    }

    pub async fn write_scripts(scripts: &Vec<ScriptEntry>) -> Result<(), ConfigError> {
        let package_json = AmarisPackageJsonHandler::read().await?;

        let mut updated_package_json = package_json.clone();

        for script in scripts {
            updated_package_json["scripts"][&script.name] = serde_json::json!(script.script);
        }

        AmarisPackageJsonHandler::write(&updated_package_json).await?;

        Ok(())
    }

    pub async fn remove_scripts(scripts: &Vec<ScriptEntry>) -> Result<(), ConfigError> {
        let package_json = AmarisPackageJsonHandler::read().await?;

        let mut updated_package_json = package_json.clone();

        for script in scripts {
            updated_package_json["scripts"]
                .as_object_mut()
                .unwrap()
                .remove(&script.name);
        }

        AmarisPackageJsonHandler::write(&updated_package_json).await?;

        Ok(())
    }
}

pub struct AmarisInitialConfigHandler;

impl AmarisInitialConfigHandler {
    pub async fn ensure_dirs() -> Result<(), ConfigError> {
        let config_dir = AmarisPathHandler::get_default_config_path()?;
        let provider_dir = AmarisPathHandler::get_default_provider_path()?;
        let root = AmarisPathHandler::get_root_config_path()?;

        if !config_dir.exists() {
            tokio::fs::create_dir_all(&config_dir).await?;
        }

        if !provider_dir.exists() {
            tokio::fs::create_dir_all(&provider_dir).await?;
        }

        println!("Configuration home directory created at {:?}", root);
        println!("Configuration directory created at {:?}", config_dir);
        println!("Provider directory created at {:?}", provider_dir);

        println!("Start by adding a configuration provider to the provider directory");
        println!("Then add a configuration to the configuration directory");

        Ok(())
    }

    pub async fn create_initial_config() -> Result<(), ConfigError> {
        let config_dir = AmarisPathHandler::get_default_config_path()?;
        let provider_dir = AmarisPathHandler::get_default_provider_path()?;

        let biome_provider = DynamicProvider {
            name: "biome".to_string(),
            description: "Biome".to_string(),
            package_manager: "bun".to_string(),
            packages: vec!["@biomejs/biome".to_string()],
            configuration: vec![
                ConfigEntry {
                    file_location: "biome.json".to_string(),
                    file_name: "biome.json".to_string(),
                    source_from: "biome.json".to_string(),
                },
                ConfigEntry {
                    file_location: ".vscode/settings.json".to_string(),
                    file_name: "settings.json".to_string(),
                    source_from: "settings.json".to_string(),
                },
            ],
            scripts: vec![
                ScriptEntry {
                    name: "format".to_string(),
                    script: "biome format .".to_string(),
                },
                ScriptEntry {
                    name: "lint".to_string(),
                    script: "biome lint .".to_string(),
                },
            ],
        };
        let biome_config_from_provider = serde_json::to_string_pretty(&biome_provider).unwrap();

        let biome_config = serde_json::json!({
            "$schema": "https://biomejs.dev/schemas/1.9.4/schema.json",
            "extends": [
                "ultracite"
            ],
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
                "ignore": [
                    "node_modules"
                ]
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
                "globals": [
                    "Bun"
                ]
            },
            "json": {
                "formatter": {
                    "indentWidth": 4,
                    "indentStyle": "space"
                }
            }
        });

        let vscode_settings = serde_json::json!({
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
        });

        let biome_config_path = config_dir.join("biome").join("biome.json");
        let vscode_settings_path = config_dir.join("biome").join("settings.json");
        let biome_provider_path = provider_dir.join("biome.json");

        println!("Creating initial configuration files");

        AmarisFileHandler::write_file(biome_config_path, &biome_config.to_string()).await?;
        AmarisFileHandler::write_file(biome_provider_path, &biome_config_from_provider).await?;
        AmarisFileHandler::write_file(vscode_settings_path, &vscode_settings.to_string()).await?;

        Ok(())
    }
}
