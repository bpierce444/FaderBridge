#[cfg(test)]
mod tests {
    use super::super::*;
    use tempfile::TempDir;

    fn setup_test_db() -> (Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::init(db_path).unwrap();
        db.migrate().unwrap();
        (db, temp_dir)
    }

    #[test]
    fn test_create_and_get_project() {
        let (db, _temp) = setup_test_db();

        let project = db
            .create_project("Test Project", Some("Test Description"))
            .unwrap();

        assert_eq!(project.name, "Test Project");
        assert_eq!(project.description, Some("Test Description".to_string()));
        assert!(!project.is_active);

        let retrieved = db.get_project(project.id).unwrap();
        assert_eq!(retrieved.id, project.id);
        assert_eq!(retrieved.name, project.name);
    }

    #[test]
    fn test_update_project() {
        let (db, _temp) = setup_test_db();

        let project = db.create_project("Original Name", None).unwrap();

        let updated = db
            .update_project(project.id, Some("Updated Name"), Some("New Description"))
            .unwrap();

        assert_eq!(updated.name, "Updated Name");
        assert_eq!(updated.description, Some("New Description".to_string()));
    }

    #[test]
    fn test_set_active_project() {
        let (db, _temp) = setup_test_db();

        let project1 = db.create_project("Project 1", None).unwrap();
        let project2 = db.create_project("Project 2", None).unwrap();

        db.set_active_project(project1.id).unwrap();
        let active = db.get_active_project().unwrap();
        assert_eq!(active.unwrap().id, project1.id);

        db.set_active_project(project2.id).unwrap();
        let active = db.get_active_project().unwrap();
        assert_eq!(active.unwrap().id, project2.id);
    }

    #[test]
    fn test_delete_project() {
        let (db, _temp) = setup_test_db();

        let project = db.create_project("To Delete", None).unwrap();
        assert!(db.get_project(project.id).is_ok());

        db.delete_project(project.id).unwrap();
        assert!(db.get_project(project.id).is_err());
    }

    #[test]
    fn test_get_recent_projects() {
        let (db, _temp) = setup_test_db();

        let _p1 = db.create_project("Project 1", None).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let _p2 = db.create_project("Project 2", None).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let _p3 = db.create_project("Project 3", None).unwrap();

        let recent = db.get_recent_projects(2).unwrap();
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].name, "Project 3");
        assert_eq!(recent[1].name, "Project 2");
    }

    #[test]
    fn test_create_and_get_device() {
        let (db, _temp) = setup_test_db();

        let project = db.create_project("Test Project", None).unwrap();
        let device = db
            .create_device(
                project.id,
                "MIDI Controller",
                types::DeviceType::Midi,
                Some("Port 1"),
            )
            .unwrap();

        assert_eq!(device.name, "MIDI Controller");
        assert_eq!(device.device_type, types::DeviceType::Midi);
        assert_eq!(device.identifier, Some("Port 1".to_string()));

        let devices = db.get_devices_by_project(project.id).unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].id, device.id);
    }

    #[test]
    fn test_update_device_config() {
        let (db, _temp) = setup_test_db();

        let project = db.create_project("Test Project", None).unwrap();
        let device = db
            .create_device(project.id, "Device", types::DeviceType::Midi, None)
            .unwrap();

        let config = serde_json::json!({"key": "value"});
        let updated = db
            .update_device_config(device.id, config.clone())
            .unwrap();

        assert_eq!(updated.config, Some(config));
    }

    #[test]
    fn test_create_and_get_mapping() {
        let (db, _temp) = setup_test_db();

        let project = db.create_project("Test Project", None).unwrap();
        let mapping = db
            .create_mapping(
                project.id,
                7,
                "line/ch1/vol",
                types::TaperCurve::Linear,
                None,
                None,
            )
            .unwrap();

        assert_eq!(mapping.midi_cc, 7);
        assert_eq!(mapping.ucnet_parameter, "line/ch1/vol");
        assert_eq!(mapping.taper_curve, types::TaperCurve::Linear);

        let mappings = db.get_mappings_by_project(project.id).unwrap();
        assert_eq!(mappings.len(), 1);
        assert_eq!(mappings[0].id, mapping.id);
    }

    #[test]
    fn test_update_mapping() {
        let (db, _temp) = setup_test_db();

        let project = db.create_project("Test Project", None).unwrap();
        let mapping = db
            .create_mapping(project.id, 7, "line/ch1/vol", types::TaperCurve::Linear, None, None)
            .unwrap();

        let updated = db
            .update_mapping(
                mapping.id,
                Some(types::TaperCurve::Logarithmic),
                Some(0.0),
                Some(1.0),
            )
            .unwrap();

        assert_eq!(updated.taper_curve, types::TaperCurve::Logarithmic);
        assert_eq!(updated.min_value, Some(0.0));
        assert_eq!(updated.max_value, Some(1.0));
    }

    #[test]
    fn test_find_mapping_by_midi_cc() {
        let (db, _temp) = setup_test_db();

        let project = db.create_project("Test Project", None).unwrap();
        db.set_active_project(project.id).unwrap();

        let _mapping = db
            .create_mapping(project.id, 7, "line/ch1/vol", types::TaperCurve::Linear, None, None)
            .unwrap();

        let found = db.find_mapping_by_midi_cc(7).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().midi_cc, 7);

        let not_found = db.find_mapping_by_midi_cc(99).unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_delete_mapping() {
        let (db, _temp) = setup_test_db();

        let project = db.create_project("Test Project", None).unwrap();
        let mapping = db
            .create_mapping(project.id, 7, "line/ch1/vol", types::TaperCurve::Linear, None, None)
            .unwrap();

        db.delete_mapping(mapping.id).unwrap();
        let mappings = db.get_mappings_by_project(project.id).unwrap();
        assert_eq!(mappings.len(), 0);
    }

    #[test]
    fn test_cascade_delete_project() {
        let (db, _temp) = setup_test_db();

        let project = db.create_project("Test Project", None).unwrap();
        let _device = db
            .create_device(project.id, "Device", types::DeviceType::Midi, None)
            .unwrap();
        let _mapping = db
            .create_mapping(project.id, 7, "line/ch1/vol", types::TaperCurve::Linear, None, None)
            .unwrap();

        db.delete_project(project.id).unwrap();

        let devices = db.get_devices_by_project(project.id).unwrap();
        assert_eq!(devices.len(), 0);

        let mappings = db.get_mappings_by_project(project.id).unwrap();
        assert_eq!(mappings.len(), 0);
    }
}
