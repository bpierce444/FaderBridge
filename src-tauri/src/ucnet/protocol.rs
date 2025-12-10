//! UCNet Protocol Implementation
//!
//! This module implements the actual UCNet protocol used by PreSonus StudioLive
//! Series III mixers. Protocol details derived from reverse engineering work by
//! featherbear (https://featherbear.cc/presonus-studiolive-api/).
//!
//! ## Protocol Overview
//! - TCP port 53000 for control communication
//! - UDP port 47809 for discovery broadcasts (console → clients)
//! - All packets start with magic bytes: 0x55 0x43 0x00 0x01 ("UC\x00\x01")
//! - Payload types identified by 2-byte codes (e.g., "UM", "JM", "PV", "KA")
//! - JSON payloads used for subscription and state updates
//! - ZLIB compression for large state dumps (UBJSON format)

use super::error::{Result, UcNetError};
use flate2::read::ZlibDecoder;
use log::debug;
use serde::{Deserialize, Serialize};
use std::io::Read;

/// UCNet protocol magic bytes that start every packet
pub const MAGIC_BYTES: [u8; 4] = [0x55, 0x43, 0x00, 0x01]; // "UC\x00\x01"

/// TCP port for UCNet control communication
pub const CONTROL_PORT: u16 = 53000;

/// UDP port for discovery broadcasts
pub const DISCOVERY_PORT: u16 = 47809;

/// Payload type codes (2 bytes each)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadType {
    /// UM - Hello/Handshake message
    Hello,
    /// JM - JSON message (Subscribe, etc.)
    Json,
    /// PV - Parameter Value (settings)
    ParameterValue,
    /// PS - Parameter Set
    ParameterSet,
    /// PL - Parameter List
    ParameterList,
    /// KA - Keep-Alive
    KeepAlive,
    /// DA - Discovery Advertisement (from console)
    DiscoveryAdvertisement,
    /// DQ - Discovery Query (from client, optional)
    DiscoveryQuery,
    /// ZB - ZLIB compressed data
    ZlibData,
    /// CK - Chunk data (multi-part payload)
    ChunkData,
    /// FR - File Request
    FileRequest,
    /// FD - File Data
    FileData,
    /// MS - Metering/Fader positions
    MeterStatus,
    /// BO - Unknown (possibly boolean)
    Boolean,
    /// Unknown payload type
    Unknown([u8; 2]),
}

impl PayloadType {
    /// Convert payload type to its 2-byte code
    pub fn to_bytes(&self) -> [u8; 2] {
        match self {
            PayloadType::Hello => [0x55, 0x4D],                    // "UM"
            PayloadType::Json => [0x4A, 0x4D],                     // "JM"
            PayloadType::ParameterValue => [0x50, 0x56],           // "PV"
            PayloadType::ParameterSet => [0x50, 0x53],             // "PS"
            PayloadType::ParameterList => [0x50, 0x4C],            // "PL"
            PayloadType::KeepAlive => [0x4B, 0x41],                // "KA"
            PayloadType::DiscoveryAdvertisement => [0x44, 0x41],   // "DA"
            PayloadType::DiscoveryQuery => [0x44, 0x51],           // "DQ"
            PayloadType::ZlibData => [0x5A, 0x42],                 // "ZB"
            PayloadType::ChunkData => [0x43, 0x4B],                // "CK"
            PayloadType::FileRequest => [0x46, 0x52],              // "FR"
            PayloadType::FileData => [0x46, 0x44],                 // "FD"
            PayloadType::MeterStatus => [0x4D, 0x53],              // "MS"
            PayloadType::Boolean => [0x42, 0x4F],                  // "BO"
            PayloadType::Unknown(bytes) => *bytes,
        }
    }

    /// Parse payload type from 2 bytes
    pub fn from_bytes(bytes: [u8; 2]) -> Self {
        match &bytes {
            [0x55, 0x4D] => PayloadType::Hello,
            [0x4A, 0x4D] => PayloadType::Json,
            [0x50, 0x56] => PayloadType::ParameterValue,
            [0x50, 0x53] => PayloadType::ParameterSet,
            [0x50, 0x4C] => PayloadType::ParameterList,
            [0x4B, 0x41] => PayloadType::KeepAlive,
            [0x44, 0x41] => PayloadType::DiscoveryAdvertisement,
            [0x44, 0x51] => PayloadType::DiscoveryQuery,
            [0x5A, 0x42] => PayloadType::ZlibData,
            [0x43, 0x4B] => PayloadType::ChunkData,
            [0x46, 0x52] => PayloadType::FileRequest,
            [0x46, 0x44] => PayloadType::FileData,
            [0x4D, 0x53] => PayloadType::MeterStatus,
            [0x42, 0x4F] => PayloadType::Boolean,
            _ => PayloadType::Unknown(bytes),
        }
    }
}

/// C-Bytes used for request/response matching
/// First byte (A) and third byte (B) are significant, second and fourth are NULL
#[derive(Debug, Clone, Copy, Default)]
pub struct CBytes {
    pub a: u8,
    pub b: u8,
}

impl CBytes {
    /// Create new C-Bytes with default values (r, e)
    /// These are the CBytes that Universal Control uses for Subscribe and PV packets
    pub fn new() -> Self {
        Self { a: 0x72, b: 0x65 } // 'r', 'e' - matches Universal Control
    }

