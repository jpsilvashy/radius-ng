// config.rs - Configuration management for rust-radius
//
// This module handles loading, parsing, and validating configuration for the RADIUS server.
// It implements the "Simplified Deployment and Configuration" goal by providing sensible
// defaults and deployment templates for common scenarios.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use toml;

use crate::Result;

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Host to bind to (default: 0.0.0.0)
    #[serde(default = "default_host")]
    pub host: String,
    
    /// Authentication port (default: 1812)
    #[serde(default = "default_auth_port")]
    pub auth_port: u16,
    
    /// Accounting port (default: 1813)
    #[serde(default = "default_acct_port")]
    pub acct_port: u16,
    
    /// RADIUS shared secret
    pub secret: String,
    
    /// Number of worker threads (default: number of CPU cores)
    pub worker_threads: Option<usize>,
    
    /// Graceful shutdown timeout in seconds (default: 30)
    #[serde(default = "default_shutdown_timeout")]
    pub shutdown_timeout_secs: u64,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Supported authentication protocols
    #[serde(default = "default_auth_protocols")]
    pub auth_protocols: Vec<String>,
    
    /// Maximum request size in bytes (default: 4096)
    #[serde(default = "default_max_request_size")]
    pub max_request_size: usize,
    
    /// Request timeout in milliseconds (default: 5000)
    #[serde(default = "default_request_timeout")]
    pub request_timeout_ms: u64,
    
    /// Enable RadSec (RADIUS over TLS) (default: true if the feature is enabled)
    #[serde(default = "default_radsec_enabled")]
    pub radsec_enabled: bool,
    
    /// RadSec certificate path (required if radsec_enabled is true)
    pub radsec_cert_path: Option<PathBuf>,
    
    /// RadSec key path (required if radsec_enabled is true)
    pub radsec_key_path: Option<PathBuf>,
    
    /// Require Message-Authenticator attribute (default: true)
    #[serde(default = "default_true")]
    pub require_message_authenticator: bool,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (default: info)
    #[serde(default = "default_log_level")]
    pub level: String,
    
    /// Log file path (optional)
    pub file: Option<PathBuf>,
    
    /// Log to console (default: true)
    #[serde(default = "default_true")]
    pub console: bool,
    
    /// Log format (default: json)
    #[serde(default = "default_log_format")]
    pub format: String,
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection (default: true)
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Enable Prometheus endpoint (default: true)
    #[serde(default = "default_true")]
    pub prometheus_enabled: bool,
    
    /// Prometheus endpoint host (default: 127.0.0.1)
    #[serde(default = "default_metrics_host")]
    pub host: String,
    
    /// Prometheus endpoint port (default: 9090)
    #[serde(default = "default_prometheus_port")]
    pub port: u16,
    
    /// Metrics reporting interval in seconds (default: 10)
    #[serde(default = "default_metrics_interval")]
    pub interval_secs: u64,
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,
    
    /// Security configuration
    pub security: SecurityConfig,
    
    /// Logging configuration
    pub logging: LoggingConfig,
    
    /// Metrics configuration
    pub metrics: MetricsConfig,
    
    /// Authentication backends
    pub auth_backends: HashMap<String, AuthBackendConfig>,
    
    /// Captive portal configuration (optional)
    pub captive_portal: Option<CaptivePortalConfig>,
    
    /// Deployment template (optional)
    #[serde(skip)]
    pub template: Option<DeploymentTemplate>,
}

/// Authentication backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthBackendConfig {
    /// Backend type (local, ldap, radius, oauth, etc.)
    pub backend_type: String,
    
    /// Whether this backend is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Backend-specific configuration
    #[serde(flatten)]
    pub config: HashMap<String, toml::Value>,
}

/// Captive portal configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptivePortalConfig {
    /// Whether the captive portal is enabled
    #[serde(default = "default_false")]
    pub enabled: bool,
    
    /// HTTP port for the captive portal
    #[serde(default = "default_portal_port")]
    pub port: u16,
    
    /// Host to bind the captive portal to
    #[serde(default = "default_host")]
    pub host: String,
    
    /// Path to the template directory
    pub template_dir: PathBuf,
    
    /// Portal branding options
    #[serde(default)]
    pub branding: PortalBrandingConfig,
}

/// Captive portal branding configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PortalBrandingConfig {
    /// Portal title
    #[serde(default = "default_portal_title")]
    pub title: String,
    
    /// Path to logo image
    pub logo: Option<PathBuf>,
    
    /// Primary color (hex)
    #[serde(default = "default_primary_color")]
    pub primary_color: String,
    
    /// Secondary color (hex)
    #[serde(default = "default_secondary_color")]
    pub secondary_color: String,
    
    /// Path to background image
    pub background_image: Option<PathBuf>,
}

/// Deployment template for simplified configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentTemplate {
    /// Basic authentication with local user database
    Basic,
    
    /// Open WiFi with captive portal
    OpenWithCaptivePortal,
    
    /// WPA2/WPA3 Enterprise
    Enterprise,
    
    /// Hotel guest access
    HotelGuest,
    
    /// Cafe/restaurant guest access
    CafeGuest,
    
    /// Corporate guest access
    CorporateGuest,
}

