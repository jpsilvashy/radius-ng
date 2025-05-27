// rust-radius: A high-performance RADIUS server implementation in Rust
// with modern features for public WiFi networks, authentication, and captive portals.

// This file serves as the main library entry point, exposing the core components
// and public API for the rust-radius crate.

// === Re-exports for public API ===
pub mod auth;
pub mod config;
pub mod captive_portal;
pub mod metrics;
pub mod plugins;
pub mod protocol;
pub mod radsec;
pub mod server;
pub mod utils;

use std::error::Error;

/// Library version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library result type
pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

/// Initialize the RADIUS server with the provided configuration
///
/// # Examples
///
/// ```no_run
/// use rust_radius::config::Config;
/// use rust_radius::server::Server;
///
/// #[tokio::main]
/// async fn main() -> rust_radius::Result<()> {
///     // Load configuration
///     let config = Config::from_file("config/radius.toml")?;
///     
///     // Initialize the server
///     let server = Server::new(config).await?;
///     
///     // Run the server
///     server.run().await
/// }
/// ```
pub fn init() -> Result<()> {
    // This is a placeholder function that will be implemented later
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
