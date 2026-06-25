use dialoguer::Input;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct SwitchboardConfig {
    pub port: u16,
    pub klondike_url: String,
    #[serde(skip_serializing)]
    pub slack_token: Option<String>,
    #[serde(skip_serializing)]
    pub github_token: Option<String>,
}

pub fn prompt_config() -> Result<SwitchboardConfig, String> {
    let port: u16 = Input::new()
        .with_prompt("Port")
        .default(4080)
        .interact_text()
        .map_err(|e| e.to_string())?;

    let klondike_url: String = Input::new()
        .with_prompt("Klondike URL")
        .default("http://localhost:3000".into())
        .interact_text()
        .map_err(|e| e.to_string())?;

    let slack_token: String = Input::new()
        .with_prompt("Slack token (leave empty to skip)")
        .default(String::new())
        .interact_text()
        .map_err(|e| e.to_string())?;

    let github_token: String = Input::new()
        .with_prompt("GitHub token (leave empty to skip)")
        .default(String::new())
        .interact_text()
        .map_err(|e| e.to_string())?;

    Ok(SwitchboardConfig {
        port,
        klondike_url,
        slack_token: if slack_token.is_empty() { None } else { Some(slack_token) },
        github_token: if github_token.is_empty() { None } else { Some(github_token) },
    })
}

pub fn write_config(cfg: &SwitchboardConfig) -> Result<(), String> {
    let content = toml::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    fs::write("switchboard.toml", content).map_err(|e| e.to_string())
}

pub fn write_secrets(cfg: &SwitchboardConfig) -> Result<(), String> {
    let dir = Path::new("secrets");
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;

    if let Some(token) = &cfg.slack_token {
        fs::write(dir.join("slack_token"), token).map_err(|e| e.to_string())?;
    }
    if let Some(token) = &cfg.github_token {
        fs::write(dir.join("github_token"), token).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn load_resolved_config() -> Result<SwitchboardConfig, String> {
    let settings = config::Config::builder()
        .add_source(config::File::with_name("switchboard").required(false))
        .add_source(
            config::Environment::default()
                .prefix("SWITCHBOARD")
                .separator("_")
                .try_parsing(true),
        )
        .add_source(
            config::Environment::default()
                .try_parsing(true),
        )
        .set_default("port", 4080)
        .map_err(|e| e.to_string())?
        .set_default("klondike_url", "http://localhost:3000")
        .map_err(|e| e.to_string())?
        .build()
        .map_err(|e| e.to_string())?;

    Ok(SwitchboardConfig {
        port: settings.get::<u16>("port").unwrap_or(4080),
        klondike_url: settings
            .get::<String>("klondike_url")
            .unwrap_or_else(|_| "http://localhost:3000".into()),
        slack_token: settings.get::<String>("slack_token").ok(),
        github_token: settings.get::<String>("github_token").ok(),
    })
}
