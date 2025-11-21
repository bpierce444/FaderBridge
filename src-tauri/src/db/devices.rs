//! Device repository - CRUD operations for devices

use rusqlite::params;
use std::str::FromStr;

use super::connection::Database;
use super::error::{DbError, DbResult};
use super::types::{ConnectionType, CreateDeviceRequest, Device, DeviceType};

impl Database {
    /// Create a new device
    pub fn create_device(&self, req: CreateDeviceRequest) -> DbResult<Device> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO devices (project_id, device_type, device_name, device_id, connection_type, config_json) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    req.project_id,
                    req.device_type.as_str(),
                    req.device_name,
                    req.device_id,
                    req.connection_type.map(|ct| ct.as_str()),
                    req.config_json,
                ],
            )?;

            let id = conn.last_insert_rowid();
            self.get_device_by_id(id)
        })
    }

    /// Get a device by ID
    pub fn get_device_by_id(&self, id: i64) -> DbResult<Device> {
        self.with_conn(|conn| {
            conn.query_row(
                "SELECT id, project_id, device_type, device_name, device_id, connection_type, config_json, created_at 
                 FROM devices WHERE id = ?1",
                params![id],
                |row| {
                    let device_type_str: String = row.get(2)?;
                    let connection_type_str: Option<String> = row.get(5)?;

                    Ok(Device {
                        id: row.get(0)?,
                        project_id: row.get(1)?,
                        device_type: DeviceType::from_str(&device_type_str)
                            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?,
                        device_name: row.get(3)?,
                        device_id: row.get(4)?,
                        connection_type: connection_type_str
                            .map(|s| ConnectionType::from_str(&s))
                            .transpose()
                            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?,
                        config_json: row.get(6)?,
                        created_at: row.get(7)?,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    DbError::DeviceNotFound(format!("Device with ID {} not found", id))
                }
                _ => DbError::from(e),
            })
        })
    }

    /// Get all devices for a project
    pub fn get_devices_by_project(&self, project_id: i64) -> DbResult<Vec<Device>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, project_id, device_type, device_name, device_id, connection_type, config_json, created_at 
                 FROM devices WHERE project_id = ?1 ORDER BY created_at ASC",
            )?;

            let devices = stmt
                .query_map(params![project_id], |row| {
                    let device_type_str: String = row.get(2)?;
                    let connection_type_str: Option<String> = row.get(5)?;

                    Ok(Device {
                        id: row.get(0)?,
                        project_id: row.get(1)?,
                        device_type: DeviceType::from_str(&device_type_str)
                            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?,
                        device_name: row.get(3)?,
                        device_id: row.get(4)?,
                        connection_type: connection_type_str
                            .map(|s| ConnectionType::from_str(&s))
                            .transpose()
                            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?,
                        config_json: row.get(6)?,
                        created_at: row.get(7)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(devices)
        })
    }

    /// Get devices by type for a project
    pub fn get_devices_by_type(
        &self,
        project_id: i64,
        device_type: DeviceType,
    ) -> DbResult<Vec<Device>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, project_id, device_type, device_name, device_id, connection_type, config_json, created_at 
                 FROM devices WHERE project_id = ?1 AND device_type = ?2 ORDER BY created_at ASC",
            )?;

            let devices = stmt
                .query_map(params![project_id, device_type.as_str()], |row| {
                    let device_type_str: String = row.get(2)?;
                    let connection_type_str: Option<String> = row.get(5)?;

                    Ok(Device {
                        id: row.get(0)?,
                        project_id: row.get(1)?,
                        device_type: DeviceType::from_str(&device_type_str)
                            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?,
                        device_name: row.get(3)?,
                        device_id: row.get(4)?,
                        connection_type: connection_type_str
                            .map(|s| ConnectionType::from_str(&s))
                            .transpose()
                            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?,
                        config_json: row.get(6)?,
                        created_at: row.get(7)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(devices)
        })
    }

    /// Update device configuration
    pub fn update_device_config(&self, id: i64, config_json: Option<String>) -> DbResult<Device> {
        self.with_conn(|conn| {
            let rows_affected = conn.execute(
                "UPDATE devices SET config_json = ?1 WHERE id = ?2",
                params![config_json, id],
            )?;

            if rows_affected == 0 {
                return Err(DbError::DeviceNotFound(format!(
                    "Device with ID {} not found",
                    id
                )));
            }

            self.get_device_by_id(id)
        })
    }

    /// Delete a device (will cascade delete all associated mappings)
    pub fn delete_device(&self, id: i64) -> DbResult<()> {
        self.with_conn(|conn| {
            let rows_affected = conn.execute("DELETE FROM devices WHERE id = ?1", params![id])?;

            if rows_affected == 0 {
                return Err(DbError::DeviceNotFound(format!(
                    "Device with ID {} not found",
                    id
                )));
            }

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::types::CreateProjectRequest;

    fn create_test_db() -> Database {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("faderbridge_test_{}.db", uuid::Uuid::new_v4()));
        Database::init(db_path).expect("Failed to initialize test database")
    }

    #[test]
    fn test_create_and_get_device() {
        let db = create_test_db();

        let project = db
            .create_project(CreateProjectRequest {
                name: "Test Project".to_string(),
                description: None,
            })
            .expect("Failed to create project");

        let req = CreateDeviceRequest {
            project_id: project.id,
            device_type: DeviceType::Midi,
            device_name: "FaderPort 8".to_string(),
            device_id: "FaderPort8:0".to_string(),
            connection_type: Some(ConnectionType::Usb),
            config_json: Some(r#"{"channels": 8}"#.to_string()),
        };

        let device = db.create_device(req).expect("Failed to create device");

        assert_eq!(device.device_name, "FaderPort 8");
        assert_eq!(device.device_type, DeviceType::Midi);

        let fetched = db
            .get_device_by_id(device.id)
            .expect("Failed to get device");
        assert_eq!(fetched.device_name, device.device_name);
    }

    #[test]
    fn test_get_devices_by_type() {
        let db = create_test_db();

        let project = db
            .create_project(CreateProjectRequest {
                name: "Test Project".to_string(),
                description: None,
            })
            .expect("Failed to create project");

        db.create_device(CreateDeviceRequest {
            project_id: project.id,
            device_type: DeviceType::Midi,
            device_name: "MIDI Device 1".to_string(),
            device_id: "midi1".to_string(),
            connection_type: None,
            config_json: None,
        })
        .expect("Failed to create MIDI device");

        db.create_device(CreateDeviceRequest {
            project_id: project.id,
            device_type: DeviceType::Ucnet,
            device_name: "UCNet Device 1".to_string(),
            device_id: "ucnet1".to_string(),
            connection_type: Some(ConnectionType::Network),
            config_json: None,
        })
        .expect("Failed to create UCNet device");

        let midi_devices = db
            .get_devices_by_type(project.id, DeviceType::Midi)
            .expect("Failed to get MIDI devices");
        assert_eq!(midi_devices.len(), 1);

        let ucnet_devices = db
            .get_devices_by_type(project.id, DeviceType::Ucnet)
            .expect("Failed to get UCNet devices");
        assert_eq!(ucnet_devices.len(), 1);
    }
}