    /// Create C-Bytes from raw bytes
    pub fn from_bytes(bytes: [u8; 4]) -> Self {
        Self {
            a: bytes[0],
            b: bytes[2],
        }
    }

    /// Convert to 4-byte array
    pub fn to_bytes(&self) -> [u8; 4] {
        [self.a, 0x00, self.b, 0x00]
    }

    /// Create response C-Bytes (swapped A and B)
    pub fn response(&self) -> Self {
        Self {
            a: self.b,
            b: self.a,
        }
    }
}

/// UCNet packet header
#[derive(Debug, Clone)]
pub struct PacketHeader {
    /// Magic bytes (always MAGIC_BYTES)
    pub magic: [u8; 4],
    /// Payload size (excluding header)
    pub size: u16,
    /// Payload type code
    pub payload_type: PayloadType,
}

impl PacketHeader {
    /// Parse header from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 8 {
            return Err(UcNetError::InvalidResponse(
                "Packet too short for header".to_string(),
            ));
        }

        let magic: [u8; 4] = bytes[0..4].try_into().map_err(|_| {
            UcNetError::InvalidResponse("Failed to read magic bytes".to_string())
        })?;

        if magic != MAGIC_BYTES {
            return Err(UcNetError::InvalidResponse(format!(
                "Invalid magic bytes: {:02X?}",
                magic
            )));
        }

        // Size is little-endian and includes the 2-byte type
        let size = u16::from_le_bytes([bytes[4], bytes[5]]);
        let payload_type = PayloadType::from_bytes([bytes[6], bytes[7]]);

        Ok(Self {
            magic,
            size,
            payload_type,
        })
    }

    /// Serialize header to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(8);
        bytes.extend_from_slice(&self.magic);
        bytes.extend_from_slice(&self.size.to_le_bytes());
        bytes.extend_from_slice(&self.payload_type.to_bytes());
        bytes
    }

    /// Create a new header with the given payload type and size
    /// Note: size field includes the 2-byte type in UCNet protocol
    pub fn new(payload_type: PayloadType, payload_size: u16) -> Self {
        Self {
            magic: MAGIC_BYTES,
            // Size includes the 2-byte payload type
            size: payload_size + 2,
            payload_type,
        }
    }
}

/// Discovery advertisement packet data
#[derive(Debug, Clone)]
pub struct DiscoveryInfo {
    /// Device model name (e.g., "StudioLive 32SX")
    pub model: String,
    /// Device serial number
    pub serial: String,
    /// Device category (e.g., "AUD" for audio)
    pub category: String,
    /// Friendly name (user-configurable)
    pub friendly_name: String,
}

impl DiscoveryInfo {
    /// Parse discovery info from packet payload
    ///
    /// Discovery packet format (after header):
    /// - C-Bytes (4 bytes)
    /// - Unknown data (variable)
    /// - Model name (null-terminated string)
    /// - Category (null-terminated string, e.g., "AUD")
    /// - Serial number (null-terminated string)
    /// - Friendly name (null-terminated string)
    pub fn from_payload(payload: &[u8]) -> Result<Self> {
        if payload.len() < 20 {
            return Err(UcNetError::InvalidResponse(
                "Discovery payload too short".to_string(),
            ));
        }

        // Skip C-Bytes and unknown header data
        // The strings start after the binary header portion
        // Find null-terminated strings in the payload
        let strings = Self::extract_null_terminated_strings(payload);

        if strings.len() < 2 {
            return Err(UcNetError::InvalidResponse(
                "Not enough strings in discovery payload".to_string(),
            ));
        }

        // Parse based on observed packet structure:
        // First string: Model name (e.g., "StudioLive 24R")
        // Second string: Category (e.g., "AUD")
        // Third string: Serial number
        // Fourth string: Friendly name (may be same as model)
        let model = strings.first().cloned().unwrap_or_default();
        let category = strings.get(1).cloned().unwrap_or_else(|| "AUD".to_string());
        let serial = strings.get(2).cloned().unwrap_or_default();
        let friendly_name = strings.get(3).cloned().unwrap_or_else(|| model.clone());

        Ok(Self {
            model,
            serial,
            category,
            friendly_name,
        })
    }

    /// Extract null-terminated strings from binary data
    fn extract_null_terminated_strings(data: &[u8]) -> Vec<String> {
        let mut strings = Vec::new();
        let mut current = Vec::new();
        let mut in_string = false;

        for &byte in data {
            if byte == 0 {
                if !current.is_empty() {
                    if let Ok(s) = String::from_utf8(current.clone()) {
                        // Only include printable strings
                        if s.chars().all(|c| c.is_ascii_graphic() || c == ' ') {
                            strings.push(s);
                        }
                    }
                    current.clear();
                }
                in_string = false;
            } else if byte.is_ascii_graphic() || byte == b' ' {
                current.push(byte);
                in_string = true;
            } else if in_string {
                // Non-printable byte in middle of string, reset
                current.clear();
                in_string = false;
            }
        }

        // Handle case where string doesn't end with null
        if !current.is_empty() {
            if let Ok(s) = String::from_utf8(current) {
                if s.chars().all(|c| c.is_ascii_graphic() || c == ' ') {
                    strings.push(s);
                }
            }
        }

        strings
    }
}

/// Subscribe request JSON payload
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscribeRequest {
    pub id: String,
    pub client_name: String,
    pub client_internal_name: String,
    pub client_type: String,
    pub client_description: String,
    pub client_identifier: String,
    pub client_options: String,
    pub client_encoding: u32,
}