impl Config {
    /// Load configuration from a file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file
    ///
    /// # Returns
    ///
    /// Loaded configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration file cannot be loaded or parsed
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        // GOAL: Simplified Deployment and Configuration
        // Load and parse configuration with good error messages
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file {}: {}", path.display(), e))?;
        
        let mut config: Self = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config file {}: {}", path.display(), e))?;
        
        // Validate the configuration
        config.validate()?;
        
        Ok(config)
    }
    
    /// Create a new configuration from a deployment template
    ///
    /// # Arguments
    ///
    /// * `template` - Deployment template to use
    /// * `secret` - RADIUS shared secret
    ///
    /// # Returns
    ///
    /// New configuration based on the template
    pub fn from_template(template: DeploymentTemplate, secret: String) -> Self {
        // GOAL: Simplified Deployment and Configuration
        // Create pre-configured templates for common deployment scenarios
        let mut config = Self::default();
        config.template = Some(template.clone());
        config.server.secret = secret;
        
        match template {
            DeploymentTemplate::Basic => {
                // Basic configuration with local user database
                let mut auth_backend = AuthBackendConfig {
                    backend_type: "local".to_string(),
                    enabled: true,
                    config: HashMap::new(),
                };
                auth_backend.config.insert("users_file".to_string(), 
                    toml::Value::String("config/users.json".to_string()));
                config.auth_backends.insert("local".to_string(), auth_backend);
            },
            DeploymentTemplate::OpenWithCaptivePortal => {
                // Open WiFi with captive portal configuration
                let mut auth_backend = AuthBackendConfig {
                    backend_type: "mac".to_string(),
                    enabled: true,
                    config: HashMap::new(),
                };
                auth_backend.config.insert("accept_unknown".to_string(), 
                    toml::Value::Boolean(true));
                config.auth_backends.insert("mac".to_string(), auth_backend);
                
                // Enable captive portal
                config.captive_portal = Some(CaptivePortalConfig {
                    enabled: true,
                    port: 8080,
                    host: "0.0.0.0".to_string(),
                    template_dir: PathBuf::from("templates/default"),
                    branding: Default::default(),
                });
            },
            DeploymentTemplate::Enterprise => {
                // WPA2/WPA3 Enterprise configuration
                config.security.auth_protocols = vec![
                    "eap-tls".to_string(),
                    "peap".to_string(),
                    "ttls".to_string(),
                ];
                
                // Add LDAP backend
                let mut auth_backend = AuthBackendConfig {
                    backend_type: "ldap".to_string(),
                    enabled: true,
                    config: HashMap::new(),
                };
                auth_backend.config.insert("server".to_string(), 
                    toml::Value::String("ldap://ldap.example.com:389".to_string()));
                auth_backend.config.insert("bind_dn".to_string(), 
                    toml::Value::String("cn=admin,dc=example,dc=com".to_string()));
                auth_backend.config.insert("bind_password".to_string(), 
                    toml::Value::String("password".to_string()));
                auth_backend.config.insert("user_base_dn".to_string(), 
                    toml::Value::String("ou=users,dc=example,dc=com".to_string()));
                auth_backend.config.insert("user_filter".to_string(), 
                    toml::Value::String("(uid={username})".to_string()));
                
                config.auth_backends.insert("ldap".to_string(), auth_backend);
            },
            DeploymentTemplate::HotelGuest => {
                // Hotel guest access configuration
                Self::configure_hospitality_template(&mut config, "Hotel");
            },
            DeploymentTemplate::CafeGuest => {
                // Cafe guest access configuration
                Self::configure_hospitality_template(&mut config, "Cafe");
            },
            DeploymentTemplate::CorporateGuest => {
                // Corporate guest access configuration
                let mut auth_backend = AuthBackendConfig {
                    backend_type: "oauth".to_string(),
                    enabled: true,
                    config: HashMap::new(),
                };
                auth_backend.config.insert("provider".to_string(), 
                    toml::Value::String("azure".to_string()));
                auth_backend.config.insert("client_id".to_string(), 
                    toml::Value::String("your-client-id".to_string()));
                auth_backend.config.insert("client_secret".to_string(), 
                    toml::Value::String("your-client-secret".to_string()));
                
                config.auth_backends.insert("oauth".to_string(), auth_backend);
                
                // Enable captive portal with corporate branding
                config.captive_portal = Some(CaptivePortalConfig {
                    enabled: true,
                    port: 8080,
                    host: "0.0.0.0".to_string(),
                    template_dir: PathBuf::from("templates/corporate"),
                    branding: PortalBrandingConfig {
                        title: "Corporate WiFi Access".to_string(),
                        logo: Some(PathBuf::from("assets/corporate-logo.png")),
                        primary_color: "#0056b3".to_string(),
                        secondary_color: "#ffffff".to_string(),
                        background_image: None,
                    },
                });
            },
        }
        
        config
    }
    
    /// Configure a hospitality template (hotel or cafe)
    fn configure_hospitality_template(config: &mut Self, venue_type: &str) {
        // GOAL: Modern Public WiFi Features
        // Configure for hospitality use cases with captive portal
        
        // MAC authentication for initial connection
        let mut mac_auth = AuthBackendConfig {
            backend_type: "mac".to_string(),
            enabled: true,
            config: HashMap::new(),
        };
        mac_auth.config.insert("accept_unknown".to_string(), 
            toml::Value::Boolean(true));
        config.auth_backends.insert("mac".to_string(), mac_auth);
        
        // Local user database for vouchers
        let mut local_auth = AuthBackendConfig {
            backend_type: "local".to_string(),
            enabled: true,
            config: HashMap::new(),
        };
        local_auth.config.insert("users_file".to_string(), 
            toml::Value::String("config/vouchers.json".to_string()));
        config.auth_backends.insert("local".to_string(), local_auth);
        
        // Captive portal with venue-specific branding
        let title = format!("{} WiFi Access", venue_type);
        let template_dir = format!("templates/{}", venue_type.to_lowercase());
        let logo = format!("assets/{}-logo.png", venue_type.to_lowercase());
        
        config.captive_portal = Some(CaptivePortalConfig {
            enabled: true,
            port: 8080,
            host: "0.0.0.0".to_string(),
            template_dir: PathBuf::from(template_dir),
            branding: PortalBrandingConfig {
                title,
                logo: Some(PathBuf::from(logo)),
                primary_color: if venue_type == "Hotel" { "#8a2be2" } else { "#4caf50" }.to_string(),
                secondary_color: "#ffffff".to_string(),
                background_image: Some(PathBuf::from(format!("assets/{}-background.jpg", 
                    venue_type.to_lowercase()))),
            },
        });
    }
    
    /// Export configuration to a file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to write the configuration to
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub fn export<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize configuration: {}", e))?;
        
        fs::write(path, content)
            .map_err(|e| format!("Failed to write configuration: {}", e))?;
        
        Ok(())
    }
    
    /// Validate the configuration
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    fn validate(&self) -> Result<()> {
        // GOAL: Security by Design
        // Validate configuration to ensure security
        
        // Validate shared secret
        if self.server.secret.len() < 16 {
            return Err("Shared secret must be at least 16 characters long".into());
        }
        
        // Validate RadSec configuration if enabled
        if self.security.radsec_enabled {
            if self.security.radsec_cert_path.is_none() {
                return Err("RadSec certificate path must be specified when RadSec is enabled".into());
            }
            
            if self.security.radsec_key_path.is_none() {
                return Err("RadSec key path must be specified when RadSec is enabled".into());
            }
        }
        
        // Validate that at least one auth backend is enabled
        let has_enabled_backend = self.auth_backends.values()
            .any(|backend| backend.enabled);
            
        if !has_enabled_backend {
            return Err("At least one authentication backend must be enabled".into());
        }
        
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        // GOAL: Simplified Deployment and Configuration
        // Provide sensible defaults for all configuration options
        Self {
            server: ServerConfig {
                host: default_host(),
                auth_port: default_auth_port(),
                acct_port: default_acct_port(),
                secret: "change-me-to-a-secure-secret".to_string(),
                worker_threads: None,
                shutdown_timeout_secs: default_shutdown_timeout(),
            },
            security: SecurityConfig {
                auth_protocols: default_auth_protocols(),
                max_request_size: default_max_request_size(),
                request_timeout_ms: default_request_timeout(),
                radsec_enabled: default_radsec_enabled(),
                radsec_cert_path: None,
                radsec_key_path: None,
                require_message_authenticator: default_true(),
            },
            logging: LoggingConfig {
                level: default_log_level(),
                file: None,
                console: default_true(),
                format: default_log_format(),
            },
            metrics: MetricsConfig {
                enabled: default_true(),
                prometheus_enabled: default_true(),
                host: default_metrics_host(),
                port: default_prometheus_port(),
                interval_secs: default_metrics_interval(),
            },
            auth_backends: HashMap::new(),
            captive_portal: None,
            template: None,
        }
    }
}

// Default value functions
fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_auth_port() -> u16 {
    1812
}

fn default_acct_port() -> u16 {
    1813
}

fn default_shutdown_timeout() -> u64 {
    30
}

fn default_auth_protocols() -> Vec<String> {
    vec![
        "pap".to_string(),
        "chap".to_string(),
        "mschap".to_string(),
        "peap".to_string(),
    ]
}

fn default_max_request_size() -> usize {
    4096
}

fn default_request_timeout() -> u64 {
    5000
}

fn default_radsec_enabled() -> bool {
    cfg!(feature = "radsec")
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "json".to_string()
}

fn default_metrics_host() -> String {
    "127.0.0.1".to_string()
}

fn default_prometheus_port() -> u16 {
    9090
}

fn default_metrics_interval() -> u64 {
    10
}

fn default_portal_port() -> u16 {
    8080
}

fn default_portal_title() -> String {
    "WiFi Access Portal".to_string()
}

fn default_primary_color() -> String {
    "#4a86e8".to_string()
}

fn default_secondary_color() -> String {
    "#ffffff".to_string()
}
