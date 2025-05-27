//! Rust RADIUS: Modern Authentication & Captive Portal Platform
//!
//! This is the main entry point for the rust-radius server.

use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use rust_radius::config::{Config, DeploymentTemplate};
use rust_radius::server::Server;
use rust_radius::Result;

/// Command line arguments
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Subcommand to run
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Path to configuration file
    #[arg(short, long, default_value = "config/radius.toml")]
    config: PathBuf,
}

/// Available subcommands
#[derive(Subcommand)]
enum Commands {
    /// Create a new configuration file from a template
    #[command(about = "Create a new configuration from a template")]
    Init {
        /// Template to use
        #[arg(short, long, default_value = "basic")]
        template: String,
        
        /// Output path for the configuration file
        #[arg(short, long, default_value = "config/radius.toml")]
        output: PathBuf,
        
        /// RADIUS shared secret
        #[arg(short, long)]
        secret: Option<String>,
    },
    
    /// Test the RADIUS server configuration
    #[command(about = "Test the RADIUS server configuration")]
    Test {
        /// Path to configuration file
        #[arg(short, long, default_value = "config/radius.toml")]
        config: PathBuf,
    },
    
    /// Start the RADIUS server
    #[command(about = "Start the RADIUS server")]
    Start {
        /// Path to configuration file
        #[arg(short, long, default_value = "config/radius.toml")]
        config: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // GOAL: Comprehensive Observability
    // Initialize tracing for structured logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    // Parse command line arguments
    let args = Args::parse();
    
    match args.command {
        Some(Commands::Init { template, output, secret }) => {
            // GOAL: Simplified Deployment and Configuration
            // Create a new configuration file from a template
            tracing::info!(template = template, output = ?output, "Creating new configuration");
            
            // Determine template type
            let template_type = match template.to_lowercase().as_str() {
                "basic" => DeploymentTemplate::Basic,
                "open" => DeploymentTemplate::OpenWithCaptivePortal,
                "enterprise" => DeploymentTemplate::Enterprise,
                "hotel" => DeploymentTemplate::HotelGuest,
                "cafe" => DeploymentTemplate::CafeGuest,
                "corporate" => DeploymentTemplate::CorporateGuest,
                _ => {
                    tracing::error!(template = template, "Unknown template");
                    return Err(format!("Unknown template: {}", template).into());
                }
            };
            
            // Get secret
            let secret = match secret {
                Some(s) => s,
                None => {
                    // Generate a random secret
                    use rand::{thread_rng, Rng};
                    let mut rng = thread_rng();
                    let secret: String = (0..32)
                        .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
                        .collect();
                    secret
                }
            };
            
            // Create configuration
            let config = Config::from_template(template_type, secret);
            
            // Ensure parent directory exists
            if let Some(parent) = output.parent() {
                std::fs::create_dir_all(parent)?;
            }
            
            // Write configuration file
            config.export(&output)?;
            
            tracing::info!(path = ?output, "Configuration created");
        },
        Some(Commands::Test { config }) => {
            // GOAL: Simplified Deployment and Configuration
            // Test the RADIUS server configuration
            tracing::info!(config = ?config, "Testing configuration");
            
            // Load configuration
            let config = Config::from_file(&config)?;
            
            // Validate configuration
            // In a real implementation, we would perform more thorough validation
            tracing::info!("Configuration is valid");
        },
        Some(Commands::Start { config }) => {
            // GOAL: High-Performance and Concurrency
            // Start the RADIUS server
            tracing::info!(config = ?config, "Starting RADIUS server");
            
            // Load configuration
            let config = Config::from_file(&config)?;
            
            // Create server
            let server = Server::new(config).await?;
            
            // Run server
            server.run().await?;
        },
        None => {
            // Default to starting the server
            tracing::info!(config = ?args.config, "Starting RADIUS server");
            
            // Load configuration
            let config = Config::from_file(&args.config)?;
            
            // Create server
            let server = Server::new(config).await?;
            
            // Run server
            server.run().await?;
        }
    }
    
    Ok(())
}