impl Default for SubscribeRequest {
    fn default() -> Self {
        Self {
            id: "Subscribe".to_string(),
            // These fields must match Universal Control's format for the mixer to accept commands
            client_name: "Universal Control".to_string(),
            client_internal_name: "ucapp".to_string(),
            client_type: "Mac".to_string(),
            client_description: "FaderBridge".to_string(),
            client_identifier: "FaderBridge".to_string(),
            // Options: perm=permissions, users=user list, levl=levels, redu=redux, rtan=real-time analysis
            client_options: "perm users levl redu rtan".to_string(),
            client_encoding: 23106,
        }
    }
}

/// Subscribe response JSON payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeResponse {
    pub id: String,
}

/// Parameter value packet
#[derive(Debug, Clone)]
pub struct ParameterValue {
    /// Parameter key (e.g., "line.ch1.mute", "line.ch1.volume")
    pub key: String,
    /// Parameter value (4 bytes, interpretation depends on parameter type)
    pub value: [u8; 4],
    /// Whether this is a filter group parameter (uses different format)
    pub is_filter_group: bool,
}

impl ParameterValue {
    /// Parse parameter value from payload
    pub fn from_payload(payload: &[u8]) -> Result<Self> {
        // Skip C-Bytes (4 bytes)
        if payload.len() < 6 {
            return Err(UcNetError::InvalidResponse(
                "Parameter value payload too short".to_string(),
            ));
        }

        let data = &payload[4..]; // Skip C-Bytes

        // Find null terminator for key
        let key_end = data
            .iter()
            .position(|&b| b == 0)
            .ok_or_else(|| UcNetError::InvalidResponse("No key terminator found".to_string()))?;

        let key = String::from_utf8(data[..key_end].to_vec())
            .map_err(|_| UcNetError::InvalidResponse("Invalid key encoding".to_string()))?;

        let remaining = &data[key_end + 1..];

        // Check partA (2 bytes after key)
        if remaining.len() < 2 {
            return Err(UcNetError::InvalidResponse(
                "Missing partA in parameter value".to_string(),
            ));
        }

        let is_filter_group = remaining[0] == 0x00 && remaining[1] == 0x01;

        let value = if is_filter_group {
            // Filter group parameters don't have partB
            [0, 0, 0, 0]
        } else if remaining.len() >= 6 {
            // Normal parameters have 4-byte value after partA
            [remaining[2], remaining[3], remaining[4], remaining[5]]
        } else {
            [0, 0, 0, 0]
        };

        Ok(Self {
            key,
            value,
            is_filter_group,
        })
    }

    /// Get value as f32 (for fader levels, pan, etc.)
    pub fn as_f32(&self) -> f32 {
        f32::from_le_bytes(self.value)
    }

    /// Get value as u32
    pub fn as_u32(&self) -> u32 {
        u32::from_le_bytes(self.value)
    }

    /// Get value as bool (for mute, solo, etc.)
    pub fn as_bool(&self) -> bool {
        self.value != [0, 0, 0, 0]
    }

    /// Create a parameter set packet payload
    pub fn to_set_payload(key: &str, value: f32, cbytes: CBytes) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(&cbytes.to_bytes());
        payload.extend_from_slice(key.as_bytes());
        payload.push(0x00); // Null terminator
        payload.extend_from_slice(&[0x00, 0x00]); // partA for normal parameters
        payload.extend_from_slice(&value.to_le_bytes());
        payload
    }

    /// Create a parameter set packet payload for boolean values
    pub fn to_set_bool_payload(key: &str, value: bool, cbytes: CBytes) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(&cbytes.to_bytes());
        payload.extend_from_slice(key.as_bytes());
        payload.push(0x00); // Null terminator
        payload.extend_from_slice(&[0x00, 0x00]); // partA
        let bool_value: u32 = if value { 1 } else { 0 };
        payload.extend_from_slice(&bool_value.to_le_bytes());
        payload
    }
}

/// Build a complete UCNet packet
pub fn build_packet(payload_type: PayloadType, payload: &[u8]) -> Vec<u8> {
    let header = PacketHeader::new(payload_type, payload.len() as u16);
    let mut packet = header.to_bytes();
    packet.extend_from_slice(payload);
    packet
}

/// Build a Hello packet (first packet sent to console)
pub fn build_hello_packet() -> Vec<u8> {
    // Hello packet: UM with minimal payload
    // 55 43 00 01 08 00 55 4d 00 00 65 00 15 fa
    let payload = [0x00, 0x00, 0x65, 0x00, 0x15, 0xFA];
    build_packet(PayloadType::Hello, &payload)
}

/// Build a Subscribe packet
pub fn build_subscribe_packet(request: &SubscribeRequest) -> Result<Vec<u8>> {
    let json = serde_json::to_string(request)
        .map_err(|e| UcNetError::Protocol(format!("Failed to serialize subscribe request: {}", e)))?;

    let cbytes = CBytes::new();
    let mut payload = Vec::new();
    payload.extend_from_slice(&cbytes.to_bytes());

    // JSON length as u32 LE
    let json_len = json.len() as u32;
    payload.extend_from_slice(&json_len.to_le_bytes());

    // JSON data
    payload.extend_from_slice(json.as_bytes());

    Ok(build_packet(PayloadType::Json, &payload))
}

