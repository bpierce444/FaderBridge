//! Tauri commands for UCNet device management

use crate::ucnet::{
    discovery::DeviceDiscovery, ConnectionManager, DefaultDeviceDiscovery, UcNetDevice,
};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

/// Application state for UCNet management
pub struct UcNetState {
    pub discovery: Arc<DefaultDeviceDiscovery>,
    pub connection_manager: Arc<ConnectionManager>,
    pub discovered_devices: Arc<RwLock<Vec<UcNetDevice>>>,
}

impl UcNetState {
    /// Creates a new UCNet state
    pub fn new() -> Self {
        let connection_manager = Arc::new(ConnectionManager::new());
        
        // Note: Keep-alive task will be started on first connect
        // to ensure Tokio runtime is available
        
        Self {
            discovery: Arc::new(DefaultDeviceDiscovery::new()),
            connection_manager,
            discovered_devices: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

/// Discovers UCNet devices on network and USB
#[tauri::command]
pub async fn discover_devices(state: State<'_, UcNetState>) -> Result<Vec<UcNetDevice>, String> {
    log::info!("Starting device discovery");
    
    let mut all_devices = Vec::new();
    
    // Discover network devices
    match state.discovery.discover_network_devices().await {
        Ok(network_devices) => {
            log::info!("Found {} network device(s)", network_devices.len());
            all_devices.extend(network_devices.into_iter().map(UcNetDevice::from_network));
        }
        Err(e) => {
            log::error!("Network discovery failed: {}", e);
            // Continue with USB discovery even if network fails
        }
    }
    
    // Discover USB devices
    match state.discovery.discover_usb_devices() {
        Ok(usb_devices) => {
            log::info!("Found {} USB device(s)", usb_devices.len());
            all_devices.extend(usb_devices.into_iter().map(UcNetDevice::from_usb));
        }
        Err(e) => {
            log::error!("USB discovery failed: {}", e);
        }
    }
    
    // Update stored devices
    {
        let mut discovered = state.discovered_devices.write().await;
        *discovered = all_devices.clone();
    }
    
    log::info!("Discovery complete. Total devices found: {}", all_devices.len());
    Ok(all_devices)
}

/// Connects to a specific device
#[tauri::command]
pub async fn connect_device(
    device_id: String,
    state: State<'_, UcNetState>,
) -> Result<(), String> {
    log::info!("Connecting to device: {}", device_id);
    
    // Find the device in discovered devices
    let device = {
        let discovered = state.discovered_devices.read().await;
        discovered
            .iter()
            .find(|d| d.id == device_id)
            .cloned()
            .ok_or_else(|| format!("Device not found: {}", device_id))?
    };
    
    // Connect to the device
    state
        .connection_manager
        .connect(device)
        .await
        .map_err(|e| e.to_string())?;
    
    log::info!("Successfully connected to device: {}", device_id);
    Ok(())
}

/// Disconnects from a specific device
#[tauri::command]
pub async fn disconnect_device(
    device_id: String,
    state: State<'_, UcNetState>,
) -> Result<(), String> {
    log::info!("Disconnecting from device: {}", device_id);
    
    state
        .connection_manager
        .disconnect(&device_id)
        .await
        .map_err(|e| e.to_string())?;
    
    log::info!("Successfully disconnected from device: {}", device_id);
    Ok(())
}

/// Gets all currently connected devices
#[tauri::command]
pub async fn get_connected_devices(
    state: State<'_, UcNetState>,
) -> Result<Vec<UcNetDevice>, String> {
    let devices = state.connection_manager.get_connected_devices().await;
    Ok(devices)
}

/// Gets the connection state of a specific device
#[tauri::command]
pub async fn get_device_state(
    device_id: String,
    state: State<'_, UcNetState>,
) -> Result<Option<String>, String> {
    let state_opt = state.connection_manager.get_device_state(&device_id).await;
    Ok(state_opt.map(|s| format!("{:?}", s)))
}
