pub mod db;
pub mod parser;

use std::sync::Arc;

#[salsa::db]
#[derive(Default, Clone)]
pub struct RootDatabase {
    storage: salsa::Storage<Self>,
}

#[salsa::db]
impl salsa::Database for RootDatabase {}

#[salsa::db]
impl db::Db for RootDatabase {
    fn find_file(&self, _path: &str) -> Option<db::SourceFile> {
        None
    }
}

/// Thread-safe handle for the salsa database.
/// Provides exclusive input mutations via `with_db` and lock-free thread-local database snapshots via `snapshot()`.
pub struct SalsaThreadHandle(Arc<std::sync::Mutex<RootDatabase>>);

impl SalsaThreadHandle {
    pub fn new(db: RootDatabase) -> Self {
        SalsaThreadHandle(Arc::new(std::sync::Mutex::new(db)))
    }

    /// Mutate inputs or database state using exclusive write access.
    pub fn with_db<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut RootDatabase) -> R,
    {
        let mut guard = self.0.lock().unwrap();
        f(&mut *guard)
    }

    /// Obtain an isolated thread-local database snapshot for parallel, lock-free query execution.
    /// The lock is held only briefly to clone the Salsa database handle.
    pub fn snapshot(&self) -> RootDatabase {
        self.0.lock().unwrap().clone()
    }

    pub fn clone_db(&self) -> SalsaThreadHandle {
        SalsaThreadHandle(self.0.clone())
    }

    pub fn peek_db(&self) -> RootDatabase {
        self.snapshot()
    }

    pub fn read_db(&self, workspace: db::Workspace) -> (RootDatabase, db::Workspace) {
        (self.snapshot(), workspace)
    }
}

impl Clone for SalsaThreadHandle {
    fn clone(&self) -> Self {
        self.clone_db()
    }
}

impl Default for SalsaThreadHandle {
    fn default() -> Self {
        Self::new(RootDatabase::default())
    }
}

pub struct AnalysisHost {
    pub db_handle: SalsaThreadHandle,
    pub workspace: db::Workspace,
}

impl AnalysisHost {
    pub fn new() -> Self {
        let db = RootDatabase::default();
        let workspace = db::Workspace::new(&db, Vec::new());
        Self {
            db_handle: SalsaThreadHandle::new(db),
            workspace,
        }
    }

    pub fn validate_file(&self, path: &str, content: String) -> Vec<db::ValidationError> {
        let db = self.db_handle.snapshot();
        let source_file = db::SourceFile::new(&db, path.to_string(), content);
        db::validate_file(&db, self.workspace, source_file)
    }
}

impl Default for AnalysisHost {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_salsa_thread_handle_parallel_snapshots() {
        let handle = SalsaThreadHandle::default();
        let db1 = handle.snapshot();
        let db2 = handle.snapshot();

        let t1 = std::thread::spawn(move || {
            let _ws = db::Workspace::new(&db1, Vec::new());
            42
        });

        let t2 = std::thread::spawn(move || {
            let _ws = db::Workspace::new(&db2, Vec::new());
            84
        });

        assert_eq!(t1.join().unwrap(), 42);
        assert_eq!(t2.join().unwrap(), 84);
    }
}