/// Build a Keep-Alive packet
pub fn build_keepalive_packet() -> Vec<u8> {
    // KA packet is just the header with empty payload
    build_packet(PayloadType::KeepAlive, &[])
}

/// Build a Parameter Value packet (PV) for setting float parameters
/// 
/// This uses the PV packet type which is what Universal Control uses
/// for controlling mixer parameters like fader levels.
pub fn build_parameter_value_packet(key: &str, value: f32) -> Vec<u8> {
    let payload = ParameterValue::to_set_payload(key, value, CBytes::new());
    build_packet(PayloadType::ParameterValue, &payload)
}

/// Build a Parameter Value packet (PV) for boolean values
pub fn build_parameter_value_bool_packet(key: &str, value: bool) -> Vec<u8> {
    let payload = ParameterValue::to_set_bool_payload(key, value, CBytes::new());
    build_packet(PayloadType::ParameterValue, &payload)
}

/// Build a Parameter Set packet (PS) - legacy, prefer build_parameter_value_packet
#[deprecated(note = "Use build_parameter_value_packet instead - PV packets are what UC uses")]
pub fn build_parameter_set_packet(key: &str, value: f32) -> Vec<u8> {
    let payload = ParameterValue::to_set_payload(key, value, CBytes::new());
    build_packet(PayloadType::ParameterSet, &payload)
}

/// Build a Parameter Set packet for boolean values - legacy
#[deprecated(note = "Use build_parameter_value_bool_packet instead")]
pub fn build_parameter_set_bool_packet(key: &str, value: bool) -> Vec<u8> {
    let payload = ParameterValue::to_set_bool_payload(key, value, CBytes::new());
    build_packet(PayloadType::ParameterSet, &payload)
}

// =============================================================================
// ZLIB State Dump Handling
// =============================================================================

/// Decompress ZLIB-compressed data from ZB packets
///
/// UCNet uses ZLIB compression for large state dumps sent after subscription.
/// The compressed data contains UBJSON-encoded mixer state.
pub fn decompress_zlib(compressed: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(compressed);
    let mut decompressed = Vec::new();
    
    decoder.read_to_end(&mut decompressed).map_err(|e| {
        UcNetError::Protocol(format!("ZLIB decompression failed: {}", e))
    })?;
    
    debug!(
        "Decompressed {} bytes to {} bytes",
        compressed.len(),
        decompressed.len()
    );
    
    Ok(decompressed)
}

/// State dump entry from ZLIB-compressed state
#[derive(Debug, Clone)]
pub struct StateEntry {
    /// Parameter key
    pub key: String,
    /// Parameter value (raw bytes)
    pub value: Vec<u8>,
}

/// Parse state entries from decompressed UBJSON data
///
/// UBJSON format uses type markers followed by data:
/// - 'S' = string (followed by length marker and string)
/// - 'i' = int8, 'I' = int16, 'l' = int32, 'L' = int64
/// - 'd' = float32, 'D' = float64
/// - '{' = object start, '}' = object end
/// - '[' = array start, ']' = array end
///
/// State dump format: key-value pairs where keys are parameter paths
pub fn parse_state_dump(data: &[u8]) -> Result<Vec<StateEntry>> {
    let mut entries = Vec::new();
    let mut pos = 0;
    
    while pos < data.len() {
        // Try to find key-value pairs
        // Keys are typically null-terminated strings followed by values
        
        // Look for string start
        if pos + 1 >= data.len() {
            break;
        }
        
        // Simple heuristic: look for printable ASCII sequences that look like keys
        if data[pos].is_ascii_alphabetic() {
            // Find end of key (null terminator or non-printable)
            let key_start = pos;
            while pos < data.len() && data[pos] != 0 && data[pos].is_ascii_graphic() {
                pos += 1;
            }
            
            if pos > key_start && pos < data.len() {
                let key = String::from_utf8_lossy(&data[key_start..pos]).to_string();
                
                // Skip null terminator if present
                if pos < data.len() && data[pos] == 0 {
                    pos += 1;
                }
                
                // Read value (assume 4 bytes for now - most common)
                if pos + 4 <= data.len() && key.contains('.') {
                    let value = data[pos..pos + 4].to_vec();
                    entries.push(StateEntry { key, value });
                    pos += 4;
                    continue;
                }
            }
        }
        
        pos += 1;
    }
    
    debug!("Parsed {} state entries from dump", entries.len());
    Ok(entries)
}

// =============================================================================
// Incoming Packet Handling
// =============================================================================

/// Parsed incoming packet from UCNet device
#[derive(Debug, Clone)]
pub enum IncomingPacket {
    /// Parameter value change (PV)
    ParameterChange(ParameterValue),
    /// Keep-alive response (KA)
    KeepAlive,
    /// JSON message (JM) - subscription reply, etc.
    Json(String),
    /// ZLIB compressed state dump (ZB)
    StateDump(Vec<StateEntry>),
    /// Metering data (MS)
    Metering(Vec<f32>),
    /// Unknown packet type
    Unknown(PayloadType, Vec<u8>),
}

