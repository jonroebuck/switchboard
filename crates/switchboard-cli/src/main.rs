mod config_file;

use clap::{Parser, Subcommand};
use std::process::{Command, ExitCode};

#[derive(Parser)]
#[command(name = "switchboard", about = "Switchboard CLI")]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Setup and start Switchboard
    Up,
    /// Start containers (docker compose up -d)
    Start,
    /// Stop containers (docker compose down)
    Stop,
    /// Show running containers and config
    Status,
}

fn is_interactive() -> bool {
    atty::is(atty::Stream::Stdin) && std::env::var("CI").is_err()
}

fn run_docker_compose(args: &[&str]) -> Result<(), String> {
    let status = Command::new("docker")
        .arg("compose")
        .args(args)
        .status()
        .map_err(|e| format!("failed to run docker compose: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("docker compose exited with {status}"))
    }
}

fn cmd_up() -> Result<(), String> {
    if is_interactive() {
        let cfg = config_file::prompt_config()?;
        config_file::write_config(&cfg)?;
        config_file::write_secrets(&cfg)?;
        println!("Config written to switchboard.toml");
        println!("Secrets written to secrets/");
    } else {
        println!("CI/non-interactive mode — reading config from environment");
    }

    let resolved = config_file::load_resolved_config()?;
    let port_str = resolved.port.to_string();

    let mut env_pairs: Vec<(String, String)> = vec![
        ("SWITCHBOARD_PORT".into(), port_str),
        ("KLONDIKE_URL".into(), resolved.klondike_url),
    ];
    if let Some(t) = &resolved.slack_token {
        env_pairs.push(("SLACK_TOKEN".into(), t.clone()));
    }
    if let Some(t) = &resolved.github_token {
        env_pairs.push(("GITHUB_TOKEN".into(), t.clone()));
    }

    let status = Command::new("docker")
        .arg("compose")
        .args(["up", "-d"])
        .envs(env_pairs)
        .status()
        .map_err(|e| format!("failed to run docker compose: {e}"))?;

    if status.success() {
        println!("Switchboard is running");
        Ok(())
    } else {
        Err(format!("docker compose exited with {status}"))
    }
}

fn cmd_status() -> Result<(), String> {
    match config_file::load_resolved_config() {
        Ok(cfg) => {
            println!("Port:         {}", cfg.port);
            println!("Klondike URL: {}", cfg.klondike_url);
            println!(
                "Slack token:  {}",
                if cfg.slack_token.is_some() { "set" } else { "not set" }
            );
            println!(
                "GitHub token: {}",
                if cfg.github_token.is_some() { "set" } else { "not set" }
            );
            println!();
        }
        Err(e) => {
            println!("Could not load config: {e}");
            println!();
        }
    }
    run_docker_compose(&["ps"])
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Cmd::Up => cmd_up(),
        Cmd::Start => run_docker_compose(&["up", "-d"]),
        Cmd::Stop => run_docker_compose(&["down"]),
        Cmd::Status => cmd_status(),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
