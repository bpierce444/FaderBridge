//! UCNet connection management
//!
//! Handles connection state, keep-alive heartbeats, and communication with UCNet devices.
//!
//! ## Protocol Overview
//! - TCP port 53000 for control communication
//! - Hello packet sent first, then Subscribe with JSON payload
//! - Keep-alive (KA) packets sent every 3-5 seconds
//! - Parameter changes sent via PS (Parameter Set) packets

use super::error::{Result, UcNetError};
use super::protocol::{
    build_hello_packet, build_keepalive_packet, build_parameter_set_bool_packet,
    build_parameter_set_packet, build_subscribe_packet, keys, PacketHeader, PayloadType,
    SubscribeRequest, CONTROL_PORT, MAGIC_BYTES,
};
use super::types::{constants::*, ConnectionState, ConnectionType, UcNetDevice};
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;

/// Connection event types
#[derive(Debug, Clone)]
pub enum ConnectionEvent {
    /// Device connected successfully
    Connected(String),
    /// Device disconnected
    Disconnected(String),
    /// Connection failed
    Failed(String, String), // device_id, reason
    /// Keep-alive sent
    KeepaliveSent(String),
    /// Keep-alive timeout
    KeepaliveTimeout(String),
}

/// Manages connections to UCNet devices
pub struct ConnectionManager {
    /// Map of device ID to connection state
    connections: Arc<RwLock<HashMap<String, DeviceConnection>>>,
    /// Event channel for connection events
    event_tx: mpsc::UnboundedSender<ConnectionEvent>,
    /// Event receiver (for consumers)
    event_rx: Arc<RwLock<mpsc::UnboundedReceiver<ConnectionEvent>>>,
    /// Flag to track if keepalive task has been started
    keepalive_started: Arc<RwLock<bool>>,
}

/// Represents a single device connection
struct DeviceConnection {
    /// Device information
    device: UcNetDevice,
    /// Last keep-alive timestamp
    last_keepalive: Instant,
    /// Connection-specific data (socket, handle, etc.)
    connection_data: ConnectionData,
}

/// Connection-specific data based on connection type
enum ConnectionData {
    /// Network connection data
    Network {
        /// Socket address for communication
        addr: std::net::SocketAddr,
        /// TCP stream for communication (wrapped in Arc<RwLock> for sharing)
        stream: Arc<RwLock<Option<TcpStream>>>,
    },
    /// USB connection data
    Usb {
        /// USB device handle
        device_path: String,
    },
}

impl ConnectionManager {
    /// Creates a new connection manager
    pub fn new() -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            event_rx: Arc::new(RwLock::new(event_rx)),
            keepalive_started: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Connects to a device
    ///
    /// # Arguments
    /// * `device` - Device to connect to
    pub async fn connect(&self, mut device: UcNetDevice) -> Result<()> {
        let device_id = device.id.clone();
        
        // Check if already connected
        {
            let connections = self.connections.read().await;
            if connections.contains_key(&device_id) {
                return Err(UcNetError::AlreadyConnected(device_id));
            }
        }
        
        info!("Connecting to device: {} ({})", device.model, device_id);
        device.state = ConnectionState::Connecting;
        
        // Establish connection based on type
        let connection_data = match device.connection_type {
            ConnectionType::Network => self.connect_network(&device).await?,
            ConnectionType::Usb => self.connect_usb(&device).await?,
        };
        
        device.state = ConnectionState::Connected;
        
        // Store connection
        let connection = DeviceConnection {
            device: device.clone(),
            last_keepalive: Instant::now(),
            connection_data,
        };
        
        {
            let mut connections = self.connections.write().await;
            connections.insert(device_id.clone(), connection);
        }
        
        // Start keepalive task on first connection (lazy initialization)
        {
            let mut started = self.keepalive_started.write().await;
            if !*started {
                info!("Starting keepalive task");
                let manager = Arc::new(Self {
                    connections: Arc::clone(&self.connections),
                    event_tx: self.event_tx.clone(),
                    event_rx: Arc::clone(&self.event_rx),
                    keepalive_started: Arc::clone(&self.keepalive_started),
                });
                manager.start_keepalive_task();
                *started = true;
            }
        }
        
        // Notify connection event
        let _ = self.event_tx.send(ConnectionEvent::Connected(device_id.clone()));
        
        info!("Successfully connected to device: {}", device_id);
        Ok(())
    }
    