/// Parse an incoming UCNet packet
///
/// This function handles all incoming packet types from the mixer:
/// - PV: Parameter value changes (fader moves, mute toggles, etc.)
/// - KA: Keep-alive responses
/// - JM: JSON messages (subscription replies)
/// - ZB: ZLIB compressed state dumps
/// - MS: Metering/fader position data
pub fn parse_incoming_packet(data: &[u8]) -> Result<IncomingPacket> {
    let header = PacketHeader::from_bytes(data)?;
    
    // Extract payload (after 8-byte header)
    let payload = if data.len() > 8 {
        &data[8..]
    } else {
        &[]
    };
    
    match header.payload_type {
        PayloadType::ParameterValue => {
            let pv = ParameterValue::from_payload(payload)?;
            debug!("Received PV: {} = {:?}", pv.key, pv.value);
            Ok(IncomingPacket::ParameterChange(pv))
        }
        
        PayloadType::KeepAlive => {
            debug!("Received KA");
            Ok(IncomingPacket::KeepAlive)
        }
        
        PayloadType::Json => {
            // JSON payload: C-Bytes (4) + length (4) + JSON string
            if payload.len() < 8 {
                return Err(UcNetError::InvalidResponse(
                    "JSON payload too short".to_string(),
                ));
            }
            
            let json_len = u32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]) as usize;
            
            if payload.len() < 8 + json_len {
                return Err(UcNetError::InvalidResponse(
                    "JSON payload truncated".to_string(),
                ));
            }
            
            let json_str = String::from_utf8(payload[8..8 + json_len].to_vec())
                .map_err(|_| UcNetError::InvalidResponse("Invalid JSON encoding".to_string()))?;
            
            debug!("Received JM: {}", &json_str[..json_str.len().min(100)]);
            Ok(IncomingPacket::Json(json_str))
        }
        
        PayloadType::ZlibData => {
            // ZLIB payload: C-Bytes (4) + compressed data
            if payload.len() < 5 {
                return Err(UcNetError::InvalidResponse(
                    "ZLIB payload too short".to_string(),
                ));
            }
            
            let compressed = &payload[4..];
            let decompressed = decompress_zlib(compressed)?;
            let entries = parse_state_dump(&decompressed)?;
            
            debug!("Received ZB: {} entries", entries.len());
            Ok(IncomingPacket::StateDump(entries))
        }
        
        PayloadType::MeterStatus => {
            // Metering data: array of f32 values for channel levels
            // Skip C-Bytes (4 bytes)
            if payload.len() < 4 {
                return Ok(IncomingPacket::Metering(Vec::new()));
            }
            
            let meter_data = &payload[4..];
            let mut levels = Vec::new();
            
            // Each level is a 4-byte float
            for chunk in meter_data.chunks_exact(4) {
                let level = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                levels.push(level);
            }
            
            debug!("Received MS: {} channels", levels.len());
            Ok(IncomingPacket::Metering(levels))
        }
        
        _ => {
            debug!("Received unknown packet type: {:?}", header.payload_type);
            Ok(IncomingPacket::Unknown(header.payload_type, payload.to_vec()))
        }
    }
}

/// Parse multiple packets from a buffer (packets may be concatenated)
pub fn parse_packet_stream(data: &[u8]) -> Vec<Result<IncomingPacket>> {
    let mut packets = Vec::new();
    let mut pos = 0;
    
    while pos + 8 <= data.len() {
        // Check for magic bytes
        if data[pos..pos + 4] != MAGIC_BYTES {
            pos += 1;
            continue;
        }
        
        // Parse header to get size
        let size = u16::from_be_bytes([data[pos + 4], data[pos + 5]]) as usize;
        let packet_len = 8 + size;
        
        if pos + packet_len > data.len() {
            // Incomplete packet
            break;
        }
        
        let packet_data = &data[pos..pos + packet_len];
        packets.push(parse_incoming_packet(packet_data));
        
        pos += packet_len;
    }
    
    packets
}

// =============================================================================
// USB Protocol Constants and Helpers
// =============================================================================

/// USB endpoint addresses for UCNet communication
pub mod usb {
    /// USB Vendor ID for PreSonus
    pub const PRESONUS_VENDOR_ID: u16 = 0x194F;
    
    /// USB Product IDs for Series III mixers
    pub mod product_ids {
        pub const STUDIOLIVE_32S: u16 = 0x0101;
        pub const STUDIOLIVE_32SC: u16 = 0x0102;
        pub const STUDIOLIVE_64S: u16 = 0x0103;
        pub const STUDIOLIVE_32SX: u16 = 0x0104;
        pub const STUDIOLIVE_24R: u16 = 0x0105;
        pub const STUDIOLIVE_32R: u16 = 0x0106;
    }
    
    /// USB interface number for UCNet control
    pub const UCNET_INTERFACE: u8 = 3;
    
    /// USB endpoint for sending data (OUT)
    pub const ENDPOINT_OUT: u8 = 0x03;
    
    /// USB endpoint for receiving data (IN)
    pub const ENDPOINT_IN: u8 = 0x83;
    
    /// USB transfer timeout in milliseconds
    pub const TRANSFER_TIMEOUT_MS: u64 = 1000;
    
    /// Maximum USB packet size
    pub const MAX_PACKET_SIZE: usize = 512;
    
    /// Check if a USB device is a supported PreSonus mixer
    pub fn is_supported_mixer(vendor_id: u16, product_id: u16) -> bool {
        if vendor_id != PRESONUS_VENDOR_ID {
            return false;
        }
        
        matches!(
            product_id,
            product_ids::STUDIOLIVE_32S
                | product_ids::STUDIOLIVE_32SC
                | product_ids::STUDIOLIVE_64S
                | product_ids::STUDIOLIVE_32SX
                | product_ids::STUDIOLIVE_24R
                | product_ids::STUDIOLIVE_32R
        )
    }
    
