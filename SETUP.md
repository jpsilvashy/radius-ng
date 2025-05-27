# Rust RADIUS: Setup & Configuration Guide

## Core Components Implementation

This guide covers the setup, configuration, and integration of the three core components of Rust RADIUS:

1. **Core RADIUS Server**
2. **Authentication Backends**
3. **Simple Captive Portal**

## System Requirements

- **Operating System**: Linux (Ubuntu 22.04+ recommended), macOS 12+, or Windows 10/11 with WSL2
- **Rust**: 1.70.0 or newer
- **Memory**: 2GB minimum (4GB recommended)
- **Storage**: 1GB available space
- **Network**: Static IP address recommended

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/rust-radius.git
cd rust-radius

# Install dependencies
cargo install --path .

# Build the project
cargo build --release
```

### Using Docker

```bash
# Pull the image
docker pull yourusername/rust-radius:latest

# Run the container
docker run -d --name rust-radius \
  -p 1812:1812/udp \
  -p 1813:1813/udp \
  -p 8080:8080 \
  -v $(pwd)/config:/app/config \
  yourusername/rust-radius:latest
```

## Configuration

### Core RADIUS Server

The RADIUS server configuration is defined in `config/radius.toml`:

```toml
[server]
host = "0.0.0.0"
auth_port = 1812
acct_port = 1813
secret = "your_radius_secret"  # Change this to a secure value

[security]
# Options: "pap", "chap", "mschap", "eap-tls", "eap-ttls", "peap"
auth_protocols = ["pap", "mschap", "peap"]  
max_request_size = 4096
request_timeout_ms = 5000

[logging]
level = "info"  # Options: "debug", "info", "warn", "error"
file = "/var/log/rust-radius/radius.log"
console = true
```

### Authentication Backends

Authentication backends are configured in `config/auth.toml`:

```toml
[backend.local]
type = "local"
enabled = true
users_file = "config/users.json"

[backend.ldap]
type = "ldap"
enabled = false
server = "ldap://ldap.example.com:389"
bind_dn = "cn=admin,dc=example,dc=com"
bind_password = "password"
user_base_dn = "ou=users,dc=example,dc=com"
user_filter = "(uid={username})"

[backend.radius]
type = "radius"
enabled = false
server = "radius.example.com:1812"
secret = "radius_shared_secret"

[backend.oauth]
type = "oauth"
enabled = false
provider = "keycloak"  # Options: "keycloak", "auth0", "okta", "custom"
client_id = "your_client_id"
client_secret = "your_client_secret"
auth_url = "https://auth.example.com/realms/master/protocol/openid-connect/auth"
token_url = "https://auth.example.com/realms/master/protocol/openid-connect/token"
user_info_url = "https://auth.example.com/realms/master/protocol/openid-connect/userinfo"
```

### Captive Portal

Captive portal settings are in `config/portal.toml`:

```toml
[portal]
enabled = true
port = 8080
host = "0.0.0.0"
template_dir = "templates/default"

[portal.branding]
title = "WiFi Access Portal"
logo = "assets/logo.png"
primary_color = "#4a86e8"
secondary_color = "#ffffff"
background_image = "assets/background.jpg"

