//! Project repository - CRUD operations for projects

use rusqlite::params;

use super::connection::Database;
use super::error::{DbError, DbResult};
use super::types::{CreateProjectRequest, Project, UpdateProjectRequest};

impl Database {
    /// Create a new project
    pub fn create_project(&self, req: CreateProjectRequest) -> DbResult<Project> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO projects (name, description) VALUES (?1, ?2)",
                params![req.name, req.description],
            )?;

            let id = conn.last_insert_rowid();
            self.get_project_by_id(id)
        })
    }

    /// Get a project by ID
    pub fn get_project_by_id(&self, id: i64) -> DbResult<Project> {
        self.with_conn(|conn| {
            conn.query_row(
                "SELECT id, name, description, created_at, updated_at, last_opened_at, is_active 
                 FROM projects WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Project {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        description: row.get(2)?,
                        created_at: row.get(3)?,
                        updated_at: row.get(4)?,
                        last_opened_at: row.get(5)?,
                        is_active: row.get::<_, i32>(6)? != 0,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    DbError::ProjectNotFound(format!("Project with ID {} not found", id))
                }
                _ => DbError::from(e),
            })
        })
    }

    /// Get a project by name
    pub fn get_project_by_name(&self, name: &str) -> DbResult<Project> {
        self.with_conn(|conn| {
            conn.query_row(
                "SELECT id, name, description, created_at, updated_at, last_opened_at, is_active 
                 FROM projects WHERE name = ?1",
                params![name],
                |row| {
                    Ok(Project {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        description: row.get(2)?,
                        created_at: row.get(3)?,
                        updated_at: row.get(4)?,
                        last_opened_at: row.get(5)?,
                        is_active: row.get::<_, i32>(6)? != 0,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    DbError::ProjectNotFound(format!("Project '{}' not found", name))
                }
                _ => DbError::from(e),
            })
        })
    }

    /// Get all projects, ordered by last opened (most recent first)
    pub fn get_all_projects(&self) -> DbResult<Vec<Project>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, description, created_at, updated_at, last_opened_at, is_active 
                 FROM projects 
                 ORDER BY last_opened_at DESC NULLS LAST, created_at DESC",
            )?;

            let projects = stmt
                .query_map([], |row| {
                    Ok(Project {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        description: row.get(2)?,
                        created_at: row.get(3)?,
                        updated_at: row.get(4)?,
                        last_opened_at: row.get(5)?,
                        is_active: row.get::<_, i32>(6)? != 0,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(projects)
        })
    }

    /// Get recent projects (limited by count)
    pub fn get_recent_projects(&self, limit: usize) -> DbResult<Vec<Project>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, description, created_at, updated_at, last_opened_at, is_active 
                 FROM projects 
                 WHERE last_opened_at IS NOT NULL
                 ORDER BY last_opened_at DESC 
                 LIMIT ?1",
            )?;

            let projects = stmt
                .query_map(params![limit as i64], |row| {
                    Ok(Project {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        description: row.get(2)?,
                        created_at: row.get(3)?,
                        updated_at: row.get(4)?,
                        last_opened_at: row.get(5)?,
                        is_active: row.get::<_, i32>(6)? != 0,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(projects)
        })
    }

    /// Update a project
    pub fn update_project(&self, req: UpdateProjectRequest) -> DbResult<Project> {
        self.with_conn(|conn| {
            // Build dynamic UPDATE query based on provided fields
            let mut updates = Vec::new();
            let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

            if let Some(name) = &req.name {
                updates.push("name = ?");
                params.push(Box::new(name.clone()));
            }

            if let Some(description) = &req.description {
                updates.push("description = ?");
                params.push(Box::new(description.clone()));
            }

            if updates.is_empty() {
                return self.get_project_by_id(req.id);
            }

            updates.push("updated_at = CURRENT_TIMESTAMP");

            let query = format!(
                "UPDATE projects SET {} WHERE id = ?",
                updates.join(", ")
            );

            params.push(Box::new(req.id));

            let param_refs: Vec<&dyn rusqlite::ToSql> =
                params.iter().map(|p| p.as_ref()).collect();

            conn.execute(&query, param_refs.as_slice())?;

            self.get_project_by_id(req.id)
        })
    }

    /// Mark a project as active and update last_opened_at
    pub fn set_active_project(&self, id: i64) -> DbResult<()> {
        self.with_conn(|conn| {
            // Deactivate all projects first
            conn.execute("UPDATE projects SET is_active = 0", [])?;

            // Activate the specified project and update last_opened_at
            let rows_affected = conn.execute(
                "UPDATE projects SET is_active = 1, last_opened_at = CURRENT_TIMESTAMP WHERE id = ?1",
                params![id],
            )?;

            if rows_affected == 0 {
                return Err(DbError::ProjectNotFound(format!(
                    "Project with ID {} not found",
                    id
                )));
            }

            Ok(())
        })
    }

    /// Get the currently active project, if any
    pub fn get_active_project(&self) -> DbResult<Option<Project>> {
        self.with_conn(|conn| {
            match conn.query_row(
                "SELECT id, name, description, created_at, updated_at, last_opened_at, is_active 
                 FROM projects WHERE is_active = 1",
                [],
                |row| {
                    Ok(Project {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        description: row.get(2)?,
                        created_at: row.get(3)?,
                        updated_at: row.get(4)?,
                        last_opened_at: row.get(5)?,
                        is_active: row.get::<_, i32>(6)? != 0,
                    })
                },
            ) {
                Ok(project) => Ok(Some(project)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(DbError::from(e)),
            }
        })
    }

    /// Delete a project and all associated data (cascades to devices and mappings)
    pub fn delete_project(&self, id: i64) -> DbResult<()> {
        self.with_conn(|conn| {
            let rows_affected = conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;

            if rows_affected == 0 {
                return Err(DbError::ProjectNotFound(format!(
                    "Project with ID {} not found",
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

    fn create_test_db() -> Database {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("faderbridge_test_{}.db", uuid::Uuid::new_v4()));
        Database::init(db_path).expect("Failed to initialize test database")
    }

    #[test]
    fn test_create_and_get_project() {
        let db = create_test_db();

        let req = CreateProjectRequest {
            name: "Test Project".to_string(),
            description: Some("A test project".to_string()),
        };

        let project = db.create_project(req).expect("Failed to create project");

        assert_eq!(project.name, "Test Project");
        assert_eq!(project.description, Some("A test project".to_string()));

        let fetched = db
            .get_project_by_id(project.id)
            .expect("Failed to get project");
        assert_eq!(fetched.name, project.name);
    }

    #[test]
    fn test_duplicate_project_name() {
        let db = create_test_db();

        let req = CreateProjectRequest {
            name: "Duplicate".to_string(),
            description: None,
        };

        db.create_project(req.clone())
            .expect("Failed to create first project");

        let result = db.create_project(req);
        assert!(matches!(result, Err(DbError::DuplicateProjectName(_))));
    }

    #[test]
    fn test_set_active_project() {
        let db = create_test_db();

        let p1 = db
            .create_project(CreateProjectRequest {
                name: "Project 1".to_string(),
                description: None,
            })
            .expect("Failed to create project 1");

        let p2 = db
            .create_project(CreateProjectRequest {
                name: "Project 2".to_string(),
                description: None,
            })
            .expect("Failed to create project 2");

        db.set_active_project(p1.id)
            .expect("Failed to set active project");

        let active = db
            .get_active_project()
            .expect("Failed to get active project")
            .expect("No active project");
        assert_eq!(active.id, p1.id);

        db.set_active_project(p2.id)
            .expect("Failed to set active project");

        let active = db
            .get_active_project()
            .expect("Failed to get active project")
            .expect("No active project");
        assert_eq!(active.id, p2.id);
    }
}
