use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct SiteConfig {
    pub title: String,
    pub description: String,
    pub base_url: String,

    #[serde(default = "default_output_dir")]
    pub output_dir: String,

    #[serde(default = "default_content_dir")]
    pub content_dir: String,

    #[serde(default = "default_assets_dir")]
    pub assets_dir: String,

    #[serde(default = "default_templates_dir")]
    pub templates_dir: String,

    #[serde(default)]
    pub build: BuildConfig,
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            title: "My Site".to_string(),
            description: "A static site".to_string(),
            base_url: "http://localhost:8080".to_string(),
            output_dir: "public".to_string(),
            content_dir: "content".to_string(),
            assets_dir: "static".to_string(),
            templates_dir: "templates".to_string(),
            build: BuildConfig::default(),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct BuildConfig {
    #[serde(default = "default_true")]
    pub drafts: bool,
    #[serde(default = "default_true")]
    pub future: bool,
}

fn default_output_dir() -> String {
    "public".to_string()
}

fn default_content_dir() -> String {
    "content".to_string()
}

fn default_assets_dir() -> String {
    "assets".to_string()
}

fn default_templates_dir() -> String {
    "templates".to_string()
}

fn default_true() -> bool {
    true
}

pub fn load_config() -> anyhow::Result<SiteConfig> {
    let content = fs::read_to_string("config.toml")?;
    let config: SiteConfig = toml::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse config.toml: {}", e))?;
    Ok(config)
}