[portal.features]
guest_access = true
social_login = true
terms_acceptance = true
usage_analytics = true
```

## Integrating with Ubiquiti UniFi Controller

### Prerequisites

- UniFi Controller version 6.0.0 or newer
- Admin access to the UniFi Controller
- Rust RADIUS server running and accessible from the UniFi network

### Configuration Steps

1. **Set up Rust RADIUS Server**:
   
   Ensure your RADIUS server is running with the proper configuration. In `radius.toml`, set the appropriate `secret` that will be shared with the UniFi controller.

2. **Configure UniFi Controller**:

   a. Log in to your UniFi Controller web interface.
   
   b. Navigate to **Settings** > **Networks**.
   
   c. Select the network you want to configure with RADIUS authentication or create a new one.
   
   d. In the **RADIUS** section:
      - Enable RADIUS authentication
      - Set **Authentication Server** to your Rust RADIUS server IP
      - Set **Authentication Port** to 1812 (default)
      - Set **Shared Secret** to match the secret in your `radius.toml`
      
   e. If you're using accounting, configure:
      - **Accounting Server** as your Rust RADIUS server IP
      - **Accounting Port** to 1813 (default)
      
   f. Click **Save** to apply the settings.

3. **Configure RADIUS Profiles (Optional)**:

   a. Navigate to **Settings** > **Profiles** > **RADIUS Profiles**.
   
   b. Create a new profile or edit an existing one.
   
   c. Configure the appropriate settings for your use case.

4. **Test the Configuration**:

   a. Connect a client device to the configured network.
   
   b. You should be redirected to the captive portal if enabled.
   
   c. Check the logs at `/var/log/rust-radius/radius.log` to verify authentication requests.

### Advanced UniFi Integration

#### VLAN Assignment

Rust RADIUS can dynamically assign VLANs based on user attributes. Add the following to your authentication backend configuration:

```toml
[backend.local.vlan_mapping]
enabled = true
attribute = "user_group"  # The attribute to base VLAN assignment on
default_vlan = 1
mappings = [
  { value = "guests", vlan = 10 },
  { value = "staff", vlan = 20 },
  { value = "management", vlan = 30 }
]
```

#### Bandwidth Controls

To implement bandwidth controls, add the following to your RADIUS configuration:

```toml
[bandwidth_control]
enabled = true
default_download = 5  # in Mbps
default_upload = 2    # in Mbps
user_profiles = [
  { name = "basic", download = 10, upload = 5 },
  { name = "premium", download = 50, upload = 20 },
  { name = "unlimited", download = 0, upload = 0 }  # 0 means unlimited
]
```

## Monitoring & Management

### Command-Line Tools

Rust RADIUS comes with built-in CLI tools for management:

```bash
# Check server status
rust-radius status

# Test authentication
rust-radius test-auth username password

# Manage users (when using local backend)
rust-radius user add <username> <password>
rust-radius user delete <username>
rust-radius user list

# View real-time logs
rust-radius logs --follow
```

### API Access

The built-in REST API allows programmatic management:

```
# Get authentication statistics
GET http://localhost:8080/api/v1/stats/auth

# Get active sessions
GET http://localhost:8080/api/v1/sessions

# Disconnect a user
POST http://localhost:8080/api/v1/sessions/{session_id}/disconnect
```

## Troubleshooting

### Common Issues

1. **Authentication Failures**:
   - Verify the shared secret matches between RADIUS and UniFi
   - Check user credentials in the configured backend
   - Ensure authentication protocols match

2. **Captive Portal Not Loading**:
   - Verify network access to the portal server (port 8080)
   - Check portal configuration in `portal.toml`
   - Inspect browser console for JavaScript errors

3. **UniFi Controller Connection Issues**:
   - Confirm firewall rules allow UDP traffic on ports 1812 and 1813
   - Verify IP addressing and routing between UniFi and RADIUS server
   - Check NTP synchronization between systems

### Diagnostic Commands

```bash
# Test RADIUS authentication directly
radtest username password radius-server:1812 0 testing123

# Check network connectivity
nc -uvz radius-server 1812
nc -uvz radius-server 1813

# Verify ports are listening
ss -tulpn | grep -E '1812|1813|8080'
```

## Security Best Practices

1. **Secure Secrets**: 
   - Use strong, unique shared secrets
   - Rotate secrets periodically
   - Store secrets securely (environment variables or a secrets manager)

2. **Transport Security**:
   - Enable TLS for captive portal (HTTPS)
   - Consider implementing RADIUS over TLS (RadSec)
   - Secure admin API with strong authentication

3. **Authentication Hardening**:
   - Implement multi-factor authentication where possible
   - Set minimum password requirements
   - Rate-limit authentication attempts

## Future Expansion

The modular architecture of Rust RADIUS allows for easy expansion. Future integrations planned include:

1. **Advanced AI Features**:
   - User behavior analytics
   - Anomaly detection
   - Predictive bandwidth allocation

2. **IoT Device Management**:
   - Automated device classification
   - Device-specific policies
   - Zero-trust implementation

3. **Enhanced Reporting**:
   - Business intelligence dashboards
   - Compliance reporting
   - Usage trend analysis

## Community Support

Join our community for help and discussion:

- GitHub Issues: [https://github.com/yourusername/rust-radius/issues](https://github.com/yourusername/rust-radius/issues)
- Discord: [https://discord.gg/your-discord](https://discord.gg/your-discord)
- Documentation: [https://docs.rust-radius.io](https://docs.rust-radius.io)
