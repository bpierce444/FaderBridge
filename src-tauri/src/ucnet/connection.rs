//! UCNet connection management
//!
//! Handles connection state, keep-alive heartbeats, and communication with UCNet devices.

use super::error::{Result, UcNetError};
use super::types::{constants::*, ConnectionState, ConnectionType, UcNetDevice};
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
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
            match connection.connection_data {
                ConnectionData::Network { .. } => {
                    // Send disconnect packet if needed
                    debug!("Closing network connection for {}", device_id);
                }
                ConnectionData::Usb { .. } => {
                    // Close USB handle
                    debug!("Closing USB connection for {}", device_id);
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
            ConnectionData::Network { addr } => {
                // TODO: Implement actual keep-alive packet sending
                debug!("Sending network keep-alive to {}", addr);
                Ok(())
            }
            ConnectionData::Usb { device_path } => {
                // TODO: Implement USB keep-alive
                debug!("Sending USB keep-alive to {}", device_path);
                Ok(())
            }
        }
    }
    
    /// Establishes a network connection to a device
    async fn connect_network(&self, device: &UcNetDevice) -> Result<ConnectionData> {
        // Parse IP address from identifier
        let ip_addr: std::net::IpAddr = device.identifier.parse()
            .map_err(|_| UcNetError::Protocol("Invalid IP address".to_string()))?;
        
        let addr = std::net::SocketAddr::new(ip_addr, UCNET_DISCOVERY_PORT);
        
        // TODO: Perform actual connection handshake
        debug!("Establishing network connection to {}", addr);
        
        Ok(ConnectionData::Network { addr })
    }
    
    /// Establishes a USB connection to a device
    async fn connect_usb(&self, device: &UcNetDevice) -> Result<ConnectionData> {
        let device_path = device.identifier.clone();
        
        // TODO: Open USB device and establish communication
        debug!("Establishing USB connection to {}", device_path);
        
        Ok(ConnectionData::Usb { device_path })
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
