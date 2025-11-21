//! Mapping repository - CRUD operations for parameter mappings

use rusqlite::params;
use std::str::FromStr;

use super::connection::Database;
use super::error::{DbError, DbResult};
use super::types::{CreateMappingRequest, Mapping, TaperCurve, UpdateMappingRequest};

impl Database {
    /// Create a new mapping
    pub fn create_mapping(&self, req: CreateMappingRequest) -> DbResult<Mapping> {
        self.with_conn(|conn| {
            let taper_curve = req.taper_curve.unwrap_or(TaperCurve::Linear);
            let min_value = req.min_value.unwrap_or(0.0);
            let max_value = req.max_value.unwrap_or(1.0);
            let invert = req.invert.unwrap_or(false);
            let bidirectional = req.bidirectional.unwrap_or(true);

            conn.execute(
                "INSERT INTO mappings (
                    project_id, midi_device_id, ucnet_device_id,
                    midi_channel, midi_cc,
                    ucnet_parameter_id, ucnet_parameter_name,
                    taper_curve, min_value, max_value, invert, bidirectional, label
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    req.project_id,
                    req.midi_device_id,
                    req.ucnet_device_id,
                    req.midi_channel,
                    req.midi_cc,
                    req.ucnet_parameter_id,
                    req.ucnet_parameter_name,
                    taper_curve.as_str(),
                    min_value,
                    max_value,
                    invert as i32,
                    bidirectional as i32,
                    req.label,
                ],
            )?;

            let id = conn.last_insert_rowid();
            self.get_mapping_by_id(id)
        })
    }

    /// Get a mapping by ID
    pub fn get_mapping_by_id(&self, id: i64) -> DbResult<Mapping> {
        self.with_conn(|conn| {
            conn.query_row(
                "SELECT id, project_id, midi_device_id, ucnet_device_id,
                        midi_channel, midi_cc, ucnet_parameter_id, ucnet_parameter_name,
                        taper_curve, min_value, max_value, invert, bidirectional, label, created_at
                 FROM mappings WHERE id = ?1",
                params![id],
                |row| {
                    let taper_curve_str: String = row.get(8)?;
                    
                    Ok(Mapping {
                        id: row.get(0)?,
                        project_id: row.get(1)?,
                        midi_device_id: row.get(2)?,
                        ucnet_device_id: row.get(3)?,
                        midi_channel: row.get(4)?,
                        midi_cc: row.get(5)?,
                        ucnet_parameter_id: row.get(6)?,
                        ucnet_parameter_name: row.get(7)?,
                        taper_curve: TaperCurve::from_str(&taper_curve_str)
                            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?,
                        min_value: row.get(9)?,
                        max_value: row.get(10)?,
                        invert: row.get::<_, i32>(11)? != 0,
                        bidirectional: row.get::<_, i32>(12)? != 0,
                        label: row.get(13)?,
                        created_at: row.get(14)?,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    DbError::MappingNotFound(format!("Mapping with ID {} not found", id))
                }
                _ => DbError::from(e),
            })
        })
    }

    /// Get all mappings for a project
    pub fn get_mappings_by_project(&self, project_id: i64) -> DbResult<Vec<Mapping>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, project_id, midi_device_id, ucnet_device_id,
                        midi_channel, midi_cc, ucnet_parameter_id, ucnet_parameter_name,
                        taper_curve, min_value, max_value, invert, bidirectional, label, created_at
                 FROM mappings WHERE project_id = ?1 ORDER BY created_at ASC",
            )?;

            let mappings = stmt
                .query_map(params![project_id], |row| {
                    let taper_curve_str: String = row.get(8)?;
                    
                    Ok(Mapping {
                        id: row.get(0)?,
                        project_id: row.get(1)?,
                        midi_device_id: row.get(2)?,
                        ucnet_device_id: row.get(3)?,
                        midi_channel: row.get(4)?,
                        midi_cc: row.get(5)?,
                        ucnet_parameter_id: row.get(6)?,
                        ucnet_parameter_name: row.get(7)?,
                        taper_curve: TaperCurve::from_str(&taper_curve_str)
                            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?,
                        min_value: row.get(9)?,
                        max_value: row.get(10)?,
                        invert: row.get::<_, i32>(11)? != 0,
                        bidirectional: row.get::<_, i32>(12)? != 0,
                        label: row.get(13)?,
                        created_at: row.get(14)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(mappings)
        })
    }

    /// Find mapping by MIDI CC
    pub fn find_mapping_by_midi_cc(
        &self,
        project_id: i64,
        midi_device_id: i64,
        midi_channel: u8,
        midi_cc: u8,
    ) -> DbResult<Option<Mapping>> {
        self.with_conn(|conn| {
            match conn.query_row(
                "SELECT id, project_id, midi_device_id, ucnet_device_id,
                        midi_channel, midi_cc, ucnet_parameter_id, ucnet_parameter_name,
                        taper_curve, min_value, max_value, invert, bidirectional, label, created_at
                 FROM mappings 
                 WHERE project_id = ?1 AND midi_device_id = ?2 AND midi_channel = ?3 AND midi_cc = ?4",
                params![project_id, midi_device_id, midi_channel, midi_cc],
                |row| {
                    let taper_curve_str: String = row.get(8)?;
                    
                    Ok(Mapping {
                        id: row.get(0)?,
                        project_id: row.get(1)?,
                        midi_device_id: row.get(2)?,
                        ucnet_device_id: row.get(3)?,
                        midi_channel: row.get(4)?,
                        midi_cc: row.get(5)?,
                        ucnet_parameter_id: row.get(6)?,
                        ucnet_parameter_name: row.get(7)?,
                        taper_curve: TaperCurve::from_str(&taper_curve_str)
                            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?,
                        min_value: row.get(9)?,
                        max_value: row.get(10)?,
                        invert: row.get::<_, i32>(11)? != 0,
                        bidirectional: row.get::<_, i32>(12)? != 0,
                        label: row.get(13)?,
                        created_at: row.get(14)?,
                    })
                },
            ) {
                Ok(mapping) => Ok(Some(mapping)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(DbError::from(e)),
            }
        })
    }

    /// Update a mapping
    pub fn update_mapping(&self, req: UpdateMappingRequest) -> DbResult<Mapping> {
        self.with_conn(|conn| {
            let mut updates = Vec::new();
            let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

            if let Some(taper_curve) = req.taper_curve {
                updates.push("taper_curve = ?");
                params.push(Box::new(taper_curve.as_str().to_string()));
            }

            if let Some(min_value) = req.min_value {
                updates.push("min_value = ?");
                params.push(Box::new(min_value));
            }

            if let Some(max_value) = req.max_value {
                updates.push("max_value = ?");
                params.push(Box::new(max_value));
            }

            if let Some(invert) = req.invert {
                updates.push("invert = ?");
                params.push(Box::new(invert as i32));
            }

            if let Some(bidirectional) = req.bidirectional {
                updates.push("bidirectional = ?");
                params.push(Box::new(bidirectional as i32));
            }

            if let Some(label) = req.label {
                updates.push("label = ?");
                params.push(Box::new(label));
            }

            if updates.is_empty() {
                return self.get_mapping_by_id(req.id);
            }

            let query = format!("UPDATE mappings SET {} WHERE id = ?", updates.join(", "));
            params.push(Box::new(req.id));

            let param_refs: Vec<&dyn rusqlite::ToSql> =
                params.iter().map(|p| p.as_ref()).collect();

            conn.execute(&query, param_refs.as_slice())?;

            self.get_mapping_by_id(req.id)
        })
    }

    /// Delete a mapping
    pub fn delete_mapping(&self, id: i64) -> DbResult<()> {
        self.with_conn(|conn| {
            let rows_affected = conn.execute("DELETE FROM mappings WHERE id = ?1", params![id])?;

            if rows_affected == 0 {
                return Err(DbError::MappingNotFound(format!(
                    "Mapping with ID {} not found",
                    id
                )));
            }

            Ok(())
        })
    }

    /// Delete all mappings for a project
    pub fn delete_mappings_by_project(&self, project_id: i64) -> DbResult<usize> {
        self.with_conn(|conn| {
            let rows_affected =
                conn.execute("DELETE FROM mappings WHERE project_id = ?1", params![project_id])?;
            Ok(rows_affected)
        })
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
    fn test_create_and_get_mapping() {
        let db = create_test_db();

        let project = db
            .create_project(CreateProjectRequest {
                name: "Test Project".to_string(),
                description: None,
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

        let req = CreateMappingRequest {
            project_id: project.id,
            midi_device_id: midi_device.id,
            ucnet_device_id: ucnet_device.id,
            midi_channel: 0,
            midi_cc: 1,
            ucnet_parameter_id: 100,
            ucnet_parameter_name: "Channel 1 Fader".to_string(),
            taper_curve: Some(TaperCurve::Logarithmic),
            min_value: Some(0.0),
            max_value: Some(1.0),
            invert: Some(false),
            bidirectional: Some(true),
            label: Some("Ch1 Fader".to_string()),
        };

        let mapping = db.create_mapping(req).expect("Failed to create mapping");

        assert_eq!(mapping.midi_cc, 1);
        assert_eq!(mapping.taper_curve, TaperCurve::Logarithmic);

        let fetched = db
            .get_mapping_by_id(mapping.id)
            .expect("Failed to get mapping");
        assert_eq!(fetched.midi_cc, mapping.midi_cc);
    }

    #[test]
    fn test_duplicate_mapping() {
        let db = create_test_db();

        let project = db
            .create_project(CreateProjectRequest {
                name: "Test Project".to_string(),
                description: None,
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

        let req = CreateMappingRequest {
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
            label: None,
        };

        db.create_mapping(req.clone())
            .expect("Failed to create first mapping");

        let result = db.create_mapping(req);
        assert!(matches!(result, Err(DbError::DuplicateMapping { .. })));
    }
}
