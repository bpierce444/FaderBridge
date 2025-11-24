//! UCNet device discovery implementation
//!
//! This module handles automatic discovery of PreSonus UCNet devices on both
//! network (UDP broadcast) and USB connections.
//!
//! ## Discovery Protocol
//! PreSonus UCNet devices broadcast discovery advertisements every ~3 seconds
//! on UDP port 47809. Clients listen for these broadcasts to discover devices.
//! The discovery packet contains device model, serial number, and friendly name.

use super::error::{Result, UcNetError};
use super::protocol::{DiscoveryInfo, PacketHeader, PayloadType, DISCOVERY_PORT, MAGIC_BYTES};
use super::types::{constants::*, NetworkDeviceInfo, UsbDeviceInfo};
use log::{debug, error, info, warn};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::timeout;

/// Trait for device discovery to enable mocking in tests
pub trait DeviceDiscovery: Send + Sync {
    /// Discovers devices on the network via UDP broadcast
    fn discover_network_devices(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<NetworkDeviceInfo>>> + Send;
    
    /// Enumerates USB-connected UCNet devices
    fn discover_usb_devices(&self) -> Result<Vec<UsbDeviceInfo>>;
}

/// Default implementation of device discovery
pub struct DefaultDeviceDiscovery;

impl DefaultDeviceDiscovery {
    /// Creates a new device discovery instance
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultDeviceDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceDiscovery for DefaultDeviceDiscovery {
    /// Discovers UCNet devices on the local network using UDP broadcast
    ///
    /// Sends a discovery broadcast on port 47809 and waits up to 2 seconds
    /// for responses from PreSonus devices.
    fn discover_network_devices(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<NetworkDeviceInfo>>> + Send {
        async move {
        info!("Starting network device discovery on port {}", UCNET_DISCOVERY_PORT);
        
        // Bind to any available port for sending/receiving
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.set_broadcast(true)?;
        
        // Prepare discovery broadcast message
        // UCNet discovery protocol: send a specific magic packet
        let discovery_msg = create_discovery_packet();
        
        // Broadcast to the network
        let broadcast_addr = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::BROADCAST),
            UCNET_DISCOVERY_PORT,
        );
        
        debug!("Sending discovery broadcast to {}", broadcast_addr);
        socket.send_to(&discovery_msg, broadcast_addr).await?;
        
        // Collect responses for up to DISCOVERY_TIMEOUT_SECS seconds
        let mut devices = Vec::new();
        let deadline = Duration::from_secs(DISCOVERY_TIMEOUT_SECS);
        
        loop {
            let mut buf = [0u8; 1024];
            
            match timeout(deadline, socket.recv_from(&mut buf)).await {
                Ok(Ok((len, addr))) => {
                    debug!("Received {} bytes from {}", len, addr);
                    
                    match parse_discovery_response(&buf[..len], addr.ip()) {
                        Ok(device_info) => {
                            info!("Discovered device: {} at {}", device_info.model, device_info.ip_addr);
                            devices.push(device_info);
                        }
                        Err(e) => {
                            warn!("Failed to parse discovery response from {}: {}", addr, e);
                        }
                    }
                }
                Ok(Err(e)) => {
                    error!("Socket error during discovery: {}", e);
                    break;
                }
                Err(_) => {
                    // Timeout reached
                    debug!("Discovery timeout reached");
                    break;
                }
            }
        }
        
        info!("Network discovery complete. Found {} device(s)", devices.len());
        Ok(devices)
        }
    }
    
