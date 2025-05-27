// protocol.rs - RADIUS protocol implementation
//
// This module handles the RADIUS protocol implementation, including
// packet parsing, attribute handling, and protocol-specific logic.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use bytes::{Bytes, BytesMut};
// We'll use a simple implementation instead of ring for now

use crate::config::Config;
use crate::Result;

/// RADIUS packet codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketCode {
    /// Access-Request (1)
    AccessRequest = 1,
    
    /// Access-Accept (2)
    AccessAccept = 2,
    
    /// Access-Reject (3)
    AccessReject = 3,
    
    /// Accounting-Request (4)
    AccountingRequest = 4,
    
    /// Accounting-Response (5)
    AccountingResponse = 5,
    
    /// Access-Challenge (11)
    AccessChallenge = 11,
    
    /// Status-Server (12)
    StatusServer = 12,
    
    /// Status-Client (13)
    StatusClient = 13,
    
    /// Disconnect-Request (40)
    DisconnectRequest = 40,
    
    /// Disconnect-ACK (41)
    DisconnectAck = 41,
    
    /// Disconnect-NAK (42)
    DisconnectNak = 42,
    
    /// CoA-Request (43)
    CoaRequest = 43,
    
    /// CoA-ACK (44)
    CoaAck = 44,
    
    /// CoA-NAK (45)
    CoaNak = 45,
}

impl PacketCode {
    /// Convert a u8 to a PacketCode
    pub fn from_u8(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::AccessRequest),
            2 => Some(Self::AccessAccept),
            3 => Some(Self::AccessReject),
            4 => Some(Self::AccountingRequest),
            5 => Some(Self::AccountingResponse),
            11 => Some(Self::AccessChallenge),
            12 => Some(Self::StatusServer),
            13 => Some(Self::StatusClient),
            40 => Some(Self::DisconnectRequest),
            41 => Some(Self::DisconnectAck),
            42 => Some(Self::DisconnectNak),
            43 => Some(Self::CoaRequest),
            44 => Some(Self::CoaAck),
            45 => Some(Self::CoaNak),
            _ => None,
        }
    }
}

/// RADIUS attribute types
#[derive(Debug, Clone, PartialEq)]
pub enum Attribute {
    /// String attribute
    String(String, String),
    
    /// Integer attribute
    Integer(String, i32),
    
    /// IP address attribute
    IpAddr(String, std::net::IpAddr),
    
    /// Binary attribute
    Binary(String, Vec<u8>),
    
    /// IPv6 address attribute
    Ipv6Addr(String, std::net::Ipv6Addr),
    
    /// IPv6 prefix attribute
    Ipv6Prefix(String, std::net::Ipv6Addr, u8),
    
    /// Vendor-specific attribute
    VendorSpecific(u32, Vec<Attribute>),
}

impl Attribute {
    /// Get the attribute name
    pub fn name(&self) -> &str {
        match self {
            Self::String(name, _) => name,
            Self::Integer(name, _) => name,
            Self::IpAddr(name, _) => name,
            Self::Binary(name, _) => name,
            Self::Ipv6Addr(name, _) => name,
            Self::Ipv6Prefix(name, ..) => name,
            Self::VendorSpecific(..) => "Vendor-Specific",
        }
    }
}

/// RADIUS packet
#[derive(Debug, Clone)]
pub struct Packet {
    /// Packet code
    code: PacketCode,
    
    /// Packet identifier
    identifier: u8,
    
    /// Authenticator (16 bytes)
    authenticator: [u8; 16],
    
    /// Packet attributes
    attributes: HashMap<String, Attribute>,
    
    /// Raw packet data
    raw_data: Option<Bytes>,
    
    /// Source address (for requests)
    source: Option<SocketAddr>,
}

impl Packet {
    /// Access-Request packet code
    pub const ACCESS_REQUEST: PacketCode = PacketCode::AccessRequest;
    
    /// Access-Accept packet code
    pub const ACCESS_ACCEPT: PacketCode = PacketCode::AccessAccept;
    
    /// Access-Reject packet code
    pub const ACCESS_REJECT: PacketCode = PacketCode::AccessReject;
    
