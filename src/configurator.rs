use std::path::PathBuf;

use serde_json::Value;
use tokio::{
    fs::{File, create_dir_all},
    io::AsyncReadExt,
};

use crate::error::ConfigError;

fn merge_json_values(target: &mut Value, source: &Value) {
    match (target, source) {
        (Value::Object(target_map), Value::Object(source_map)) => {
            for (key, source_value) in source_map {
                match target_map.get_mut(key) {
                    Some(target_value) => {
                        // Recursively merge if both are objects
                        if target_value.is_object() && source_value.is_object() {
                            merge_json_values(target_value, source_value);
                        } else if target_value != source_value {
                            // Only update if values are different
                            *target_value = source_value.clone();
                        }
                    }
                    None => {
                        // Insert new key-value pair
                        target_map.insert(key.clone(), source_value.clone());
                    }
                }
            }
        }
        (target, source) => *target = source.clone(),
    }
}

pub struct AmarisConfigurator;

impl AmarisConfigurator {
    pub fn get_vscode_settings_path() -> PathBuf {
        PathBuf::from(".vscode/settings.json")
    }

    pub async fn read_vscode_settings() -> Result<Value, ConfigError> {
        let settings_path = AmarisConfigurator::get_vscode_settings_path();

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

    pub async fn write_vscode_settings(settings: &Value) -> Result<(), ConfigError> {
        let settings_path = AmarisConfigurator::get_vscode_settings_path();

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

    pub async fn update_vscode_settings(
        update: impl FnOnce(&mut Value),
    ) -> Result<(), ConfigError> {
        let mut settings = AmarisConfigurator::read_vscode_settings().await?;

        let mut original = settings.clone();

        update(&mut settings);
        merge_json_values(&mut original, &settings);

        AmarisConfigurator::write_vscode_settings(&original).await
    }

    pub fn get_package_json_path() -> PathBuf {
        PathBuf::from("package.json")
    }

    pub async fn read_package_json() -> Result<Value, ConfigError> {
        let package_json_path = AmarisConfigurator::get_package_json_path();

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

    pub async fn write_package_json(package_json: &Value) -> Result<(), ConfigError> {
        let package_json_path = AmarisConfigurator::get_package_json_path();

        tokio::fs::write(
            package_json_path,
            serde_json::to_string_pretty(package_json).unwrap(),
        )
        .await
        .map_err(|e| ConfigError::FileWriteError(e.to_string()))?;

        Ok(())
    }

    pub async fn update_package_json(update: impl FnOnce(&mut Value)) -> Result<(), ConfigError> {
        let mut package_json = AmarisConfigurator::read_package_json().await?;

        let mut original = package_json.clone();

        update(&mut package_json);
        merge_json_values(&mut original, &package_json);

        AmarisConfigurator::write_package_json(&original).await
    }

    pub async fn add_package_script(
        name: &str,
        content: &str,
        append: bool,
    ) -> Result<(), ConfigError> {
        Self::update_package_json(|package_json| {
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

    pub async fn remove_package_script(name: &str) -> Result<(), ConfigError> {
        Self::update_package_json(|package_json| {
            if let Some(scripts) = package_json.get_mut("scripts") {
                if let Some(obj) = scripts.as_object_mut() {
                    obj.remove(name);
                }
            }
        })
        .await
    }

    pub async fn get_package_script(name: &str) -> Result<Option<String>, ConfigError> {
        let package_json = Self::read_package_json().await?;

        Ok(package_json
            .get("scripts")
            .and_then(|scripts| scripts.get(name))
            .and_then(|script| script.as_str())
            .map(String::from))
    }

    pub async fn get_package_dependencies() -> Result<Value, ConfigError> {
        let package_json = Self::read_package_json().await?;

        let mut all_deps = Vec::new();

        if let Some(deps) = package_json.get("dependencies") {
            if let Some(obj) = deps.as_object() {
                all_deps.extend(obj.keys().cloned());
            }
        }

        if let Some(dev_deps) = package_json.get("devDependencies") {
            if let Some(obj) = dev_deps.as_object() {
                all_deps.extend(obj.keys().cloned());
            }
        }

        if let Some(peer_deps) = package_json.get("peerDependencies") {
            if let Some(obj) = peer_deps.as_object() {
                all_deps.extend(obj.keys().cloned());
            }
        }

        Ok(Value::Array(
            all_deps.into_iter().map(Value::String).collect(),
        ))
    }

    pub async fn check_if_dependency_exists(names: &[&str]) -> Result<bool, ConfigError> {
        let all_deps = Self::get_package_dependencies().await?;

        Ok(names.iter().all(|name| {
            all_deps
                .as_array()
                .unwrap()
                .contains(&Value::String(name.to_string()))
        }))
    }

    pub async fn run_command(cmd: &str, args: &[&str]) -> Result<(), ConfigError> {
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

    pub async fn write_file(path: PathBuf, content: &str) -> Result<(), ConfigError> {
        if path.exists() {
            return Err(ConfigError::AlreadyExists(
                path.to_string_lossy().to_string(),
            ));
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
}
