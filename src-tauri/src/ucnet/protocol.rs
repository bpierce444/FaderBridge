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
use serde::{Deserialize, Serialize};

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
    /// Create new C-Bytes with default values (j, e)
    pub fn new() -> Self {
        Self { a: 0x6A, b: 0x65 } // 'j', 'e'
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

        let size = u16::from_be_bytes([bytes[4], bytes[5]]);
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
        bytes.extend_from_slice(&self.size.to_be_bytes());
        bytes.extend_from_slice(&self.payload_type.to_bytes());
        bytes
    }

    /// Create a new header with the given payload type and size
    pub fn new(payload_type: PayloadType, payload_size: u16) -> Self {
        Self {
            magic: MAGIC_BYTES,
            size: payload_size,
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
            client_name: "FaderBridge".to_string(),
            client_internal_name: "faderbridge".to_string(),
            client_type: "PC".to_string(),
            client_description: "FaderBridge MIDI Controller".to_string(),
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

/// Build a Parameter Set packet
pub fn build_parameter_set_packet(key: &str, value: f32) -> Vec<u8> {
    let payload = ParameterValue::to_set_payload(key, value, CBytes::new());
    build_packet(PayloadType::ParameterSet, &payload)
}

/// Build a Parameter Set packet for boolean values
pub fn build_parameter_set_bool_packet(key: &str, value: bool) -> Vec<u8> {
    let payload = ParameterValue::to_set_bool_payload(key, value, CBytes::new());
    build_packet(PayloadType::ParameterSet, &payload)
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
}
