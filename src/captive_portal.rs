// captive_portal.rs - Captive Portal implementation for rust-radius
//
// This module implements the captive portal functionality, providing web-based
// authentication for users connecting to open WiFi networks.

use serde_json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::{
    extract::{Path as AxumPath, State},
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Form, Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::sync::RwLock;

use crate::auth::AuthResult;
use crate::config::{Config, CaptivePortalConfig};
use crate::Result;

/// Captive portal server
pub struct CaptivePortal {
    /// Server configuration
    config: Arc<Config>,
    
    /// Portal configuration
    portal_config: CaptivePortalConfig,
    
    /// Template engine
    #[allow(dead_code)]
    templates: tera::Tera,
    
    /// Active sessions
    sessions: Arc<RwLock<HashMap<String, Session>>>,
}

/// Portal session
#[derive(Debug, Clone)]
struct Session {
    /// MAC address of the client
    mac: String,
    
    /// IP address of the client
    ip: String,
    
    /// Session creation time
    created_at: chrono::DateTime<chrono::Utc>,
    
    /// Session expiration time
    expires_at: chrono::DateTime<chrono::Utc>,
    
    /// Authentication state
    auth_state: SessionState,
}

/// Session state
#[derive(Debug, Clone, PartialEq)]
enum SessionState {
    /// Unauthenticated
    Unauthenticated,
    
    /// Authenticated
    Authenticated {
        /// Username
        username: String,
        
        /// Authentication time
        auth_time: chrono::DateTime<chrono::Utc>,
    },
}

/// Login request form
#[derive(Debug, Deserialize)]
struct LoginRequest {
    /// MAC address
    mac: String,
    
    /// Username
    username: String,
    
    /// Password
    password: String,
    
    /// Redirect URL after login
    redirect_url: Option<String>,
}

/// Login response
#[derive(Debug, Serialize)]
struct LoginResponse {
    /// Success flag
    success: bool,
    
    /// Error message if login failed
    message: Option<String>,
    
    /// Redirect URL if login succeeded
    redirect_url: Option<String>,
}

/// Guest access request form
#[derive(Debug, Deserialize)]
struct GuestAccessRequest {
    /// MAC address
    mac: String,
    
    /// Email address
    email: String,
    
    /// Acceptance of terms and conditions
    accept_terms: bool,
}

impl CaptivePortal {
    /// Create a new captive portal
    ///
    /// # Arguments
    ///
    /// * `config` - Server configuration
    ///
    /// # Returns
    ///
    /// New captive portal instance
    ///
    /// # Errors
    ///
    /// Returns an error if the captive portal cannot be initialized
    pub fn new(config: Arc<Config>) -> Result<Self> {
        // GOAL: Modern Public WiFi Features
        // Implement captive portal for guest authentication
        
        // Get portal configuration
        let portal_config = match &config.captive_portal {
            Some(cfg) if cfg.enabled => cfg.clone(),
            _ => return Err("Captive portal is not enabled in configuration".into()),
        };
        
        // Initialize template engine
        let template_path = portal_config.template_dir.join("**/*.html");
        let templates = tera::Tera::new(template_path.to_str().unwrap_or("templates/**/*.html"))
            .map_err(|e| format!("Failed to initialize template engine: {}", e))?;
        
        Ok(Self {
            config,
            portal_config,
            templates,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Run the captive portal server
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub async fn run(self) -> Result<()> {
        // GOAL: Modern Public WiFi Features
        // Run the web server for captive portal
        
        let addr = format!("{}:{}", self.portal_config.host, self.portal_config.port);
        let listener = TcpListener::bind(&addr).await
            .map_err(|e| format!("Failed to bind to {}: {}", addr, e))?;
        
        tracing::info!(
            host = self.portal_config.host,
            port = self.portal_config.port,
            "Starting captive portal"
        );
        
        // Create shared state
        let state = Arc::new(self);
        
        // Build router
        let app = Router::new()
            // Static assets
            .route("/assets/*path", get(Self::serve_asset))
            
            // Login page
            .route("/", get(Self::serve_login_page))
            .route("/login", get(Self::serve_login_page))
            
            // Authentication endpoints
            .route("/api/login", post(Self::handle_login))
            .route("/api/guest", post(Self::handle_guest_access))
            
            // Status endpoints
            .route("/api/status/:mac", get(Self::get_status))
            
            // Success page
            .route("/success", get(Self::serve_success_page))
            
            // Shared state
            .with_state(state);
            
        // Run the server
        axum::serve(listener, app)
            .await
            .map_err(|e| format!("Captive portal server error: {}", e).into())
    }
    
    /// Serve a static asset
    async fn serve_asset(
        State(state): State<Arc<Self>>,
        AxumPath(path): AxumPath<String>,
    ) -> impl IntoResponse {
        // Resolve asset path
        let asset_path = state.portal_config.template_dir.join("assets").join(path);
        
        // Ensure the path doesn't escape the assets directory
        if !asset_path.starts_with(state.portal_config.template_dir.join("assets")) {
            return Err("Invalid asset path".to_string());
        }
        
        // Read file
        match tokio::fs::read(&asset_path).await {
            Ok(content) => {
                // Determine content type from extension
                let content_type = match asset_path.extension().and_then(|ext| ext.to_str()) {
                    Some("css") => "text/css",
                    Some("js") => "text/javascript",
                    Some("png") => "image/png",
                    Some("jpg") | Some("jpeg") => "image/jpeg",
                    Some("svg") => "image/svg+xml",
                    Some("woff") => "font/woff",
                    Some("woff2") => "font/woff2",
                    Some("ttf") => "font/ttf",
                    _ => "application/octet-stream",
                };
                
                Ok(([(axum::http::header::CONTENT_TYPE, content_type)], content))
            },
            Err(_) => Err("Asset not found".to_string()),
        }
    }
    
    /// Serve the login page
    async fn serve_login_page(
        State(state): State<Arc<Self>>,
    ) -> impl IntoResponse {
        // GOAL: Modern Public WiFi Features
        // Serve captive portal login page
        
        // In a real implementation, we would:
        // 1. Parse query parameters (mac, ap, etc.)
        // 2. Render the login template with these parameters
        // 3. Include branding from configuration
        
        // For now, return a simple HTML login page
        let html = format!(
            r#"<!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>{title}</title>
                <style>
                    body {{
                        font-family: Arial, sans-serif;
                        margin: 0;
                        padding: 0;
                        display: flex;
                        justify-content: center;
                        align-items: center;
                        min-height: 100vh;
                        background-color: {secondary_color};
                        color: #333;
                    }}
                    .login-container {{
                        background-color: white;
                        border-radius: 8px;
                        box-shadow: 0 4px 10px rgba(0, 0, 0, 0.1);
                        padding: 30px;
                        width: 320px;
                        max-width: 90%;
                    }}
                    .header {{
                        text-align: center;
                        margin-bottom: 20px;
                    }}
                    .header img {{
                        max-width: 150px;
                        margin-bottom: 15px;
                    }}
                    h1 {{
                        font-size: 24px;
                        margin: 0;
                        color: {primary_color};
                    }}
                    form {{
                        display: flex;
                        flex-direction: column;
                    }}
                    label {{
                        font-size: 14px;
                        margin-bottom: 5px;
                    }}
                    input {{
                        padding: 10px;
                        border: 1px solid #ddd;
                        border-radius: 4px;
                        margin-bottom: 15px;
                    }}
                    button {{
                        padding: 10px;
                        background-color: {primary_color};
                        color: white;
                        border: none;
                        border-radius: 4px;
                        cursor: pointer;
                        font-size: 16px;
                    }}
                    .or-divider {{
                        text-align: center;
                        margin: 15px 0;
                        position: relative;
                    }}
                    .or-divider:before, .or-divider:after {{
                        content: "";
                        display: block;
                        width: 45%;
                        height: 1px;
                        background: #ddd;
                        position: absolute;
                        top: 50%;
                    }}
                    .or-divider:before {{ left: 0; }}
                    .or-divider:after {{ right: 0; }}
                    .guest-button {{
                        background-color: #f0f0f0;
                        color: #333;
                    }}
                    .error {{
                        color: red;
                        font-size: 14px;
                        margin-bottom: 10px;
                        display: none;
                    }}
                </style>
            </head>
            <body>
                <div class="login-container">
                    <div class="header">
                        <img src="/assets/logo.png" alt="Logo" id="logo">
                        <h1>{title}</h1>
                    </div>
                    
                    <div class="error" id="error-message"></div>
                    
                    <form id="login-form">
                        <input type="hidden" id="mac" name="mac" value="">
                        <input type="hidden" id="redirect-url" name="redirect_url" value="">
                        
                        <label for="username">Username</label>
                        <input type="text" id="username" name="username" required>
                        
                        <label for="password">Password</label>
                        <input type="password" id="password" name="password" required>
                        
                        <button type="submit">Log in</button>
                    </form>
                    
                    <div class="or-divider">or</div>
                    
                    <form id="guest-form">
                        <input type="hidden" id="guest-mac" name="mac" value="">
                        <input type="email" placeholder="Your email" name="email" required>
                        <label>
                            <input type="checkbox" name="accept_terms" required>
                            I accept the <a href="#terms">terms and conditions</a>
                        </label>
                        <button type="submit" class="guest-button">Continue as Guest</button>
                    </form>
                </div>
                
                <script>
                    // Extract URL parameters
                    const urlParams = new URLSearchParams(window.location.search);
                    const mac = urlParams.get('mac') || '';
                    const redirectUrl = urlParams.get('url') || '';
                    
                    // Set form values
                    document.getElementById('mac').value = mac;
                    document.getElementById('guest-mac').value = mac;
                    document.getElementById('redirect-url').value = redirectUrl;
                    
                    // Handle login form submission
                    document.getElementById('login-form').addEventListener('submit', async (e) => {{
                        e.preventDefault();
                        const formData = new FormData(e.target);
                        const data = Object.fromEntries(formData.entries());
                        
                        try {{
                            const response = await fetch('/api/login', {{
                                method: 'POST',
                                headers: {{
                                    'Content-Type': 'application/json',
                                }},
                                body: JSON.stringify(data),
                            }});
                            
                            const result = await response.json();
                            
                            if (result.success) {{
                                window.location.href = result.redirect_url || '/success';
                            }} else {{
                                const errorElement = document.getElementById('error-message');
                                errorElement.textContent = result.message || 'Login failed';
                                errorElement.style.display = 'block';
                            }}
                        }} catch (error) {{
                            console.error('Login error:', error);
                            const errorElement = document.getElementById('error-message');
                            errorElement.textContent = 'An error occurred. Please try again.';
                            errorElement.style.display = 'block';
                        }}
                    }});
                    
                    // Handle guest form submission
                    document.getElementById('guest-form').addEventListener('submit', async (e) => {{
                        e.preventDefault();
                        const formData = new FormData(e.target);
                        const data = Object.fromEntries(formData.entries());
                        
                        try {{
                            const response = await fetch('/api/guest', {{
                                method: 'POST',
                                headers: {{
                                    'Content-Type': 'application/json',
                                }},
                                body: JSON.stringify(data),
                            }});
                            
                            const result = await response.json();
                            
                            if (result.success) {{
                                window.location.href = result.redirect_url || '/success';
                            }} else {{
                                const errorElement = document.getElementById('error-message');
                                errorElement.textContent = result.message || 'Guest access failed';
                                errorElement.style.display = 'block';
                            }}
                        }} catch (error) {{
                            console.error('Guest access error:', error);
                            const errorElement = document.getElementById('error-message');
                            errorElement.textContent = 'An error occurred. Please try again.';
                            errorElement.style.display = 'block';
                        }}
                    }});
                    
                    // Customize logo behavior when not available
                    document.getElementById('logo').onerror = function() {{
                        this.style.display = 'none';
                    }};
                </script>
            </body>
            </html>
            "#,
            title = state.portal_config.branding.title,
            primary_color = state.portal_config.branding.primary_color,
            secondary_color = state.portal_config.branding.secondary_color
        );
        
        Html(html)
    }
    
    /// Handle login request
    async fn handle_login(
        State(state): State<Arc<Self>>,
        Json(login): Json<LoginRequest>,
    ) -> impl IntoResponse {
        // GOAL: Modern Public WiFi Features
        // Handle login requests from captive portal
        
        // In a real implementation, we would:
        // 1. Validate the login credentials against our auth backends
        // 2. If valid, send a RADIUS CoA request to authorize the client
        // 3. Create a session for the client
        // 4. Return success with redirect URL
        
        tracing::info!(
            mac = login.mac,
            username = login.username,
            "Processing login request"
        );
        
        // For this example, always accept login
        let redirect_url = login.redirect_url.unwrap_or_else(|| "/success".to_string());
        
        // Create a new session
        let session = Session {
            mac: login.mac.clone(),
            ip: "0.0.0.0".to_string(), // This would normally come from the request
            created_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
            auth_state: SessionState::Authenticated {
                username: login.username.clone(),
                auth_time: chrono::Utc::now(),
            },
        };
        
        // Store the session
        {
            let mut sessions = state.sessions.write().await;
            sessions.insert(login.mac.clone(), session);
        }
        
        // Return success response
        Json(LoginResponse {
            success: true,
            message: None,
            redirect_url: Some(redirect_url),
        })
    }
    
    /// Handle guest access request
    async fn handle_guest_access(
        State(state): State<Arc<Self>>,
        Json(guest): Json<GuestAccessRequest>,
    ) -> impl IntoResponse {
        // GOAL: Modern Public WiFi Features
        // Handle guest access requests from captive portal
        
        tracing::info!(
            mac = guest.mac,
            email = guest.email,
            "Processing guest access request"
        );
        
        // Validate request
        if !guest.accept_terms {
            return Json(LoginResponse {
                success: false,
                message: Some("You must accept the terms and conditions".to_string()),
                redirect_url: None,
            });
        }
        
        // Create a new session
        let session = Session {
            mac: guest.mac.clone(),
            ip: "0.0.0.0".to_string(), // This would normally come from the request
            created_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(4), // Guest sessions expire sooner
            auth_state: SessionState::Authenticated {
                username: guest.email.clone(),
                auth_time: chrono::Utc::now(),
            },
        };
        
        // Store the session
        {
            let mut sessions = state.sessions.write().await;
            sessions.insert(guest.mac.clone(), session);
        }
        
        // Return success response
        Json(LoginResponse {
            success: true,
            message: None,
            redirect_url: Some("/success".to_string()),
        })
    }
    
    /// Get client status
    async fn get_status(
        State(state): State<Arc<Self>>,
        AxumPath(mac): AxumPath<String>,
    ) -> impl IntoResponse {
        // Get session for MAC address
        let sessions = state.sessions.read().await;
        
        if let Some(session) = sessions.get(&mac) {
            // Check if session is authenticated
            let authenticated = matches!(session.auth_state, SessionState::Authenticated { .. });
            
            Json(serde_json::json!({
                "mac": session.mac,
                "authenticated": authenticated,
                "expires_at": session.expires_at.to_rfc3339(),
            }))
        } else {
            Json(serde_json::json!({
                "mac": mac,
                "authenticated": false,
                "message": "No session found",
            }))
        }
    }
    
    /// Serve success page
    async fn serve_success_page() -> impl IntoResponse {
        // Simple success page
        Html(r#"<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Connection Successful</title>
            <style>
                body {
                    font-family: Arial, sans-serif;
                    margin: 0;
                    padding: 0;
                    display: flex;
                    justify-content: center;
                    align-items: center;
                    min-height: 100vh;
                    background-color: #f5f5f5;
                }
                .success-container {
                    background-color: white;
                    border-radius: 8px;
                    box-shadow: 0 4px 10px rgba(0, 0, 0, 0.1);
                    padding: 30px;
                    text-align: center;
                    width: 320px;
                    max-width: 90%;
                }
                .success-icon {
                    font-size: 60px;
                    color: #4caf50;
                    margin-bottom: 20px;
                }
                h1 {
                    font-size: 24px;
                    margin: 0 0 20px 0;
                    color: #333;
                }
                p {
                    color: #666;
                    margin-bottom: 20px;
                }
                .button {
                    display: inline-block;
                    padding: 10px 20px;
                    background-color: #4a86e8;
                    color: white;
                    text-decoration: none;
                    border-radius: 4px;
                    font-weight: bold;
                }
            </style>
        </head>
        <body>
            <div class="success-container">
                <div class="success-icon">✓</div>
                <h1>Connection Successful</h1>
                <p>You are now connected to the WiFi network. You can browse the internet.</p>
                <a href="http://example.com" class="button">Start Browsing</a>
            </div>
        </body>
        </html>"#)
    }
    
    /// Send a Change-of-Authorization (CoA) request to authorize a client
    ///
    /// # Arguments
    ///
    /// * `mac` - MAC address of the client
    /// * `username` - Username for the client
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub async fn send_coa_request(&self, mac: &str, username: &str) -> Result<()> {
        // GOAL: Modern Public WiFi Features
        // Implement CoA for seamless authentication after captive portal login
        
        tracing::info!(
            mac = mac,
            username = username,
            "Sending CoA request"
        );
        
        // In a real implementation, we would:
        // 1. Create a RADIUS CoA-Request packet
        // 2. Add attributes (NAS-IP-Address, NAS-Identifier, etc.)
        // 3. Send the packet to the NAS/AP
        // 4. Wait for and process the CoA-ACK or CoA-NAK response
        
        // For now, just log the request
        tracing::info!(
            mac = mac,
            username = username,
            "CoA request would be sent here"
        );
        
        Ok(())
    }
}
