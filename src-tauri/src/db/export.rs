//! Project export and import functionality

use std::fs;
use std::path::Path;

use super::connection::Database;
use super::error::{DbError, DbResult};
use super::types::{
    CreateDeviceRequest, CreateMappingRequest, CreateProjectRequest, ProjectExport,
};

const EXPORT_VERSION: &str = "1.0.0";

impl Database {
    /// Export a project to JSON format
    ///
    /// Returns a JSON string containing the project, devices, and mappings.
    /// The export is version-tagged for future compatibility.
    pub fn export_project(&self, project_id: i64) -> DbResult<String> {
        let project = self.get_project_by_id(project_id)?;
        let devices = self.get_devices_by_project(project_id)?;
        let mappings = self.get_mappings_by_project(project_id)?;

        let export = ProjectExport {
            version: EXPORT_VERSION.to_string(),
            exported_at: chrono::Utc::now().to_rfc3339(),
            project,
            devices,
            mappings,
        };

        serde_json::to_string_pretty(&export).map_err(DbError::from)
    }

    /// Export a project to a JSON file
    pub fn export_project_to_file(&self, project_id: i64, file_path: &Path) -> DbResult<()> {
        let json = self.export_project(project_id)?;
        fs::write(file_path, json).map_err(DbError::from)
    }

    /// Import a project from JSON string
    ///
    /// Creates a new project with all devices and mappings from the export.
    /// The imported project will have a new ID and timestamp.
    /// If a project with the same name exists, it will be renamed with a suffix.
    pub fn import_project(&self, json: &str) -> DbResult<i64> {
        let export: ProjectExport = serde_json::from_str(json).map_err(|e| {
            DbError::ExportImport(format!("Failed to parse project JSON: {}", e))
        })?;

        // Validate export version
        if !export.version.starts_with("1.") {
            return Err(DbError::ExportImport(format!(
                "Unsupported export version: {}. This app supports version 1.x",
                export.version
            )));
        }

        // Check if project name already exists and generate unique name if needed
        let mut project_name = export.project.name.clone();
        let mut counter = 1;
        while self.get_project_by_name(&project_name).is_ok() {
            project_name = format!("{} ({})", export.project.name, counter);
            counter += 1;
        }

        // Create new project
        let new_project = self.create_project(CreateProjectRequest {
            name: project_name,
            description: export.project.description.clone(),
        })?;

        // Map old device IDs to new device IDs
        let mut device_id_map = std::collections::HashMap::new();

        // Import devices
        for device in &export.devices {
            let new_device = self.create_device(CreateDeviceRequest {
                project_id: new_project.id,
                device_type: device.device_type,
                device_name: device.device_name.clone(),
                device_id: device.device_id.clone(),
                connection_type: device.connection_type,
                config_json: device.config_json.clone(),
            })?;

            device_id_map.insert(device.id, new_device.id);
        }

        // Import mappings
        for mapping in &export.mappings {
            let midi_device_id = device_id_map.get(&mapping.midi_device_id).ok_or_else(|| {
                DbError::ExportImport(format!(
                    "Invalid MIDI device reference: {}",
                    mapping.midi_device_id
                ))
            })?;

            let ucnet_device_id = device_id_map.get(&mapping.ucnet_device_id).ok_or_else(|| {
                DbError::ExportImport(format!(
                    "Invalid UCNet device reference: {}",
                    mapping.ucnet_device_id
                ))
            })?;

            self.create_mapping(CreateMappingRequest {
                project_id: new_project.id,
                midi_device_id: *midi_device_id,
                ucnet_device_id: *ucnet_device_id,
                midi_channel: mapping.midi_channel,
                midi_cc: mapping.midi_cc,
                ucnet_parameter_id: mapping.ucnet_parameter_id,
                ucnet_parameter_name: mapping.ucnet_parameter_name.clone(),
                taper_curve: Some(mapping.taper_curve),
                min_value: Some(mapping.min_value),
                max_value: Some(mapping.max_value),
                invert: Some(mapping.invert),
                bidirectional: Some(mapping.bidirectional),
                label: mapping.label.clone(),
            })?;
        }

        Ok(new_project.id)
    }

    /// Import a project from a JSON file
    pub fn import_project_from_file(&self, file_path: &Path) -> DbResult<i64> {
        let json = fs::read_to_string(file_path).map_err(|e| {
            DbError::ExportImport(format!("Failed to read file {:?}: {}", file_path, e))
        })?;

        self.import_project(&json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::types::{CreateDeviceRequest, CreateProjectRequest, DeviceType};

    fn create_test_db() -> Database {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("faderbridge_test_{}.db", uuid::Uuid::new_v4()));
        Database::init(db_path).expect("Failed to initialize test database")
    }

    #[test]
    fn test_export_and_import_project() {
        let db = create_test_db();

        // Create a project with devices and mappings
        let project = db
            .create_project(CreateProjectRequest {
                name: "Export Test".to_string(),
                description: Some("Test project for export".to_string()),
            })
            .expect("Failed to create project");

        let midi_device = db
            .create_device(CreateDeviceRequest {
                project_id: project.id,
                device_type: DeviceType::Midi,
                device_name: "MIDI Device".to_string(),
                device_id: "midi1".to_string(),
                connection_type: None,
                config_json: None,
            })
            .expect("Failed to create MIDI device");

        let ucnet_device = db
            .create_device(CreateDeviceRequest {
                project_id: project.id,
                device_type: DeviceType::Ucnet,
                device_name: "UCNet Device".to_string(),
                device_id: "ucnet1".to_string(),
                connection_type: None,
                config_json: None,
            })
            .expect("Failed to create UCNet device");

        db.create_mapping(CreateMappingRequest {
            project_id: project.id,
            midi_device_id: midi_device.id,
            ucnet_device_id: ucnet_device.id,
            midi_channel: 0,
            midi_cc: 1,
            ucnet_parameter_id: 100,
            ucnet_parameter_name: "Test Param".to_string(),
            taper_curve: None,
            min_value: None,
            max_value: None,
            invert: None,
            bidirectional: None,
            label: Some("Test Mapping".to_string()),
        })
        .expect("Failed to create mapping");

        // Export project
        let json = db
            .export_project(project.id)
            .expect("Failed to export project");

        // Import project
        let new_project_id = db.import_project(&json).expect("Failed to import project");

        // Verify imported project
        let imported_project = db
            .get_project_by_id(new_project_id)
            .expect("Failed to get imported project");
        assert_eq!(imported_project.name, "Export Test");

        let imported_devices = db
            .get_devices_by_project(new_project_id)
            .expect("Failed to get imported devices");
        assert_eq!(imported_devices.len(), 2);

        let imported_mappings = db
            .get_mappings_by_project(new_project_id)
            .expect("Failed to get imported mappings");
        assert_eq!(imported_mappings.len(), 1);
        assert_eq!(imported_mappings[0].label, Some("Test Mapping".to_string()));
    }

    #[test]
    fn test_import_duplicate_name() {
        let db = create_test_db();

        let project = db
            .create_project(CreateProjectRequest {
                name: "Duplicate Test".to_string(),
                description: None,
            })
            .expect("Failed to create project");

        let json = db
            .export_project(project.id)
            .expect("Failed to export project");

        // Import should create "Duplicate Test (1)"
        let new_project_id = db.import_project(&json).expect("Failed to import project");

        let imported_project = db
            .get_project_by_id(new_project_id)
            .expect("Failed to get imported project");
        assert_eq!(imported_project.name, "Duplicate Test (1)");
    }
}