    /// Disconnects from a device
    ///
    /// # Arguments
    /// * `device_id` - ID of the device to disconnect
    pub async fn disconnect(&self, device_id: &str) -> Result<()> {
        info!("Disconnecting from device: {}", device_id);
        
        let mut connections = self.connections.write().await;
        
        if let Some(mut connection) = connections.remove(device_id) {
            connection.device.state = ConnectionState::Disconnected;
            
            // Perform cleanup based on connection type
            match &connection.connection_data {
                ConnectionData::Network { addr, stream } => {
                    // Close the TCP stream
                    let mut stream_guard = stream.write().await;
                    if let Some(tcp_stream) = stream_guard.take() {
                        // Dropping the stream will close the connection
                        drop(tcp_stream);
                    }
                    debug!("Closed network connection for {} at {}", device_id, addr);
                }
                ConnectionData::Usb { device_path } => {
                    // Close USB handle
                    debug!("Closing USB connection for {} at {}", device_id, device_path);
                }
            }
            
            let _ = self.event_tx.send(ConnectionEvent::Disconnected(device_id.to_string()));
            Ok(())
        } else {
            Err(UcNetError::DeviceNotFound(device_id.to_string()))
        }
    }
    
    /// Gets the current state of a device connection
    pub async fn get_device_state(&self, device_id: &str) -> Option<ConnectionState> {
        let connections = self.connections.read().await;
        connections.get(device_id).map(|c| c.device.state)
    }
    
    /// Gets all connected devices
    pub async fn get_connected_devices(&self) -> Vec<UcNetDevice> {
        let connections = self.connections.read().await;
        connections.values().map(|c| c.device.clone()).collect()
    }
    
    /// Starts the keep-alive task
    ///
    /// This task runs in the background and sends keep-alive packets to all
    /// connected devices every KEEPALIVE_INTERVAL_SECS seconds.
    pub fn start_keepalive_task(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(KEEPALIVE_INTERVAL_SECS));
            