    /// Get mixer model name from product ID
    pub fn get_model_name(product_id: u16) -> Option<&'static str> {
        match product_id {
            product_ids::STUDIOLIVE_32S => Some("StudioLive 32S"),
            product_ids::STUDIOLIVE_32SC => Some("StudioLive 32SC"),
            product_ids::STUDIOLIVE_64S => Some("StudioLive 64S"),
            product_ids::STUDIOLIVE_32SX => Some("StudioLive 32SX"),
            product_ids::STUDIOLIVE_24R => Some("StudioLive 24R"),
            product_ids::STUDIOLIVE_32R => Some("StudioLive 32R"),
            _ => None,
        }
    }
}

/// USB packet wrapper for UCNet over USB
///
/// USB UCNet packets have the same format as TCP packets but may be
/// fragmented across multiple USB transfers.
#[derive(Debug, Clone)]
pub struct UsbPacketBuffer {
    /// Accumulated data from USB transfers
    buffer: Vec<u8>,
    /// Expected packet length (0 if unknown)
    expected_len: usize,
}

impl UsbPacketBuffer {
    /// Create a new USB packet buffer
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(usb::MAX_PACKET_SIZE * 4),
            expected_len: 0,
        }
    }
    
    /// Add data from a USB transfer
    ///
    /// Returns complete packets that can be parsed
    pub fn add_data(&mut self, data: &[u8]) -> Vec<Vec<u8>> {
        self.buffer.extend_from_slice(data);
        
        let mut complete_packets = Vec::new();
        
        loop {
            // Need at least 8 bytes for header
            if self.buffer.len() < 8 {
                break;
            }
            
            // Check for magic bytes
            if self.buffer[0..4] != MAGIC_BYTES {
                // Scan for magic bytes
                if let Some(pos) = self.find_magic() {
                    self.buffer.drain(0..pos);
                } else {
                    // No magic found, clear buffer
                    self.buffer.clear();
                    break;
                }
                continue;
            }
            
            // Parse size from header
            let size = u16::from_be_bytes([self.buffer[4], self.buffer[5]]) as usize;
            let packet_len = 8 + size;
            
            if self.buffer.len() >= packet_len {
                // Complete packet available
                let packet: Vec<u8> = self.buffer.drain(0..packet_len).collect();
                complete_packets.push(packet);
            } else {
                // Waiting for more data
                self.expected_len = packet_len;
                break;
            }
        }
        
        complete_packets
    }
    
    /// Find magic bytes in buffer
    fn find_magic(&self) -> Option<usize> {
        self.buffer
            .windows(4)
            .position(|w| w == MAGIC_BYTES)
    }
    
    /// Clear the buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.expected_len = 0;
    }
    
    /// Check if buffer has pending data
    pub fn has_pending(&self) -> bool {
        !self.buffer.is_empty()
    }
    
    /// Get expected remaining bytes for current packet
    pub fn bytes_needed(&self) -> usize {
        if self.expected_len > self.buffer.len() {
            self.expected_len - self.buffer.len()
        } else {
            0
        }
    }
}

impl Default for UsbPacketBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// UCNet parameter key builders for common parameters
pub mod keys {
    /// Build a channel volume key
    /// Format: line.ch{N}.volume or line.ch{N}.pan, etc.
    pub fn channel_volume(channel: u8) -> String {
        format!("line.ch{}.volume", channel)
    }

    /// Build a channel mute key
    pub fn channel_mute(channel: u8) -> String {
        format!("line.ch{}.mute", channel)
    }

    /// Build a channel pan key
    pub fn channel_pan(channel: u8) -> String {
        format!("line.ch{}.pan", channel)
    }

    /// Build a channel solo key
    pub fn channel_solo(channel: u8) -> String {
        format!("line.ch{}.solo", channel)
    }

    /// Build an aux send level key
    pub fn aux_send(channel: u8, aux: u8) -> String {
        format!("line.ch{}.aux{}.level", channel, aux)
    }

    /// Build a main mix fader key
    pub fn main_volume() -> String {
        "main.ch1.volume".to_string()
    }

    /// Build a main mix mute key
    pub fn main_mute() -> String {
        "main.ch1.mute".to_string()
    }

    /// Build an FX return level key
    pub fn fx_return(fx: u8) -> String {
        format!("fxrtn.ch{}.volume", fx)
    }

    /// Build a subgroup volume key
    pub fn subgroup_volume(subgroup: u8) -> String {
        format!("sub.ch{}.volume", subgroup)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_bytes() {
        assert_eq!(MAGIC_BYTES, [0x55, 0x43, 0x00, 0x01]);
    }

    #[test]
    fn test_payload_type_roundtrip() {
        let types = [
            PayloadType::Hello,
            PayloadType::Json,
            PayloadType::ParameterValue,
            PayloadType::KeepAlive,
            PayloadType::DiscoveryAdvertisement,
        ];

        for pt in types {
            let bytes = pt.to_bytes();
            let parsed = PayloadType::from_bytes(bytes);
            assert_eq!(pt, parsed);
        }
    }

    #[test]
    fn test_cbytes_response() {
        let cbytes = CBytes::new();
        assert_eq!(cbytes.a, 0x6A); // 'j'
        assert_eq!(cbytes.b, 0x65); // 'e'

        let response = cbytes.response();
        assert_eq!(response.a, 0x65); // 'e'
        assert_eq!(response.b, 0x6A); // 'j'
    }

    #[test]
    fn test_packet_header_roundtrip() {
        let header = PacketHeader::new(PayloadType::Json, 256);
        let bytes = header.to_bytes();
        let parsed = PacketHeader::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.magic, MAGIC_BYTES);
        assert_eq!(parsed.size, 256);
        assert_eq!(parsed.payload_type, PayloadType::Json);
    }

