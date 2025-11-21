//! Database connection management and initialization

use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::error::{DbError, DbResult};

/// Thread-safe database connection wrapper
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Initialize the database at the specified path
    ///
    /// Creates the database file and directory if they don't exist,
    /// and runs the schema initialization.
    pub fn init(db_path: PathBuf) -> DbResult<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                DbError::Connection(format!("Failed to create database directory: {}", e))
            })?;
        }

        // Open connection
        let conn = Connection::open(&db_path).map_err(|e| {
            DbError::Connection(format!("Failed to open database at {:?}: {}", db_path, e))
        })?;

        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON", [])
            .map_err(|e| DbError::Connection(format!("Failed to enable foreign keys: {}", e)))?;

        // Initialize schema
        let schema = include_str!("schema.sql");
        conn.execute_batch(schema)
            .map_err(|e| DbError::Connection(format!("Failed to initialize schema: {}", e)))?;

        Ok(Database {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Get the database path for the current platform
    ///
    /// Returns: `~/Library/Application Support/FaderBridge/projects.db` on macOS
    pub fn default_path() -> DbResult<PathBuf> {
        let app_dir = dirs::data_local_dir()
            .ok_or_else(|| DbError::Connection("Could not determine app data directory".to_string()))?
            .join("FaderBridge");

        Ok(app_dir.join("projects.db"))
    }

    /// Execute a function with access to the database connection
    ///
    /// This provides safe, synchronized access to the underlying SQLite connection.
    pub fn with_conn<F, T>(&self, f: F) -> DbResult<T>
    where
        F: FnOnce(&Connection) -> DbResult<T>,
    {
        let conn = self.conn.lock().map_err(|e| {
            DbError::Connection(format!("Failed to acquire database lock: {}", e))
        })?;
        f(&conn)
    }

    /// Get the current schema version
    pub fn get_schema_version(&self) -> DbResult<i64> {
        self.with_conn(|conn| {
            let version = conn
                .query_row("SELECT MAX(version) FROM schema_version", [], |row| {
                    row.get(0)
                })
                .map_err(|e| DbError::Query(format!("Failed to get schema version: {}", e)))?;
            Ok(version)
        })
    }

    /// Run database migrations if needed
    ///
    /// This will be expanded in the future to handle schema upgrades.
    pub fn migrate(&self) -> DbResult<()> {
        let current_version = self.get_schema_version()?;
        log::info!("Current database schema version: {}", current_version);

        // Future migrations will go here
        // Example:
        // if current_version < 2 {
        //     self.migrate_to_v2()?;
        // }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_init() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("faderbridge_test.db");

        // Clean up if exists
        let _ = std::fs::remove_file(&db_path);

        let db = Database::init(db_path.clone()).expect("Failed to initialize database");

        // Verify schema version
        let version = db.get_schema_version().expect("Failed to get schema version");
        assert_eq!(version, 1);

        // Clean up
        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn test_foreign_keys_enabled() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("faderbridge_test_fk.db");

        let _ = std::fs::remove_file(&db_path);

        let db = Database::init(db_path.clone()).expect("Failed to initialize database");

        let fk_enabled = db
            .with_conn(|conn| {
                let enabled: i32 = conn
                    .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
                    .map_err(|e| DbError::Query(e.to_string()))?;
                Ok(enabled)
            })
            .expect("Failed to check foreign keys");

        assert_eq!(fk_enabled, 1);

        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }
}
