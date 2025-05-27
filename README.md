# Rust RADIUS: Modern Authentication & Captive Portal Platform

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

## Overview

Rust RADIUS is a high-performance RADIUS (Remote Authentication Dial-In User Service) server implementation written in Rust. The project provides a robust, secure, and extensible authentication and accounting solution with integrated AI-powered captive portal capabilities for network access control.

## Goals

Rust RADIUS aims to be a next-generation RADIUS server that addresses the limitations of legacy implementations while embracing modern network security challenges:

### High-Performance and Concurrency
- Leverage Rust's async runtime to handle thousands of simultaneous authentication requests
- Utilize all CPU cores efficiently with a non-blocking design
- Implement aggressive I/O and network tuning for maximum throughput
- Provide async database access with connection pooling and intelligent caching
- Optimize session handling and cryptographic operations

### Simplified Deployment and Configuration
- Offer an opinionated, user-friendly approach with sensible defaults
- Provide pre-configured templates for common deployment scenarios
- Implement declarative and minimal configuration
- Support modern automation and container orchestration
- Lower the barrier to entry while maintaining flexibility for advanced use cases

### Modern Public WiFi Features
- Built-in support for device fingerprinting and context-aware authentication
- First-class captive portal integration with Change-of-Authorization (CoA) support
- MAC Authentication Bypass (MAB) for IoT and simplified onboarding
- Seamless federation and roaming capabilities (eduroam, OpenRoaming)
- Support for latest WiFi security standards (WPA3, advanced EAP methods)

### Federation and Zero-Trust Integration
- Multi-tenant architecture for service providers and managed services
- Dynamic routing of authentication requests between federated networks
- Integration with zero-trust security frameworks and contextual policy enforcement
- Direct connectivity with modern identity providers (SAML/OIDC, OAuth)
- Support for passwordless and certificate-based authentication

### Comprehensive Observability
- Expose rich metrics in modern formats (Prometheus, OpenTelemetry)
- Implement distributed tracing for complex authentication flows
- Provide structured logging with detailed diagnostics
- Enable real-time monitoring and troubleshooting capabilities
- Proactive detection of authentication issues

### Safe Extensibility
- Support WebAssembly (WASM) plugins for custom logic
- Provide a simple policy language for basic conditions
- Ensure memory safety and sandboxed execution for plugins
- Enable a community ecosystem of extensions
- Balance ease of use with powerful customization options

### Security by Design
- Implement RadSec (RADIUS over TLS) by default
- Address known protocol vulnerabilities (like BlastRADIUS)
- Enforce strong cryptography and integrity protection
- Support multi-factor authentication natively
- Leverage Rust's memory safety to prevent common security flaws

## Features

### Core RADIUS Functionality
- Complete implementation of RADIUS protocol (RFC 2865, 2866)
- High-performance, concurrent request handling
- Modular architecture for easy extension
- Comprehensive logging and monitoring

### Authentication & Accounting
- Flexible backend integrations:
  - OAuth/OIDC providers
  - LDAP/Active Directory
  - SQL databases (PostgreSQL, MySQL, SQLite)
  - NoSQL databases (MongoDB, Redis)
  - Custom authentication providers via plugin system
- Multi-factor authentication support
- Detailed accounting with customizable metrics
- Rate limiting and abuse protection

### AI-Powered Captive Portal
- Intelligent user experience adaptation
- Smart content recommendations
- Natural language processing for user queries
- Behavioral analytics for improved security
- Automated troubleshooting assistance

### Portal Builder
- Drag-and-drop interface designer
- Responsive templates for various devices
- Theme customization and branding options
- Interactive elements (menus, maps, services)
- Multilingual support with automatic translation

### Hospitality & Business Features
- Guest management system
- Digital menu integration for restaurants/cafes
- Interactive property maps and wayfinding
- Local points of interest with recommendations
- Promotional content management
- Usage analytics and reporting

## Getting Started

### Prerequisites
- Rust (latest stable version)
- Database system of your choice
- (Optional) Docker for containerized deployment

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/rust-radius.git
cd rust-radius

# Build the project
cargo build --release

# Run tests
cargo test

# Start the server
cargo run --release
```

### Configuration

Configuration is handled through a combination of environment variables and YAML/TOML configuration files. See the `docs/configuration.md` file for detailed options.

## Architecture

Rust RADIUS follows a modular architecture with clearly separated concerns:

1. **Core RADIUS Server**: Handles RADIUS protocol implementation
2. **Authentication Providers**: Pluggable backends for user verification
3. **Accounting Engine**: Tracks and records usage data
4. **Captive Portal**: Web interface for user interaction
5. **Portal Builder**: Tools for customizing the captive portal
6. **AI Integration Layer**: Provides intelligent features across components

## Use Cases

### Hotels & Hospitality
- Branded guest WiFi access
- Room service integration
- Facility information and booking
- Local attractions and transportation

### Cafes & Restaurants
- Free WiFi with digital menu access
- Order-at-table functionality
- Loyalty program integration
- Customer feedback collection

### Corporate Environments
- Secure guest access
- Compliance with network policies
- Visitor management
- Analytics on network usage

## Contributing

We welcome contributions from the community! Please see our [Contributing Guide](CONTRIBUTING.md) for more information on how to get involved.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Roadmap

- [ ] Core RADIUS server implementation
- [ ] Basic authentication backends
- [ ] Simple captive portal
- [ ] Portal builder MVP
- [ ] AI feature integration
- [ ] Advanced analytics
- [ ] Mobile application support
- [ ] Enterprise features

## Contact

For questions, suggestions, or collaboration opportunities, please open an issue on our GitHub repository or contact the maintainers directly.
