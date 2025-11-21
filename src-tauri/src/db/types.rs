//! Database types and models for FaderBridge

use serde::{Deserialize, Serialize};

/// Represents a saved project configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub last_opened_at: Option<String>,
    pub is_active: bool,
}

/// Request to create a new project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
}

/// Request to update an existing project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectRequest {
    pub id: i64,
    pub name: Option<String>,
    pub description: Option<String>,
}

/// Device type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    Midi,
    Ucnet,
}

impl DeviceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeviceType::Midi => "midi",
            DeviceType::Ucnet => "ucnet",
        }
    }
}

impl std::str::FromStr for DeviceType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "midi" => Ok(DeviceType::Midi),
            "ucnet" => Ok(DeviceType::Ucnet),
            _ => Err(format!("Invalid device type: {}", s)),
        }
    }
}

/// Connection type for devices
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionType {
    Usb,
    Network,
    Virtual,
}

impl ConnectionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConnectionType::Usb => "usb",
            ConnectionType::Network => "network",
            ConnectionType::Virtual => "virtual",
        }
    }
}

impl std::str::FromStr for ConnectionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "usb" => Ok(ConnectionType::Usb),
            "network" => Ok(ConnectionType::Network),
            "virtual" => Ok(ConnectionType::Virtual),
            _ => Err(format!("Invalid connection type: {}", s)),
        }
    }
}

/// Represents a device configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: i64,
    pub project_id: i64,
    pub device_type: DeviceType,
    pub device_name: String,
    pub device_id: String,
    pub connection_type: Option<ConnectionType>,
    pub config_json: Option<String>,
    pub created_at: String,
}

/// Request to create a new device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDeviceRequest {
    pub project_id: i64,
    pub device_type: DeviceType,
    pub device_name: String,
    pub device_id: String,
    pub connection_type: Option<ConnectionType>,
    pub config_json: Option<String>,
}

/// Taper curve type for parameter mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaperCurve {
    Linear,
    Logarithmic,
    Exponential,
    SCurve,
}

impl TaperCurve {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaperCurve::Linear => "linear",
            TaperCurve::Logarithmic => "logarithmic",
            TaperCurve::Exponential => "exponential",
            TaperCurve::SCurve => "s-curve",
        }
    }
}

impl std::str::FromStr for TaperCurve {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "linear" => Ok(TaperCurve::Linear),
            "logarithmic" => Ok(TaperCurve::Logarithmic),
            "exponential" => Ok(TaperCurve::Exponential),
            "s-curve" => Ok(TaperCurve::SCurve),
            _ => Err(format!("Invalid taper curve: {}", s)),
        }
    }
}

/// Represents a parameter mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapping {
    pub id: i64,
    pub project_id: i64,
    pub midi_device_id: i64,
    pub ucnet_device_id: i64,
    pub midi_channel: u8,
    pub midi_cc: u8,
    pub ucnet_parameter_id: i32,
    pub ucnet_parameter_name: String,
    pub taper_curve: TaperCurve,
    pub min_value: f64,
    pub max_value: f64,
    pub invert: bool,
    pub bidirectional: bool,
    pub label: Option<String>,
    pub created_at: String,
}

/// Request to create a new mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMappingRequest {
    pub project_id: i64,
    pub midi_device_id: i64,
    pub ucnet_device_id: i64,
    pub midi_channel: u8,
    pub midi_cc: u8,
    pub ucnet_parameter_id: i32,
    pub ucnet_parameter_name: String,
    pub taper_curve: Option<TaperCurve>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub invert: Option<bool>,
    pub bidirectional: Option<bool>,
    pub label: Option<String>,
}

/// Request to update an existing mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMappingRequest {
    pub id: i64,
    pub taper_curve: Option<TaperCurve>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub invert: Option<bool>,
    pub bidirectional: Option<bool>,
    pub label: Option<String>,
}

/// Complete project export including all related data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectExport {
    pub version: String,
    pub exported_at: String,
    pub project: Project,
    pub devices: Vec<Device>,
    pub mappings: Vec<Mapping>,
}

/// User preference key-value pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preference {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}
