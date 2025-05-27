// server.rs - Core RADIUS server implementation
//
// This module implements the main RADIUS server functionality with focus on
// high-performance, async processing, and concurrency.

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use log::{debug, error, info, warn};
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tokio::time::{self, Duration};

use crate::auth::{AuthManager, AuthResult};
use crate::config::Config;
use crate::metrics::MetricsManager;
use crate::protocol::{Packet, PacketProcessor};
use crate::Result;

/// Trait defining the core functionality for a RADIUS server handler
/// 
/// This trait allows for different server implementations (e.g., standard, high-performance, test mock)
/// while maintaining a consistent interface
pub trait RadiusServerHandler {
    /// Handle an authentication request
    async fn handle_auth_request(&self, request: &Packet) -> Result<Packet>;
    
    /// Handle an accounting request
    async fn handle_acct_request(&self, request: &Packet) -> Result<Packet>;
    
    /// Handle a CoA (Change of Authorization) request
    async fn handle_coa_request(&self, request: &Packet) -> Result<Packet>;
}

impl RadiusServerHandler for Server {
    /// Process an incoming RADIUS packet using the RadiusServerHandler trait
    async fn process_packet(&self, buf: &[u8], src: SocketAddr) -> Result<Vec<u8>> {
        // GOAL: High-Performance and Concurrency
        // Process incoming packets efficiently using the trait-based approach
        
        // Increment active connections counter
        self.connections.fetch_add(1, Ordering::SeqCst);
        
        // Parse the incoming packet
        let packet = match self.packet_processor.decode(buf, Some(src)) {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to decode packet: {}", e);
                self.connections.fetch_sub(1, Ordering::SeqCst);
                return Err(e);
            }
        };
        
        // Log the request
        debug!("Received {:?} packet from {}", packet.code(), src);
        
        // Use our trait-based handler methods for each packet type
        let response = match packet.code() {
            Packet::ACCESS_REQUEST => {
                match self.handle_auth_request(&packet).await {
                    Ok(resp) => resp,
                    Err(e) => {
                        error!("Authentication error: {}", e);
                        self.connections.fetch_sub(1, Ordering::SeqCst);
                        return Err(e);
                    }
                }
            },
            Packet::ACCOUNTING_REQUEST => {
                match self.handle_acct_request(&packet).await {
                    Ok(resp) => resp,
                    Err(e) => {
                        error!("Accounting error: {}", e);
                        self.connections.fetch_sub(1, Ordering::SeqCst);
                        return Err(e);
                    }
                }
            },
            Packet::COA_REQUEST => {
                match self.handle_coa_request(&packet).await {
                    Ok(resp) => resp,
                    Err(e) => {
                        error!("CoA error: {}", e);
                        self.connections.fetch_sub(1, Ordering::SeqCst);
                        return Err(e);
                    }
                }
            },
            _ => {
                // Unsupported packet type
                let err = format!("Unsupported packet type: {:?}", packet.code());
                error!("{}", err);
                self.connections.fetch_sub(1, Ordering::SeqCst);
                return Err(err.into());
            }
        };
        
        // Encode the response packet
        let response_buf = match self.packet_processor.encode(&response) {
            Ok(buf) => buf,
            Err(e) => {
                error!("Failed to encode response: {}", e);
                self.connections.fetch_sub(1, Ordering::SeqCst);
                return Err(e);
            }
        };
        
        // Log the response
        debug!("Sending {:?} response to {}", response.code(), src);
        
        // Decrement active connections counter
        self.connections.fetch_sub(1, Ordering::SeqCst);
        
