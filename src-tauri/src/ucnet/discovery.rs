//! UCNet device discovery implementation
//!
//! This module handles automatic discovery of PreSonus UCNet devices on both
//! network (UDP broadcast) and USB connections.

use super::error::{Result, UcNetError};
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
            
            // Check if this is a PreSonus device
            if device_desc.vendor_id() == PRESONUS_VENDOR_ID {
                debug!(
                    "Found PreSonus device: VID=0x{:04x}, PID=0x{:04x}",
                    device_desc.vendor_id(),
                    device_desc.product_id()
                );
                
                match extract_usb_device_info(&device, &device_desc) {
                    Ok(device_info) => {
                        info!("Discovered USB device: {}", device_info.model);
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

/// Creates a UCNet discovery broadcast packet
///
/// The actual packet format would be documented in the UCNet protocol specification.
/// For now, this is a placeholder that sends a simple discovery request.
fn create_discovery_packet() -> Vec<u8> {
    // TODO: Implement actual UCNet discovery packet format
    // This is a placeholder implementation
    // Real implementation would follow PreSonus UCNet protocol specification
    
    // Magic bytes for UCNet discovery (placeholder)
    let mut packet = Vec::new();
    packet.extend_from_slice(b"UCNET_DISCOVER");
    packet.push(0x01); // Protocol version
    packet
}

/// Parses a discovery response from a UCNet device
///
/// # Arguments
/// * `data` - Raw response data from the device
/// * `ip_addr` - IP address the response came from
fn parse_discovery_response(data: &[u8], ip_addr: IpAddr) -> Result<NetworkDeviceInfo> {
    // TODO: Implement actual UCNet response parsing
    // This is a placeholder implementation
    
    if data.len() < 16 {
        return Err(UcNetError::InvalidResponse(
            "Response too short".to_string()
        ));
    }
    
    // Placeholder parsing - real implementation would parse actual UCNet protocol
    // For now, return a mock device for testing
    Ok(NetworkDeviceInfo {
        ip_addr,
        port: UCNET_DISCOVERY_PORT,
        model: "StudioLive 32SX".to_string(), // Would be parsed from response
        firmware_version: "1.0.0".to_string(), // Would be parsed from response
        device_id: format!("{}", ip_addr),
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
        assert!(packet.starts_with(b"UCNET_DISCOVER"));
    }
    
    #[test]
    fn test_parse_discovery_response_too_short() {
        let data = vec![0u8; 8];
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
        let result = parse_discovery_response(&data, ip);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_parse_discovery_response_valid() {
        let data = vec![0u8; 32]; // Valid length
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
        let result = parse_discovery_response(&data, ip);
        assert!(result.is_ok());
        
        let device_info = result.unwrap();
        assert_eq!(device_info.ip_addr, ip);
        assert_eq!(device_info.port, UCNET_DISCOVERY_PORT);
    }
}