            loop {
                interval.tick().await;
                
                if let Err(e) = self.send_keepalives().await {
                    error!("Error sending keep-alives: {}", e);
                }
            }
        });
    }
    
    /// Sends keep-alive packets to all connected devices
    async fn send_keepalives(&self) -> Result<()> {
        let mut connections = self.connections.write().await;
        let now = Instant::now();
        let mut disconnected_devices = Vec::new();
        
        for (device_id, connection) in connections.iter_mut() {
            // Check if connection has timed out
            let elapsed = now.duration_since(connection.last_keepalive);
            
            if elapsed > Duration::from_secs(CONNECTION_TIMEOUT_SECS) {
                warn!("Connection timeout for device: {}", device_id);
                disconnected_devices.push(device_id.clone());
                let _ = self.event_tx.send(ConnectionEvent::KeepaliveTimeout(device_id.clone()));
                continue;
            }
            
            // Send keep-alive packet
            match self.send_keepalive_packet(connection).await {
                Ok(()) => {
                    connection.last_keepalive = now;
                    debug!("Keep-alive sent to device: {}", device_id);
                    let _ = self.event_tx.send(ConnectionEvent::KeepaliveSent(device_id.clone()));
                }
                Err(e) => {
                    error!("Failed to send keep-alive to {}: {}", device_id, e);
                    disconnected_devices.push(device_id.clone());
                }
            }
        }
        
        // Remove disconnected devices
        for device_id in disconnected_devices {
            if let Some(mut connection) = connections.remove(&device_id) {
                connection.device.state = ConnectionState::Disconnected;
                let _ = self.event_tx.send(ConnectionEvent::Disconnected(device_id));
            }
        }
        
        Ok(())
    }
    
    /// Sends a keep-alive packet to a specific device
    async fn send_keepalive_packet(&self, connection: &DeviceConnection) -> Result<()> {
        match &connection.connection_data {
            ConnectionData::Network { addr, stream } => {
                let keepalive_packet = build_keepalive_packet();
                
                let mut stream_guard = stream.write().await;
                if let Some(ref mut tcp_stream) = *stream_guard {
                    tcp_stream.write_all(&keepalive_packet).await.map_err(|e| {
                        UcNetError::Connection(format!("Failed to send keep-alive to {}: {}", addr, e))
                    })?;
                    debug!("Sent keep-alive packet to {}", addr);
                    Ok(())
                } else {
                    Err(UcNetError::Connection("TCP stream not available".to_string()))
                }
            }
            ConnectionData::Usb { device_path } => {
                // USB keep-alive would use USB bulk transfer
                // For now, USB connections don't require keep-alive in the same way
                debug!("USB keep-alive not required for {}", device_path);
                Ok(())
            }
        }
    }
    
    /// Establishes a network connection to a device
    ///
    /// Connection handshake sequence:
    /// 1. Open TCP connection to port 53000
    /// 2. Send Hello packet (UM)
    /// 3. Send Subscribe packet (JM) with client info
    /// 4. Receive SubscriptionReply and initial state
    async fn connect_network(&self, device: &UcNetDevice) -> Result<ConnectionData> {
        // Parse IP address from identifier
        let ip_addr: std::net::IpAddr = device.identifier.parse()
            .map_err(|_| UcNetError::Protocol("Invalid IP address".to_string()))?;
        
        let addr = std::net::SocketAddr::new(ip_addr, CONTROL_PORT);
        
        info!("Connecting to UCNet device at {}", addr);
        
        // Step 1: Open TCP connection
        let mut stream = TcpStream::connect(addr).await.map_err(|e| {
            UcNetError::Connection(format!("Failed to connect to {}: {}", addr, e))
        })?;
        
        debug!("TCP connection established to {}", addr);
        
        // Step 2: Send Hello packet
        let hello_packet = build_hello_packet();
        stream.write_all(&hello_packet).await.map_err(|e| {
            UcNetError::Connection(format!("Failed to send Hello packet: {}", e))
        })?;
        debug!("Sent Hello packet");
        
        // Step 3: Send Subscribe packet
        let subscribe_request = SubscribeRequest::default();
        let subscribe_packet = build_subscribe_packet(&subscribe_request)?;
        stream.write_all(&subscribe_packet).await.map_err(|e| {
            UcNetError::Connection(format!("Failed to send Subscribe packet: {}", e))
        })?;
        debug!("Sent Subscribe packet");
        
        // Step 4: Read response (SubscriptionReply)
        // We expect at least a header (8 bytes) in response
        let mut response_buf = [0u8; 1024];
        let timeout_duration = Duration::from_secs(5);
        
        let read_result = tokio::time::timeout(
            timeout_duration,
            stream.read(&mut response_buf)
        ).await;
        
        match read_result {
            Ok(Ok(n)) if n >= 8 => {
                // Verify we got a valid UCNet response
                if response_buf[0..4] == MAGIC_BYTES {
                    let header = PacketHeader::from_bytes(&response_buf[..n])?;
                    info!(
                        "Received response: {:?}, size={}",
                        header.payload_type, header.size
                    );
                    
                    // Connection successful
                    info!("Successfully connected to UCNet device at {}", addr);
                } else {
                    warn!("Received non-UCNet response from {}", addr);
                }
            }
            Ok(Ok(n)) => {
                warn!("Received short response ({} bytes) from {}", n, addr);
            }
            Ok(Err(e)) => {
                warn!("Error reading response from {}: {}", addr, e);
            }
            Err(_) => {
                warn!("Timeout waiting for response from {}", addr);
            }
        }
        
        Ok(ConnectionData::Network {
            addr,
            stream: Arc::new(RwLock::new(Some(stream))),
        })
    }
    
    /// Establishes a USB connection to a device
    async fn connect_usb(&self, device: &UcNetDevice) -> Result<ConnectionData> {
        let device_path = device.identifier.clone();
        
        // USB UCNet communication uses bulk transfers
        // This requires opening the USB device and finding the correct endpoints
        debug!("Establishing USB connection to {}", device_path);
        
        // Note: Full USB implementation would require:
        // 1. Open USB device handle
        // 2. Claim interface
        // 3. Find bulk IN/OUT endpoints
        // 4. Send Hello/Subscribe packets via bulk OUT
        // 5. Read responses via bulk IN
        
        Ok(ConnectionData::Usb { device_path })
    }
    
    /// Sends a parameter value to a connected device
    ///
    /// # Arguments
    /// * `device_id` - ID of the device to send to
    /// * `key` - Parameter key (e.g., "line.ch1.volume")
    /// * `value` - Parameter value (0.0 to 1.0 for faders, etc.)
    pub async fn send_parameter(&self, device_id: &str, key: &str, value: f32) -> Result<()> {
        let connections = self.connections.read().await;
        
        let connection = connections.get(device_id).ok_or_else(|| {
            UcNetError::DeviceNotFound(device_id.to_string())
        })?;
        
        match &connection.connection_data {
            ConnectionData::Network { addr, stream } => {
                let packet = build_parameter_set_packet(key, value);
                
                let mut stream_guard = stream.write().await;
                if let Some(ref mut tcp_stream) = *stream_guard {
                    tcp_stream.write_all(&packet).await.map_err(|e| {
                        UcNetError::Connection(format!(
                            "Failed to send parameter to {}: {}",
                            addr, e
                        ))
                    })?;
                    debug!("Sent parameter {}={} to {}", key, value, addr);
                    Ok(())
                } else {
                    Err(UcNetError::Connection("TCP stream not available".to_string()))
                }
            }
            ConnectionData::Usb { device_path } => {
                // USB parameter sending would use bulk transfer
                debug!("USB parameter send not yet implemented for {}", device_path);
                Err(UcNetError::Protocol("USB parameter send not implemented".to_string()))
            }
        }
    }
    
    /// Sends a boolean parameter value to a connected device (e.g., mute, solo)
    ///
    /// # Arguments
    /// * `device_id` - ID of the device to send to
    /// * `key` - Parameter key (e.g., "line.ch1.mute")
    /// * `value` - Boolean value
    pub async fn send_parameter_bool(&self, device_id: &str, key: &str, value: bool) -> Result<()> {
        let connections = self.connections.read().await;
        
        let connection = connections.get(device_id).ok_or_else(|| {
            UcNetError::DeviceNotFound(device_id.to_string())
        })?;
        
        match &connection.connection_data {
            ConnectionData::Network { addr, stream } => {
                let packet = build_parameter_set_bool_packet(key, value);
                
                let mut stream_guard = stream.write().await;
                if let Some(ref mut tcp_stream) = *stream_guard {
                    tcp_stream.write_all(&packet).await.map_err(|e| {
                        UcNetError::Connection(format!(
                            "Failed to send parameter to {}: {}",
                            addr, e
                        ))
                    })?;
                    debug!("Sent parameter {}={} to {}", key, value, addr);
                    Ok(())
                } else {
                    Err(UcNetError::Connection("TCP stream not available".to_string()))
                }
            }
            ConnectionData::Usb { device_path } => {
                debug!("USB parameter send not yet implemented for {}", device_path);
                Err(UcNetError::Protocol("USB parameter send not implemented".to_string()))
            }
        }
    }
    
    /// Convenience method to set channel volume
    pub async fn set_channel_volume(&self, device_id: &str, channel: u8, volume: f32) -> Result<()> {
        let key = keys::channel_volume(channel);
        self.send_parameter(device_id, &key, volume).await
    }
    
    /// Convenience method to set channel mute
    pub async fn set_channel_mute(&self, device_id: &str, channel: u8, muted: bool) -> Result<()> {
        let key = keys::channel_mute(channel);
        self.send_parameter_bool(device_id, &key, muted).await
    }
    
    /// Convenience method to set channel pan
    pub async fn set_channel_pan(&self, device_id: &str, channel: u8, pan: f32) -> Result<()> {
        let key = keys::channel_pan(channel);
        self.send_parameter(device_id, &key, pan).await
    }
    
    /// Convenience method to set main volume
    pub async fn set_main_volume(&self, device_id: &str, volume: f32) -> Result<()> {
        let key = keys::main_volume();
        self.send_parameter(device_id, &key, volume).await
    }
    
    /// Gets a receiver for connection events
    pub fn subscribe_events(&self) -> Arc<RwLock<mpsc::UnboundedReceiver<ConnectionEvent>>> {
        Arc::clone(&self.event_rx)
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ucnet::types::NetworkDeviceInfo;
    
    #[tokio::test]
    async fn test_connection_manager_creation() {
        let manager = ConnectionManager::new();
        let devices = manager.get_connected_devices().await;
        assert_eq!(devices.len(), 0);
    }
    
    #[tokio::test]
    async fn test_connect_already_connected() {
        let manager = ConnectionManager::new();
        
        let device = UcNetDevice::from_network(NetworkDeviceInfo {
            ip_addr: "192.168.1.100".parse().unwrap(),
            port: UCNET_DISCOVERY_PORT,
            model: "Test Device".to_string(),
            firmware_version: "1.0.0".to_string(),
            device_id: "test-001".to_string(),
        });
        
        // First connection should succeed
        let result1 = manager.connect(device.clone()).await;
        assert!(result1.is_ok());
        
        // Second connection should fail
        let result2 = manager.connect(device).await;
        assert!(result2.is_err());
        assert!(matches!(result2.unwrap_err(), UcNetError::AlreadyConnected(_)));
    }
    
    #[tokio::test]
    async fn test_disconnect_nonexistent() {
        let manager = ConnectionManager::new();
        let result = manager.disconnect("nonexistent").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UcNetError::DeviceNotFound(_)));
    }
}