    /// Access-Challenge packet code
    pub const ACCESS_CHALLENGE: PacketCode = PacketCode::AccessChallenge;
    
    /// Create a new RADIUS packet
    ///
    /// # Arguments
    ///
    /// * `code` - Packet code
    /// * `identifier` - Packet identifier
    /// * `authenticator` - Packet authenticator
    ///
    /// # Returns
    ///
    /// New RADIUS packet
    pub fn new(code: PacketCode, identifier: u8, authenticator: [u8; 16]) -> Self {
        Self {
            code,
            identifier,
            authenticator,
            attributes: HashMap::new(),
            raw_data: None,
            source: None,
        }
    }
    
    /// Create a response packet for a request
    ///
    /// # Arguments
    ///
    /// * `code` - Response packet code
    ///
    /// # Returns
    ///
    /// New response packet
    pub fn create_response(&self, code: PacketCode) -> Self {
        Self {
            code,
            identifier: self.identifier,
            authenticator: self.authenticator,
            attributes: HashMap::new(),
            raw_data: None,
            source: self.source,
        }
    }
    
    /// Add an attribute to the packet
    ///
    /// # Arguments
    ///
    /// * `attribute` - Attribute to add
    pub fn add_attribute(&mut self, attribute: Attribute) {
        self.attributes.insert(attribute.name().to_string(), attribute);
    }
    
    /// Get an attribute from the packet
    ///
    /// # Arguments
    ///
    /// * `name` - Attribute name
    ///
    /// # Returns
    ///
    /// Attribute if present, None otherwise
    pub fn get_attribute(&self, name: &str) -> Option<&Attribute> {
        self.attributes.get(name)
    }
    
    /// Get the packet code
    pub fn code(&self) -> PacketCode {
        self.code
    }
    
    /// Get the packet identifier
    pub fn identifier(&self) -> u8 {
        self.identifier
    }
    
    /// Get the packet authenticator
    pub fn authenticator(&self) -> &[u8; 16] {
        &self.authenticator
    }
    
    /// Get the packet source address
    pub fn source(&self) -> Option<SocketAddr> {
        self.source
    }
    
    /// Set the packet source address
    ///
    /// # Arguments
    ///
    /// * `addr` - Source address
    pub fn set_source(&mut self, addr: SocketAddr) {
        self.source = Some(addr);
    }
}

/// RADIUS packet processor
pub struct PacketProcessor {
    /// Server configuration
    config: Arc<Config>,
    
    /// Dictionary of RADIUS attributes
    dictionary: RadiusDictionary,
}

/// RADIUS dictionary for mapping attribute names to codes
struct RadiusDictionary {
    /// Attribute name to code mapping
    attributes: HashMap<String, u8>,
    
    /// Attribute code to name mapping
    attribute_names: HashMap<u8, String>,
    
    /// Vendor-specific attribute dictionaries
    vendor_attributes: HashMap<u32, HashMap<u8, String>>,
}

impl Default for RadiusDictionary {
    fn default() -> Self {
        // GOAL: Security by Design
        // Standard RADIUS attributes (RFC 2865)
        let mut attributes = HashMap::new();
        let mut attribute_names = HashMap::new();
        
        // Define standard attributes
        let standard_attributes = [
            ("User-Name", 1),
            ("User-Password", 2),
            ("CHAP-Password", 3),
            ("NAS-IP-Address", 4),
            ("NAS-Port", 5),
            ("Service-Type", 6),
            ("Framed-Protocol", 7),
            ("Framed-IP-Address", 8),
            ("Framed-IP-Netmask", 9),
            ("Framed-Routing", 10),
            ("Filter-Id", 11),
            ("Framed-MTU", 12),
            ("Framed-Compression", 13),
            ("Login-IP-Host", 14),
            ("Login-Service", 15),
            ("Login-TCP-Port", 16),
            ("Reply-Message", 18),
            ("Callback-Number", 19),
            ("Callback-Id", 20),
            ("Framed-Route", 22),
            ("Framed-IPX-Network", 23),
            ("State", 24),
            ("Class", 25),
            ("Vendor-Specific", 26),
            ("Session-Timeout", 27),
            ("Idle-Timeout", 28),
            ("Termination-Action", 29),
            ("Called-Station-Id", 30),
            ("Calling-Station-Id", 31),
            ("NAS-Identifier", 32),
            ("Proxy-State", 33),
            ("Login-LAT-Service", 34),
            ("Login-LAT-Node", 35),
            ("Login-LAT-Group", 36),
            ("Framed-AppleTalk-Link", 37),
            ("Framed-AppleTalk-Network", 38),
            ("Framed-AppleTalk-Zone", 39),
            ("CHAP-Challenge", 60),
            ("NAS-Port-Type", 61),
            ("Port-Limit", 62),
            ("Login-LAT-Port", 63),
            ("Connect-Info", 77),
            ("Message-Authenticator", 80),
        ];
        
        for (name, code) in standard_attributes.iter() {
            attributes.insert(name.to_string(), *code);
            attribute_names.insert(*code, name.to_string());
        }
        
        // Vendor-specific attributes would be defined here
        let vendor_attributes = HashMap::new();
        
        Self {
            attributes,
            attribute_names,
            vendor_attributes,
        }
    }
}

