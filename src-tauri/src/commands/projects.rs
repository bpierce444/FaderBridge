//! Tauri commands for project management

use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;

use crate::db::{
    CreateDeviceRequest, CreateMappingRequest, CreateProjectRequest, Database, Device, Mapping,
    Project, UpdateMappingRequest, UpdateProjectRequest,
};

/// Application state containing the database connection
pub struct AppState {
    pub db: Arc<Database>,
}

/// Initialize the database
#[tauri::command]
pub async fn init_database() -> Result<(), String> {
    let db_path = Database::default_path().map_err(|e| e.to_string())?;
    let db = Database::init(db_path).map_err(|e| e.to_string())?;
    db.migrate().map_err(|e| e.to_string())?;
    Ok(())
}

// ============================================================================
// Project Commands
// ============================================================================

/// Create a new project
#[tauri::command]
pub async fn create_project(
    state: State<'_, AppState>,
    name: String,
    description: Option<String>,
) -> Result<Project, String> {
    let req = CreateProjectRequest { name, description };
    state.db.create_project(req).map_err(|e| e.to_string())
}

/// Get a project by ID
#[tauri::command]
pub async fn get_project(state: State<'_, AppState>, id: i64) -> Result<Project, String> {
    state.db.get_project_by_id(id).map_err(|e| e.to_string())
}

/// Get all projects
#[tauri::command]
pub async fn get_all_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    state.db.get_all_projects().map_err(|e| e.to_string())
}

/// Get recent projects
#[tauri::command]
pub async fn get_recent_projects(
    state: State<'_, AppState>,
    limit: usize,
) -> Result<Vec<Project>, String> {
    state
        .db
        .get_recent_projects(limit)
        .map_err(|e| e.to_string())
}

/// Update a project
#[tauri::command]
pub async fn update_project(
    state: State<'_, AppState>,
    id: i64,
    name: Option<String>,
    description: Option<String>,
) -> Result<Project, String> {
    let req = UpdateProjectRequest {
        id,
        name,
        description,
    };
    state.db.update_project(req).map_err(|e| e.to_string())
}

/// Set the active project
#[tauri::command]
pub async fn set_active_project(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    state
        .db
        .set_active_project(id)
        .map_err(|e| e.to_string())
}

/// Get the active project
#[tauri::command]
pub async fn get_active_project(state: State<'_, AppState>) -> Result<Option<Project>, String> {
    state.db.get_active_project().map_err(|e| e.to_string())
}

/// Delete a project
#[tauri::command]
pub async fn delete_project(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    state.db.delete_project(id).map_err(|e| e.to_string())
}

// ============================================================================
// Device Commands
// ============================================================================

/// Create a new device
#[tauri::command]
pub async fn create_device(
    state: State<'_, AppState>,
    req: CreateDeviceRequest,
) -> Result<Device, String> {
    state.db.create_device(req).map_err(|e| e.to_string())
}

/// Get a device by ID
#[tauri::command]
pub async fn get_device(state: State<'_, AppState>, id: i64) -> Result<Device, String> {
    state.db.get_device_by_id(id).map_err(|e| e.to_string())
}

/// Get all devices for a project
#[tauri::command]
pub async fn get_devices_by_project(
    state: State<'_, AppState>,
    project_id: i64,
) -> Result<Vec<Device>, String> {
    state
        .db
        .get_devices_by_project(project_id)
        .map_err(|e| e.to_string())
}

/// Update device configuration
#[tauri::command]
pub async fn update_device_config(
    state: State<'_, AppState>,
    id: i64,
    config_json: Option<String>,
) -> Result<Device, String> {
    state
        .db
        .update_device_config(id, config_json)
        .map_err(|e| e.to_string())
}

/// Delete a device
#[tauri::command]
pub async fn delete_device(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    state.db.delete_device(id).map_err(|e| e.to_string())
}

// ============================================================================
// Mapping Commands
// ============================================================================

/// Create a new mapping
#[tauri::command]
pub async fn create_mapping(
    state: State<'_, AppState>,
    req: CreateMappingRequest,
) -> Result<Mapping, String> {
    state.db.create_mapping(req).map_err(|e| e.to_string())
}

/// Get a mapping by ID
#[tauri::command]
pub async fn get_mapping(state: State<'_, AppState>, id: i64) -> Result<Mapping, String> {
    state.db.get_mapping_by_id(id).map_err(|e| e.to_string())
}

/// Get all mappings for a project
#[tauri::command]
pub async fn get_mappings_by_project(
    state: State<'_, AppState>,
    project_id: i64,
) -> Result<Vec<Mapping>, String> {
    state
        .db
        .get_mappings_by_project(project_id)
        .map_err(|e| e.to_string())
}

/// Update a mapping
#[tauri::command]
pub async fn update_mapping(
    state: State<'_, AppState>,
    req: UpdateMappingRequest,
) -> Result<Mapping, String> {
    state.db.update_mapping(req).map_err(|e| e.to_string())
}

/// Delete a mapping
#[tauri::command]
pub async fn delete_mapping(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    state.db.delete_mapping(id).map_err(|e| e.to_string())
}

// ============================================================================
// Export/Import Commands
// ============================================================================

/// Export a project to JSON
#[tauri::command]
pub async fn export_project(state: State<'_, AppState>, id: i64) -> Result<String, String> {
    state.db.export_project(id).map_err(|e| e.to_string())
}

/// Export a project to a file
#[tauri::command]
pub async fn export_project_to_file(
    state: State<'_, AppState>,
    id: i64,
    file_path: String,
) -> Result<(), String> {
    let path = PathBuf::from(file_path);
    state
        .db
        .export_project_to_file(id, &path)
        .map_err(|e| e.to_string())
}

/// Import a project from JSON
#[tauri::command]
pub async fn import_project(state: State<'_, AppState>, json: String) -> Result<i64, String> {
    state.db.import_project(&json).map_err(|e| e.to_string())
}

/// Import a project from a file
#[tauri::command]
pub async fn import_project_from_file(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<i64, String> {
    let path = PathBuf::from(file_path);
    state
        .db
        .import_project_from_file(&path)
        .map_err(|e| e.to_string())
}