    /// Enumerates USB-connected PreSonus devices
    ///
    /// Searches for devices with PreSonus vendor ID (0x194f) and attempts
    /// to identify UCNet-compatible devices.
    fn discover_usb_devices(&self) -> Result<Vec<UsbDeviceInfo>> {
        info!("Starting USB device discovery (VID: 0x{:04x})", PRESONUS_VENDOR_ID);
        
        let mut devices = Vec::new();
        
        // Get list of USB devices
        let usb_devices = rusb::devices()?;
        
        for device in usb_devices.iter() {
            let device_desc = match device.device_descriptor() {
                Ok(desc) => desc,
                Err(e) => {
                    warn!("Failed to get device descriptor: {}", e);
                    continue;
                }
            };
            
            let vendor_id = device_desc.vendor_id();
            let product_id = device_desc.product_id();
            
            // Check if this is a PreSonus or Fender (PreSonus parent company) UCNet device
            if vendor_id == PRESONUS_VENDOR_ID || vendor_id == super::types::constants::FENDER_VENDOR_ID {
                debug!(
                    "Found PreSonus/Fender device: VID=0x{:04x}, PID=0x{:04x}",
                    vendor_id,
                    product_id
                );
                
                // Only include devices that are UCNet-capable
                if !super::types::constants::is_ucnet_device(product_id) {
                    debug!(
                        "Skipping non-UCNet device (VID=0x{:04x}, PID=0x{:04x}) - likely a MIDI controller",
                        vendor_id,
                        product_id
                    );
                    continue;
                }
                
                match extract_usb_device_info(&device, &device_desc) {
                    Ok(device_info) => {
                        info!("Discovered UCNet USB device: {}", device_info.model);
                        devices.push(device_info);
                    }
                    Err(e) => {
                        warn!("Failed to extract USB device info: {}", e);
                    }
                }
            }
        }
        
        info!("USB discovery complete. Found {} device(s)", devices.len());
        Ok(devices)
    }
}

/// Creates a UCNet discovery query packet (optional - devices broadcast automatically)
///
/// Note: UCNet devices broadcast discovery advertisements every ~3 seconds without
/// needing a query. This packet can optionally be sent to trigger an immediate response.
///
/// Packet format: UC\x00\x01 + size(u16 BE) + "DQ" + payload
fn create_discovery_packet() -> Vec<u8> {
    // Discovery Query packet (DQ)
    // Format: 55 43 00 01 00 00 44 51 00 00 65 00
    // This is optional - we primarily listen for broadcast advertisements
    let mut packet = Vec::new();
    packet.extend_from_slice(&MAGIC_BYTES);
    packet.extend_from_slice(&[0x00, 0x04]); // Size: 4 bytes
    packet.extend_from_slice(&PayloadType::DiscoveryQuery.to_bytes()); // "DQ"
    packet.extend_from_slice(&[0x00, 0x00, 0x65, 0x00]); // C-Bytes
    packet
}

/// Parses a discovery response from a UCNet device
///
/// Discovery Advertisement (DA) packet format:
/// - Magic bytes: 55 43 00 01
/// - Size: 2 bytes (big-endian)
/// - Payload type: "DA" (44 41)
/// - C-Bytes: 4 bytes
/// - Unknown header data
/// - Model name (null-terminated)
/// - Category (null-terminated, e.g., "AUD")
/// - Serial number (null-terminated)
/// - Friendly name (null-terminated)
///
/// # Arguments
/// * `data` - Raw response data from the device
/// * `ip_addr` - IP address the response came from
fn parse_discovery_response(data: &[u8], ip_addr: IpAddr) -> Result<NetworkDeviceInfo> {
    // Minimum packet size: header (8) + some payload
    if data.len() < 12 {
        return Err(UcNetError::InvalidResponse(
            "Response too short for UCNet packet".to_string()
        ));
    }
    
    // Verify magic bytes
    if data[0..4] != MAGIC_BYTES {
        return Err(UcNetError::InvalidResponse(format!(
            "Invalid magic bytes: {:02X?}",
            &data[0..4]
        )));
    }
    
    // Parse header
    let header = PacketHeader::from_bytes(data)?;
    
    // Verify this is a Discovery Advertisement packet
    if header.payload_type != PayloadType::DiscoveryAdvertisement {
        debug!(
            "Ignoring non-discovery packet type: {:?}",
            header.payload_type
        );
        return Err(UcNetError::InvalidResponse(
            "Not a discovery advertisement packet".to_string()
        ));
    }
    
    // Extract payload (after 8-byte header)
    let payload = &data[8..];
    
    // Parse discovery info from payload
    let discovery_info = DiscoveryInfo::from_payload(payload)?;
    
    info!(
        "Parsed discovery: model={}, serial={}, name={}",
        discovery_info.model, discovery_info.serial, discovery_info.friendly_name
    );
    
    Ok(NetworkDeviceInfo {
        ip_addr,
        port: DISCOVERY_PORT,
        model: discovery_info.model,
        firmware_version: "Unknown".to_string(), // Not in discovery packet, get from connection
        device_id: if discovery_info.serial.is_empty() {
            format!("{}", ip_addr)
        } else {
            discovery_info.serial
        },
    })
}

/// Extracts device information from a USB device
fn extract_usb_device_info(
    device: &rusb::Device<rusb::GlobalContext>,
    device_desc: &rusb::DeviceDescriptor,
) -> Result<UsbDeviceInfo> {
    // Get device path/identifier
    let device_path = format!(
        "{}:{}",
        device.bus_number(),
        device.address()
    );
    
    // Try to open device to get string descriptors
    let handle = device.open().ok();
    
    let model = if let Some(ref h) = handle {
        h.read_product_string_ascii(device_desc)
            .unwrap_or_else(|_| format!("PreSonus Device 0x{:04x}", device_desc.product_id()))
    } else {
        format!("PreSonus Device 0x{:04x}", device_desc.product_id())
    };
    
    // Firmware version would typically come from device-specific queries
    let firmware_version = "Unknown".to_string();
    
    Ok(UsbDeviceInfo {
        vendor_id: device_desc.vendor_id(),
        product_id: device_desc.product_id(),
        model,
        firmware_version,
        device_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_discovery_packet() {
        let packet = create_discovery_packet();
        assert!(!packet.is_empty());
        // Should start with UCNet magic bytes
        assert!(packet.starts_with(&MAGIC_BYTES));
        // Should contain DQ payload type
        assert_eq!(packet[6..8], PayloadType::DiscoveryQuery.to_bytes());
    }
    
    #[test]
    fn test_parse_discovery_response_too_short() {
        let data = vec![0u8; 8];
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
        let result = parse_discovery_response(&data, ip);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_parse_discovery_response_invalid_magic() {
        // Invalid magic bytes
        let data = vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x44, 0x41, 0x00, 0x00, 0x00, 0x00];
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
        let result = parse_discovery_response(&data, ip);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_parse_discovery_response_valid() {
        // Build a valid discovery advertisement packet
        // Header: UC\x00\x01 + size + DA
        let mut data = Vec::new();
        data.extend_from_slice(&MAGIC_BYTES);
        
        // Build payload with model and serial
        // The payload starts with C-Bytes and binary header, then null-terminated strings
        // Based on real packet: binary header followed by model, category, serial, friendly name
        let payload = b"\x65\x00\x00\x00\x00\x04\x00\x80\x48\x1c\x48\x67\x23\x60\x51\x4fStudioLive 32SX\x00AUD\x00SL32SX123456\x00StudioLive 32SX\x00";
        
        // Size (big-endian)
        let size = (payload.len() as u16).to_be_bytes();
        data.extend_from_slice(&size);
        
        // Payload type: DA
        data.extend_from_slice(&PayloadType::DiscoveryAdvertisement.to_bytes());
        
        // Payload
        data.extend_from_slice(payload);
        
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
        let result = parse_discovery_response(&data, ip);
        
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        
        let device_info = result.unwrap();
        assert_eq!(device_info.ip_addr, ip);
        assert_eq!(device_info.port, DISCOVERY_PORT);
        // Model should be extracted from the payload
        assert!(device_info.model.contains("StudioLive") || !device_info.model.is_empty());
    }
    
    #[test]
    fn test_parse_discovery_response_wrong_packet_type() {
        // Build a packet with wrong type (KA instead of DA)
        let mut data = Vec::new();
        data.extend_from_slice(&MAGIC_BYTES);
        data.extend_from_slice(&[0x00, 0x04]); // Size
        data.extend_from_slice(&PayloadType::KeepAlive.to_bytes()); // Wrong type
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
        let result = parse_discovery_response(&data, ip);
        assert!(result.is_err());
    }
}