    #[test]
    fn test_build_hello_packet() {
        let packet = build_hello_packet();
        assert!(packet.starts_with(&MAGIC_BYTES));
        assert_eq!(packet[6..8], PayloadType::Hello.to_bytes());
    }

    #[test]
    fn test_build_keepalive_packet() {
        let packet = build_keepalive_packet();
        assert!(packet.starts_with(&MAGIC_BYTES));
        assert_eq!(packet[6..8], PayloadType::KeepAlive.to_bytes());
    }

    #[test]
    fn test_build_subscribe_packet() {
        let request = SubscribeRequest::default();
        let packet = build_subscribe_packet(&request).unwrap();
        assert!(packet.starts_with(&MAGIC_BYTES));
        assert_eq!(packet[6..8], PayloadType::Json.to_bytes());
    }

    #[test]
    fn test_parameter_keys() {
        assert_eq!(keys::channel_volume(1), "line.ch1.volume");
        assert_eq!(keys::channel_mute(5), "line.ch5.mute");
        assert_eq!(keys::channel_pan(3), "line.ch3.pan");
        assert_eq!(keys::aux_send(1, 2), "line.ch1.aux2.level");
        assert_eq!(keys::main_volume(), "main.ch1.volume");
    }

    #[test]
    fn test_discovery_string_extraction() {
        // Simulated discovery payload with null-terminated strings
        let payload = b"StudioLive 32SX\0AUD\0SL32SX123456\0My Mixer\0";
        let strings = DiscoveryInfo::extract_null_terminated_strings(payload);

        assert_eq!(strings.len(), 4);
        assert_eq!(strings[0], "StudioLive 32SX");
        assert_eq!(strings[1], "AUD");
        assert_eq!(strings[2], "SL32SX123456");
        assert_eq!(strings[3], "My Mixer");
    }