impl PacketProcessor {
    /// Create a new RADIUS packet processor
    ///
    /// # Arguments
    ///
    /// * `config` - Server configuration
    ///
    /// # Returns
    ///
    /// New packet processor
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            config,
            dictionary: RadiusDictionary::default(),
        }
    }
    
    /// Parse a RADIUS packet from raw bytes
    ///
    /// # Arguments
    ///
    /// * `data` - Raw packet data
    /// * `source` - Source address
    ///
    /// # Returns
    ///
    /// Parsed packet
    ///
    /// # Errors
    ///
    /// Returns an error if the packet cannot be parsed
    pub fn parse(&self, data: &[u8], source: SocketAddr) -> Result<Packet> {
        // GOAL: Security by Design
        // Implement secure packet parsing with validation
        
        // Check minimum packet length
        if data.len() < 20 {
            return Err("Packet too short".into());
        }
        
        // Parse packet header
        let code = match PacketCode::from_u8(data[0]) {
            Some(code) => code,
            None => return Err(format!("Invalid packet code: {}", data[0]).into()),
        };
        
        let identifier = data[1];
        let length = u16::from_be_bytes([data[2], data[3]]) as usize;
        
        // Validate packet length
        if length > data.len() {
            return Err(format!("Packet length ({}) exceeds data length ({})", length, data.len()).into());
        }
        
        if length < 20 {
            return Err(format!("Packet length too short: {}", length).into());
        }
        
        // Extract authenticator
        let mut authenticator = [0u8; 16];
        authenticator.copy_from_slice(&data[4..20]);
        
        // Create packet
        let mut packet = Packet::new(code, identifier, authenticator);
        packet.set_source(source);
        packet.raw_data = Some(Bytes::copy_from_slice(&data[..length]));
        
        // Parse attributes
        self.parse_attributes(&mut packet, &data[20..length])?;
        
        // Validate Message-Authenticator if present
        if self.config.security.require_message_authenticator && 
           code == PacketCode::AccessRequest && 
           packet.get_attribute("Message-Authenticator").is_none() {
            return Err("Missing Message-Authenticator attribute".into());
        }
        
        Ok(packet)
    }
    
    /// Parse attributes from raw bytes
    ///
    /// # Arguments
    ///
    /// * `packet` - RADIUS packet to add attributes to
    /// * `data` - Raw attribute data
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    fn parse_attributes(&self, packet: &mut Packet, data: &[u8]) -> Result<()> {
        let mut offset = 0;
        
        while offset < data.len() {
            // Check if we have enough data for the attribute header
            if offset + 2 > data.len() {
                return Err("Incomplete attribute".into());
            }
            
            // Parse attribute header
            let attr_type = data[offset];
            let attr_length = data[offset + 1] as usize;
            
            // Validate attribute length
            if attr_length < 2 {
                return Err(format!("Invalid attribute length: {}", attr_length).into());
            }
            
            if offset + attr_length > data.len() {
                return Err("Attribute extends beyond packet".into());
            }
            
            // Get attribute value
            let value = &data[offset + 2..offset + attr_length];
            
            // Parse attribute based on type
            match attr_type {
                1 => { // User-Name
                    let username = String::from_utf8_lossy(value).to_string();
                    packet.add_attribute(Attribute::String("User-Name".to_string(), username));
                },
                2 => { // User-Password (encrypted)
                    // In a real implementation, we would decrypt the password here
                    let password = String::from_utf8_lossy(value).to_string();
                    packet.add_attribute(Attribute::String("User-Password".to_string(), password));
                },
                18 => { // Reply-Message
                    let message = String::from_utf8_lossy(value).to_string();
                    packet.add_attribute(Attribute::String("Reply-Message".to_string(), message));
                },
                26 => { // Vendor-Specific
                    // Parse vendor-specific attribute
                    if value.len() < 4 {
                        return Err("Vendor-Specific attribute too short".into());
                    }
                    
                    let vendor_id = u32::from_be_bytes([value[0], value[1], value[2], value[3]]);
                    let _vendor_data = &value[4..];
                    
                    // In a real implementation, we would parse vendor-specific attributes here
                    // For now, just add the raw vendor-specific attribute
                    packet.add_attribute(Attribute::VendorSpecific(vendor_id, vec![]));
                },
                80 => { // Message-Authenticator
                    packet.add_attribute(Attribute::Binary("Message-Authenticator".to_string(), value.to_vec()));
                },
                _ => {
                    // Look up attribute name
                    let attr_name = self.dictionary.attribute_names.get(&attr_type)
                        .map(|s| s.clone())
                        .unwrap_or_else(|| format!("Unknown-{}", attr_type));
                    
                    // Add as binary attribute
                    packet.add_attribute(Attribute::Binary(attr_name, value.to_vec()));
                }
            }
            
            offset += attr_length;
        }
        
        Ok(())
    }
    
    /// Encode a RADIUS packet to bytes
    ///
    /// # Arguments
    ///
    /// * `packet` - RADIUS packet to encode
    ///
    /// # Returns
    ///
    /// Encoded packet bytes
    ///
    /// # Errors
    ///
    /// Returns an error if the packet cannot be encoded
    pub fn encode(&self, packet: &Packet) -> Result<Vec<u8>> {
        // GOAL: High-Performance and Concurrency
        // Efficient packet encoding with minimal allocations
        
        // Calculate packet size
        let mut size = 20; // Header size
        
        for attr in packet.attributes.values() {
            size += self.calculate_attribute_size(attr);
        }
        
        // Check if packet size exceeds maximum
        if size > 4096 {
            return Err("Packet size exceeds maximum".into());
        }
        
        // Allocate buffer
        let mut buffer = BytesMut::with_capacity(size);
        
        // Write packet header
        buffer.extend_from_slice(&[packet.code as u8, packet.identifier]);
        buffer.extend_from_slice(&(size as u16).to_be_bytes());
        buffer.extend_from_slice(&packet.authenticator);
        
        // Write attributes
        for attr in packet.attributes.values() {
            self.encode_attribute(&mut buffer, attr)?;
        }
        
        // Return encoded packet
        Ok(buffer.to_vec())
    }
    
    /// Calculate the size of an attribute
    ///
    /// # Arguments
    ///
    /// * `attr` - Attribute to calculate size for
    ///
    /// # Returns
    ///
    /// Attribute size in bytes
    fn calculate_attribute_size(&self, attr: &Attribute) -> usize {
        match attr {
            Attribute::String(_name, value) => {
                2 + value.len() // Type + Length + Value
            },
            Attribute::Integer(_name, _value) => {
                2 + 4 // Type + Length + Value (4 bytes)
            },
            Attribute::IpAddr(_name, _value) => {
                2 + 4 // Type + Length + IPv4 address (4 bytes)
            },
            Attribute::Binary(_name, value) => {
                2 + value.len() // Type + Length + Value
            },
            Attribute::Ipv6Addr(_name, _value) => {
                2 + 16 // Type + Length + IPv6 address (16 bytes)
            },
            Attribute::Ipv6Prefix(_name, _addr, _prefix_len) => {
                2 + 18 // Type + Length + Reserved (2 bytes) + Prefix length (1 byte) + IPv6 address (16 bytes)
            },
            Attribute::VendorSpecific(_vendor_id, attrs) => {
                let mut size = 2 + 4; // Type + Length + Vendor-Id
                
                for attr in attrs {
                    size += self.calculate_attribute_size(attr);
                }
                
                size
            },
        }
    }
    
    /// Encode an attribute
    ///
    /// # Arguments
    ///
    /// * `buffer` - Buffer to write to
    /// * `attr` - Attribute to encode
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    fn encode_attribute(&self, buffer: &mut BytesMut, attr: &Attribute) -> Result<()> {
        match attr {
            Attribute::String(_name, value) => {
                // Get attribute type
                let attr_type = match self.dictionary.attributes.get(_name) {
                    Some(code) => *code,
                    None => return Err(format!("Unknown attribute: {}", _name).into()),
                };
                
                // Calculate attribute length
                let attr_length = 2 + value.len();
                
                if attr_length > 255 {
                    return Err(format!("Attribute {} value too long", name).into());
                }
                
                // Write attribute header
                buffer.extend_from_slice(&[attr_type, attr_length as u8]);
                
                // Write attribute value
                buffer.extend_from_slice(value.as_bytes());
            },
            Attribute::Integer(_name, _value) => {
                // Get attribute type
                let attr_type = match self.dictionary.attributes.get(_name) {
                    Some(code) => *code,
                    None => return Err(format!("Unknown attribute: {}", _name).into()),
                };
                
                // Write attribute header
                buffer.extend_from_slice(&[attr_type, 6]);
                
                // Write attribute value
                buffer.extend_from_slice(&_value.to_be_bytes());
            },
            Attribute::IpAddr(_name, _value) => {
                // Get attribute type
                let attr_type = match self.dictionary.attributes.get(_name) {
                    Some(code) => *code,
                    None => return Err(format!("Unknown attribute: {}", _name).into()),
                };
                
                // Write attribute header
                buffer.extend_from_slice(&[attr_type, 6]);
                
                // Write attribute value
                match _value {
                    std::net::IpAddr::V4(addr) => {
                        buffer.extend_from_slice(&addr.octets());
                    },
                    std::net::IpAddr::V6(_) => {
                        // IPv6 not supported in standard RADIUS attributes
                        return Err(format!("IPv6 address not supported for attribute {}", _name).into());
                    },
                }
            },
            // Implement other attribute types as needed
            _ => {
                return Err(format!("Unsupported attribute type: {:?}", attr).into());
            }
        }
        
        Ok(())
    }
    
    /// Calculate Message-Authenticator for a packet
    ///
    /// # Arguments
    ///
    /// * `packet` - Packet to calculate Message-Authenticator for
    /// * `secret` - RADIUS shared secret
    ///
    /// # Returns
    ///
    /// Message-Authenticator value
    pub fn calculate_message_authenticator(&self, _packet: &Packet, _secret: &str) -> Vec<u8> {
        // GOAL: Security by Design
        // Implement secure Message-Authenticator calculation
        
        // In a real implementation, we would:
        // 1. Create a copy of the packet with a zero-filled Message-Authenticator
        // 2. Calculate HMAC-MD5 of the packet using the shared secret
        // 3. Return the HMAC-MD5 digest
        
        // For now, return a dummy value for testing purposes
        // In production, this should use a proper HMAC implementation
        vec![0; 16]
    }
    
    /// Verify Message-Authenticator for a packet
    ///
    /// # Arguments
    ///
    /// * `packet` - Packet to verify Message-Authenticator for
    /// * `secret` - RADIUS shared secret
    ///
    /// # Returns
    ///
    /// true if Message-Authenticator is valid, false otherwise
    pub fn verify_message_authenticator(&self, packet: &Packet, secret: &str) -> bool {
        // GOAL: Security by Design
        // Implement secure Message-Authenticator verification
        
        // Get Message-Authenticator from packet
        let message_authenticator = match packet.get_attribute("Message-Authenticator") {
            Some(Attribute::Binary(_, value)) => value,
            _ => return false,
        };
        
        // Calculate expected Message-Authenticator
        let expected = self.calculate_message_authenticator(packet, secret);
        
        // Compare Message-Authenticator values
        message_authenticator == &expected
    }
}