        Ok(response_buf)
    }

    /// Handle an authentication request by routing it to the appropriate authentication backend
    async fn handle_auth_request(&self, request: &Packet) -> Result<Packet> {
        // Record request metrics
        self.metrics.record_auth_request();
        let start_time = Instant::now();
        
        // Process the authentication request through the auth manager
        let result = match self.auth_manager.authenticate(request).await {
            Ok(auth_result) => {
                match auth_result {
                    AuthResult::Accept { attributes } => {
                        // Create accept response
                        let mut response = request.create_response(Packet::ACCESS_ACCEPT);
                        for attr in attributes {
                            response.add_attribute(attr);
                        }
                        self.metrics.record_auth_success();
                        Ok(response)
                    },
                    AuthResult::Reject { reason, attributes } => {
                        // Create reject response
                        let mut response = request.create_response(Packet::ACCESS_REJECT);
                        for attr in attributes {
                            response.add_attribute(attr);
                        }
                        self.metrics.record_auth_failure();
                        Ok(response)
                    },
                    AuthResult::Challenge { state, attributes } => {
                        // Create challenge response
                        let mut response = request.create_response(Packet::ACCESS_CHALLENGE);
                        for attr in attributes {
                            response.add_attribute(attr);
                        }
                        self.metrics.record_auth_challenge();
                        Ok(response)
                    },
                }
            },
            Err(err) => {
                // Handle error
                self.metrics.record_auth_error();
                Err(err)
            }
        };
        
        // Record request processing time
        let duration = start_time.elapsed();
        self.metrics.record_request_latency(duration.as_millis() as u64);
        
        result
    }

    /// Handle an accounting request
    async fn handle_acct_request(&self, request: &Packet) -> Result<Packet> {
        // Record metrics
        self.metrics.record_acct_request();
        
        // For now, just acknowledge the accounting request
        // In a real implementation, we would process and store accounting data
        let response = request.create_response(Packet::ACCOUNTING_RESPONSE);
        
        Ok(response)
    }

    /// Handle a Change of Authorization (CoA) request
    async fn handle_coa_request(&self, request: &Packet) -> Result<Packet> {
        // For now, return a CoA-NAK (Not Acknowledged) response
        // In a real implementation, we would validate and process the CoA request
        let response = request.create_response(Packet::COA_NAK);
        
        Ok(response)
    }
}

/// Maximum UDP packet size for RADIUS (RFC 2865)
const MAX_PACKET_SIZE: usize = 4096;

/// Main RADIUS server implementation
pub struct Server {
    /// Server configuration
    config: Arc<Config>,
    
    /// Authentication manager
    auth_manager: Arc<AuthManager>,
    
    /// Packet processor for encoding/decoding
    packet_processor: Arc<PacketProcessor>,
    
    /// Metrics collector
    metrics: Arc<MetricsManager>,
    
    /// Authentication socket
    auth_socket: Option<UdpSocket>,
    
    /// Accounting socket
    acct_socket: Option<UdpSocket>,
    
    /// CoA socket
    coa_socket: Option<UdpSocket>,
    
    /// Shutdown signal
    shutdown: Option<mpsc::Receiver<()>>,
    
    /// Active connections counter
    connections: Arc<AtomicU64>,
}

/// Builder for Server configuration
pub struct ServerBuilder {
    config: Config,
    auth_manager: Option<AuthManager>,
    metrics: Option<MetricsManager>,
}

impl ServerBuilder {
    /// Create a new ServerBuilder with default configuration
    pub fn new(config: Config) -> Self {
        Self {
            config,
            auth_manager: None,
            metrics: None,
        }
    }
    
    /// Set a custom authentication manager
    pub fn with_auth_manager(mut self, auth_manager: AuthManager) -> Self {
        self.auth_manager = Some(auth_manager);
        self
    }
    
    /// Set a custom metrics manager
    pub fn with_metrics(mut self, metrics: MetricsManager) -> Self {
        self.metrics = Some(metrics);
        self
    }
    