    #[test]
    fn test_parameter_value_f32() {
        let pv = ParameterValue {
            key: "line.ch1.volume".to_string(),
            value: 0.75_f32.to_le_bytes(),
            is_filter_group: false,
        };
        assert!((pv.as_f32() - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_parameter_value_bool() {
        let pv_true = ParameterValue {
            key: "line.ch1.mute".to_string(),
            value: [1, 0, 0, 0],
            is_filter_group: false,
        };
        assert!(pv_true.as_bool());

        let pv_false = ParameterValue {
            key: "line.ch1.mute".to_string(),
            value: [0, 0, 0, 0],
            is_filter_group: false,
        };
        assert!(!pv_false.as_bool());
    }

    #[test]
    fn test_zlib_decompression() {
        // Create some test data and compress it
        use flate2::write::ZlibEncoder;
        use flate2::Compression;
        use std::io::Write;
        
        let original = b"line.ch1.volume\x00\x00\x00\x80\x3fline.ch2.mute\x00\x01\x00\x00\x00";
        
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(original).unwrap();
        let compressed = encoder.finish().unwrap();
        
        let decompressed = decompress_zlib(&compressed).unwrap();
        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_parse_incoming_keepalive() {
        // Build a KA packet
        let packet = build_keepalive_packet();
        let parsed = parse_incoming_packet(&packet).unwrap();
        
        assert!(matches!(parsed, IncomingPacket::KeepAlive));
    }

    #[test]
    fn test_parse_incoming_pv() {
        // Build a PV packet manually
        let mut packet = Vec::new();
        packet.extend_from_slice(&MAGIC_BYTES);
        
        // Payload: C-Bytes + key + null + partA + value
        let key = b"line.ch1.volume";
        let value: f32 = 0.75;
        let payload_len = 4 + key.len() + 1 + 2 + 4; // C-Bytes + key + null + partA + value
        
        packet.extend_from_slice(&(payload_len as u16).to_be_bytes());
        packet.extend_from_slice(&PayloadType::ParameterValue.to_bytes());
        
        // C-Bytes
        packet.extend_from_slice(&[0x6A, 0x00, 0x65, 0x00]);
        // Key
        packet.extend_from_slice(key);
        packet.push(0x00); // null terminator
        // partA
        packet.extend_from_slice(&[0x00, 0x00]);
        // Value
        packet.extend_from_slice(&value.to_le_bytes());
        
        let parsed = parse_incoming_packet(&packet).unwrap();
        
        if let IncomingPacket::ParameterChange(pv) = parsed {
            assert_eq!(pv.key, "line.ch1.volume");
            assert!((pv.as_f32() - 0.75).abs() < 0.001);
        } else {
            panic!("Expected ParameterChange");
        }
    }

    #[test]
    fn test_parse_incoming_json() {
        // Build a JM packet
        let json = r#"{"id":"SubscriptionReply"}"#;
        let mut packet = Vec::new();
        packet.extend_from_slice(&MAGIC_BYTES);
        
        let payload_len = 4 + 4 + json.len(); // C-Bytes + length + JSON
        packet.extend_from_slice(&(payload_len as u16).to_be_bytes());
        packet.extend_from_slice(&PayloadType::Json.to_bytes());
        
        // C-Bytes
        packet.extend_from_slice(&[0x6A, 0x00, 0x65, 0x00]);
        // JSON length (LE)
        packet.extend_from_slice(&(json.len() as u32).to_le_bytes());
        // JSON
        packet.extend_from_slice(json.as_bytes());
        
        let parsed = parse_incoming_packet(&packet).unwrap();
        
        if let IncomingPacket::Json(s) = parsed {
            assert_eq!(s, json);
        } else {
            panic!("Expected Json");
        }
    }

    #[test]
    fn test_parse_metering() {
        // Build an MS packet with some meter values
        let mut packet = Vec::new();
        packet.extend_from_slice(&MAGIC_BYTES);
        
        let levels: [f32; 4] = [0.5, 0.75, 0.25, 1.0];
        let payload_len = 4 + levels.len() * 4; // C-Bytes + levels
        
        packet.extend_from_slice(&(payload_len as u16).to_be_bytes());
        packet.extend_from_slice(&PayloadType::MeterStatus.to_bytes());
        
        // C-Bytes
        packet.extend_from_slice(&[0x6A, 0x00, 0x65, 0x00]);
        // Levels
        for level in &levels {
            packet.extend_from_slice(&level.to_le_bytes());
        }
        
        let parsed = parse_incoming_packet(&packet).unwrap();
        
        if let IncomingPacket::Metering(parsed_levels) = parsed {
            assert_eq!(parsed_levels.len(), 4);
            assert!((parsed_levels[0] - 0.5).abs() < 0.001);
            assert!((parsed_levels[1] - 0.75).abs() < 0.001);
        } else {
            panic!("Expected Metering");
        }
    }

    #[test]
    fn test_parse_packet_stream() {
        // Build multiple packets concatenated
        let ka1 = build_keepalive_packet();
        let ka2 = build_keepalive_packet();
        
        let mut stream = Vec::new();
        stream.extend_from_slice(&ka1);
        stream.extend_from_slice(&ka2);
        
        let packets = parse_packet_stream(&stream);
        assert_eq!(packets.len(), 2);
        
        for result in packets {
            assert!(matches!(result.unwrap(), IncomingPacket::KeepAlive));
        }
    }

    #[test]
    fn test_usb_supported_mixer() {
        assert!(usb::is_supported_mixer(usb::PRESONUS_VENDOR_ID, usb::product_ids::STUDIOLIVE_32SX));
        assert!(usb::is_supported_mixer(usb::PRESONUS_VENDOR_ID, usb::product_ids::STUDIOLIVE_64S));
        assert!(!usb::is_supported_mixer(0x1234, usb::product_ids::STUDIOLIVE_32SX));
        assert!(!usb::is_supported_mixer(usb::PRESONUS_VENDOR_ID, 0x9999));
    }

    #[test]
    fn test_usb_model_name() {
        assert_eq!(usb::get_model_name(usb::product_ids::STUDIOLIVE_32SX), Some("StudioLive 32SX"));
        assert_eq!(usb::get_model_name(usb::product_ids::STUDIOLIVE_64S), Some("StudioLive 64S"));
        assert_eq!(usb::get_model_name(0x9999), None);
    }

    #[test]
    fn test_usb_packet_buffer_single() {
        let mut buffer = UsbPacketBuffer::new();
        
        // Add a complete packet
        let packet = build_keepalive_packet();
        let complete = buffer.add_data(&packet);
        
        assert_eq!(complete.len(), 1);
        assert_eq!(complete[0], packet);
        assert!(!buffer.has_pending());
    }

    #[test]
    fn test_usb_packet_buffer_fragmented() {
        let mut buffer = UsbPacketBuffer::new();
        
        // Build a packet and split it
        let packet = build_keepalive_packet();
        let (first, second) = packet.split_at(4);
        
        // Add first fragment
        let complete = buffer.add_data(first);
        assert!(complete.is_empty());
        assert!(buffer.has_pending());
        
        // Add second fragment
        let complete = buffer.add_data(second);
        assert_eq!(complete.len(), 1);
        assert_eq!(complete[0], packet);
    }

    #[test]
    fn test_usb_packet_buffer_multiple() {
        let mut buffer = UsbPacketBuffer::new();
        
        // Add two complete packets at once
        let packet1 = build_keepalive_packet();
        let packet2 = build_hello_packet();
        
        let mut combined = packet1.clone();
        combined.extend_from_slice(&packet2);
        
        let complete = buffer.add_data(&combined);
        assert_eq!(complete.len(), 2);
        assert_eq!(complete[0], packet1);
        assert_eq!(complete[1], packet2);
    }

    #[test]
    fn test_usb_packet_buffer_garbage_prefix() {
        let mut buffer = UsbPacketBuffer::new();
        
        // Add garbage followed by a valid packet
        let packet = build_keepalive_packet();
        let mut data = vec![0x00, 0x01, 0x02, 0x03]; // garbage
        data.extend_from_slice(&packet);
        
        let complete = buffer.add_data(&data);
        assert_eq!(complete.len(), 1);
        assert_eq!(complete[0], packet);
    }
}
