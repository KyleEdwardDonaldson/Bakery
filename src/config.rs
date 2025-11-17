//! Configuration management for Bakery
//!
//! This module handles loading, validating, and managing configuration for Bakery.
//! Configuration is stored in TOML format and supports user customization for
//! Azure DevOps connections, storage locations, and AI integration settings.

use serde::{Deserialize, Serialize};
use anyhow::Result;

/// Main configuration structure for Bakery
///
/// Contains all configuration sections needed for Bakery operation:
/// - Azure DevOps connection settings
/// - Storage and file organization preferences
/// - OpenSpec and AI integration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BakeryConfig {
    /// Azure DevOps connection configuration
    pub azure_devops: AzureDevOpsConfig,
    /// Storage and file system configuration
    pub storage: StorageConfig,
    /// OpenSpec and AI integration configuration
    pub openspec: OpenSpecConfig,
}

/// Configuration for Azure DevOps API connection
///
/// Contains organization details, authentication tokens, and API version settings
/// required to connect to Azure DevOps and retrieve work items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureDevOpsConfig {
    /// Azure DevOps organization name (e.g., "myorg")
    pub organization: String,
    /// Azure DevOps project name (e.g., "MyProject")
    pub project: String,
    /// Personal Access Token for API authentication
    /// Should be treated as sensitive information
    pub pat_token: String,
    /// Azure DevOps REST API version (default: "7.1")
    pub api_version: String,
}

/// Configuration for storage and file organization
///
/// Controls where and how Bakery stores scraped work items and generated plans.
/// Supports both centralized and local storage modes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Base directory for all Bakery storage
    /// Can be an absolute path or relative path
    pub base_directory: String,
    /// Subdirectory name for storing scraped tickets
    pub tickets_subdir: String,
    /// Subdirectory name for storing OpenSpec plans
    pub openspec_subdir: String,

    /// Local baking mode - creates ticket and openspec folders in current working directory
    /// When enabled, Bakery will create folders in the directory where the command is run
    /// instead of using the base_directory. This is useful for per-project ticket organization.
    pub local_baking: bool,
}

/// Configuration for OpenSpec integration and AI plan generation
///
/// Controls how Bakery generates OpenSpec implementation plans using AI tools.
/// Supports multiple AI platforms and customizable command templates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenSpecConfig {
    /// Command template for AI plan generation
    /// Use {prompt} as placeholder for the generated prompt
    /// Example: "claude -p \"{prompt}\""
    pub ai_command_template: String,
    /// Whether to automatically generate OpenSpec plans after scraping
    /// Set to false to disable automatic plan generation
    pub auto_generate: bool,
    /// Enable rich output mode with maximum visual features by default
    /// Can be overridden with --rich, --compact, or --no-color flags
    #[serde(default = "default_rich_output")]
    pub rich_output: bool,
}

fn default_rich_output() -> bool {
    true
}

impl Default for BakeryConfig {
    fn default() -> Self {
        Self {
            azure_devops: AzureDevOpsConfig {
                organization: "your-organization".to_string(),
                project: "your-project".to_string(),
                pat_token: "your-pat-token-here".to_string(),
                api_version: "7.1".to_string(),
            },
            storage: StorageConfig {
                base_directory: if cfg!(windows) {
                    "C:/DevOpsData".to_string()
                } else {
                    "~/devops-data".to_string()
                },
                tickets_subdir: "Tickets".to_string(),
                openspec_subdir: "openspec".to_string(),
                local_baking: false,
            },
            openspec: OpenSpecConfig {
                ai_command_template: "claude -p \"{prompt}\"".to_string(),
                auto_generate: true,
                rich_output: true,
            },
        }
    }
}

impl BakeryConfig {
    pub fn get_config_dir() -> Result<String> {
        let home_dir = if cfg!(windows) {
            std::env::var("USERPROFILE").map_err(|_| anyhow::anyhow!("USERPROFILE environment variable not found"))?
        } else {
            std::env::var("HOME").map_err(|_| anyhow::anyhow!("HOME environment variable not found"))?
        };

        let bakery_dir = format!("{}\\.bakery", home_dir);
        Ok(bakery_dir)
    }

    pub fn get_config_path() -> Result<String> {
        let config_dir = Self::get_config_dir()?;
        Ok(format!("{}\\bakery-config.toml", config_dir))
    }

    pub fn get_example_config_path() -> String {
        if cfg!(windows) {
            format!("{}\\bakery-config.example.toml", env!("CARGO_MANIFEST_DIR"))
        } else {
            format!("{}/bakery-config.example.toml", env!("CARGO_MANIFEST_DIR"))
        }
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        let config_dir = Self::get_config_dir()?;

        // Ensure .bakery directory exists
        std::fs::create_dir_all(&config_dir)?;

        if std::path::Path::new(&config_path).exists() {
            let config_content = std::fs::read_to_string(&config_path)?;
            let config: BakeryConfig = toml::from_str(&config_content)?;
            Ok(config)
        } else {
            // Copy example config to user config directory
            let example_path = Self::get_example_config_path();
            if std::path::Path::new(&example_path).exists() {
                std::fs::copy(&example_path, &config_path)?;
            }

            // Create default config file if example doesn't exist
            if !std::path::Path::new(&config_path).exists() {
                let default_config = Self::default();
                let config_toml = toml::to_string_pretty(&default_config)?;
                std::fs::write(&config_path, config_toml)?;
            }

            // Load the created config
            let config_content = std::fs::read_to_string(&config_path)?;
            let config: BakeryConfig = toml::from_str(&config_content)?;
            Ok(config)
        }
    }

    pub fn get_tickets_directory(&self) -> String {
        format!("{}/{}", self.storage.base_directory, self.storage.tickets_subdir)
    }

    pub fn get_openspec_directory(&self) -> String {
        format!("{}/{}", self.storage.base_directory, self.storage.openspec_subdir)
    }

    pub fn get_ticket_directory(&self, ticket_id: u32) -> String {
        format!("{}/{}", self.get_tickets_directory(), ticket_id)
    }

    /// Gets the base directory to use for storage operations
    /// Returns the current working directory if local_baking is enabled, otherwise the configured base_directory
    pub fn get_effective_base_directory(&self) -> String {
        if self.storage.local_baking {
            match std::env::current_dir() {
                Ok(dir) => dir.to_string_lossy().to_string(),
                Err(_) => {
                    // Fallback to configured base_directory if we can't get current directory
                    eprintln!("⚠️ Warning: Could not determine current directory, using configured base_directory");
                    self.storage.base_directory.clone()
                }
            }
        } else {
            self.storage.base_directory.clone()
        }
    }

    /// Gets the effective tickets directory based on local_baking setting
    pub fn get_effective_tickets_directory(&self) -> String {
        let base_dir = self.get_effective_base_directory();
        format!("{}/{}", base_dir, self.storage.tickets_subdir)
    }

    /// Gets the effective openspec directory based on local_baking setting
    pub fn get_effective_openspec_directory(&self) -> String {
        let base_dir = self.get_effective_base_directory();
        format!("{}/{}", base_dir, self.storage.openspec_subdir)
    }
}