    /// Build the Server instance
    pub fn build(self) -> Server {
        let config = Arc::new(self.config);
        
        // Create default auth manager if none provided
        let auth_manager = match self.auth_manager {
            Some(am) => Arc::new(am),
            None => Arc::new(AuthManager::new(config.clone())),
        };
        
        // Create default metrics manager if none provided
        let metrics = match self.metrics {
            Some(m) => Arc::new(m),
            None => Arc::new(MetricsManager::new(config.clone())),
        };
        
        // Create packet processor
        let packet_processor = Arc::new(PacketProcessor::new(config.clone()));
        
        Server {
            config,
            auth_manager,
            packet_processor,
            metrics,
            auth_socket: None,
            acct_socket: None,
            coa_socket: None,
            shutdown: None,
            connections: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl Server {
    /// Create a new RADIUS server instance
    ///
    /// # Arguments
    ///
    /// * `config` - Server configuration
    ///
    /// # Returns
    ///
    /// A new Server instance
    }

    /// Bind to the authentication and accounting ports
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub async fn bind(&mut self) -> Result<()> {
        // GOAL: High-Performance and Concurrency
        // Use socket2 for advanced socket options to optimize performance
        let auth_addr = format!("{}:{}", self.config.server.host, self.config.server.auth_port);
        let acct_addr = format!("{}:{}", self.config.server.host, self.config.server.acct_port);

        // Create and configure authentication socket
        let auth_socket = UdpSocket::bind(&auth_addr).await?;
        
        // Optimize socket for high throughput
        let socket_ref = socket2::Socket::from(auth_socket.into_std()?);
        socket_ref.set_recv_buffer_size(1024 * 1024)?; // 1MB receive buffer
        socket_ref.set_send_buffer_size(1024 * 1024)?; // 1MB send buffer
        
        // Convert back to tokio UdpSocket
        let auth_socket = UdpSocket::from_std(socket_ref.into())?;
        
        // Create and configure accounting socket with similar optimizations
        let acct_socket = UdpSocket::bind(&acct_addr).await?;
        
        self.auth_socket = Some(auth_socket);
        self.acct_socket = Some(acct_socket);
        
        tracing::info!(
            auth_port = self.config.server.auth_port,
            acct_port = self.config.server.acct_port,
            "RADIUS server bound to ports"
        );

        Ok(())
    }

    /// Run the RADIUS server
    ///
    /// This method starts the server and processes incoming requests
    /// until a shutdown signal is received.
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub async fn run(mut self) -> Result<()> {
        // GOAL: High-Performance and Concurrency
        // Use Tokio for async I/O and efficient task management
        
        // Initialize sockets if not already bound
        if self.auth_socket.is_none() || self.acct_socket.is_none() {
            self.bind().await?;
        }
        
        // Unwrap sockets - we know they're initialized from the check above
        let auth_socket = self.auth_socket.take().unwrap();
        let acct_socket = self.acct_socket.take().unwrap();
        
        // Create a channel for shutdown signaling
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
        self.shutdown = Some(shutdown_rx);
        
        // Register signal handlers for graceful shutdown
        Self::register_shutdown_handler(shutdown_tx.clone());
        
        // Create shared sockets
        let auth_socket = Arc::new(auth_socket);
        let acct_socket = Arc::new(acct_socket);
        
        // GOAL: Comprehensive Observability
        // Start metrics reporter task
        let metrics = self.metrics.clone();
        let config = self.config.clone();
        tokio::spawn(async move {
            let interval = Duration::from_secs(config.metrics.interval_secs);
            let mut interval_timer = time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                if let Err(e) = metrics.report().await {
                    tracing::error!(?e, "Failed to report metrics");
                }
            }
        });
        
        // Spawn worker tasks based on CPU cores
        let worker_count = self.config.server.worker_threads.unwrap_or_else(num_cpus::get);
        tracing::info!(workers = worker_count, "Starting RADIUS server workers");
        
        // Spawn authentication worker tasks
        for i in 0..worker_count {
            // Clone required components
            let auth_socket = auth_socket.clone();
            let processor = self.packet_processor.clone();
            let auth_manager = self.auth_manager.clone();
            let metrics = self.metrics.clone();
            let active_connections = self.active_connections.clone();
            
            // Spawn authentication worker
            tokio::spawn(async move {
                let worker_id = format!("auth-{}", i);
                tracing::debug!(worker = worker_id, "Authentication worker started");
                
                // Allocate buffer for receiving packets
                let mut buf = vec![0u8; MAX_PACKET_SIZE];
                
                loop {
                    // Receive packet
                    let (size, src_addr) = match auth_socket.recv_from(&mut buf).await {
                        Ok((size, addr)) => (size, addr),
                        Err(e) => {
                            tracing::error!(?e, "Failed to receive packet");
                            continue;
                        }
                    };
                    
                    // Update active connections metric
                    {
                        let mut count = active_connections.lock().await;
                        *count += 1;
                    }
                    
                    // Start timing for request latency
                    let start_time = std::time::Instant::now();
                    
                    // Process the packet
                    if let Err(e) = Self::process_auth_packet(
                        &worker_id,
                        &buf[..size],
                        src_addr,
                        &auth_socket,
                        &processor,
                        &auth_manager,
                        &metrics,
                    ).await {
                        tracing::error!(?e, src=?src_addr, "Failed to process authentication packet");
                    }
                    
                    // Record request latency
                    let elapsed = start_time.elapsed();
                    metrics.record_request_latency(elapsed.as_millis() as u64);
                    
                    // Update active connections metric
                    {
                        let mut count = active_connections.lock().await;
                        *count -= 1;
                    }
                }
            });
        }
        
        // Similar pattern for accounting packets (simplified here)
        tokio::spawn(async move {
            let mut buf = vec![0u8; MAX_PACKET_SIZE];
            
            loop {
                if let Ok((size, src)) = acct_socket.recv_from(&mut buf).await {
                    tracing::debug!(bytes = size, src = ?src, "Received accounting packet");
                    // Process accounting packet...
                }
            }
        });
        
        // Wait for shutdown signal
        if let Some(mut shutdown) = self.shutdown.take() {
            shutdown.recv().await;
            tracing::info!("Shutdown signal received, stopping server");
        }
        
        // Perform graceful shutdown
        self.shutdown_gracefully().await?;
        
        Ok(())
    }
    
    /// Process an authentication packet
    ///
    /// This method processes a RADIUS authentication packet and sends a response.
    async fn process_auth_packet(
        worker_id: &str,
        data: &[u8],
        src_addr: SocketAddr,
        socket: &UdpSocket,
        processor: &PacketProcessor,
        auth_manager: &AuthManager,
        metrics: &MetricsCollector,
    ) -> Result<()> {
        // GOAL: High-Performance and Concurrency
        // Process packets efficiently with minimal allocations

        // Parse the packet
        let request = processor.parse(data, src_addr)?;
        
        // Log request (debug level to avoid excessive logging)
        tracing::debug!(
            worker = worker_id,
            packet_id = request.identifier(),
            src = ?src_addr,
            "Processing authentication request"
        );
        
        // Record the request in metrics
        metrics.increment_auth_requests();
        
        // Authenticate the request
        let response = auth_manager.authenticate(&request).await?;
        
        // Encode the response packet
        let response_data = processor.encode(&response)?;
        
        // Send the response
        socket.send_to(&response_data, src_addr).await?;
        
        // Record the response in metrics
        metrics.increment_auth_responses();
        
        // Log response
        tracing::debug!(
            worker = worker_id,
            packet_id = response.identifier(),
            src = ?src_addr,
            "Sent authentication response"
        );
        
        Ok(())
    }
    
    /// Register signal handlers for graceful shutdown
    fn register_shutdown_handler(shutdown_tx: mpsc::Sender<()>) {
        // Setup Ctrl+C handler
        let tx = shutdown_tx.clone();
        tokio::spawn(async move {
            match tokio::signal::ctrl_c().await {
                Ok(()) => {
                    tracing::info!("Received Ctrl+C, initiating shutdown");
                    let _ = tx.send(()).await;
                }
                Err(err) => {
                    tracing::error!("Failed to setup Ctrl+C handler: {}", err);
                }
            }
        });
        
        // Setup SIGTERM handler for containerized environments
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            
            let tx = shutdown_tx;
            tokio::spawn(async move {
                let mut sigterm = match signal(SignalKind::terminate()) {
                    Ok(stream) => stream,
                    Err(err) => {
                        tracing::error!("Failed to setup SIGTERM handler: {}", err);
                        return;
                    }
                };
                
                sigterm.recv().await;
                tracing::info!("Received SIGTERM, initiating shutdown");
                let _ = tx.send(()).await;
            });
        }
    }
    
    /// Perform graceful shutdown
    async fn shutdown_gracefully(&self) -> Result<()> {
        // GOAL: Comprehensive Observability
        // Log detailed shutdown process
        tracing::info!("Performing graceful shutdown");
        
        // Wait for active connections to complete (with timeout)
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(self.config.server.shutdown_timeout_secs);
        
        loop {
            let active = *self.active_connections.lock().await;
            if active == 0 {
                break;
            }
            
            if start.elapsed() > timeout {
                tracing::warn!(
                    active = active,
                    "Shutdown timeout reached with active connections remaining"
                );
                break;
            }
            
            tracing::info!(active = active, "Waiting for active connections to complete");
            time::sleep(Duration::from_millis(100)).await;
        }
        
        tracing::info!("Server shutdown complete");
        Ok(())
    }
}
