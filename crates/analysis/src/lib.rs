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
pub struct SalsaThreadHandle(Arc<std::sync::Mutex<RootDatabase>>);

impl SalsaThreadHandle {
    pub fn new(db: RootDatabase) -> Self {
        SalsaThreadHandle(Arc::new(std::sync::Mutex::new(db)))
    }

    pub fn with_db<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut RootDatabase) -> R,
    {
        let mut guard = self.0.lock().unwrap();
        f(&mut *guard)
    }

    pub fn clone_db(&self) -> SalsaThreadHandle {
        SalsaThreadHandle(self.0.clone())
    }

    pub fn peek_db(&self) -> RootDatabase {
        self.0.lock().unwrap().clone()
    }

    pub fn read_db(&self, workspace: db::Workspace) -> (RootDatabase, db::Workspace) {
        let guard = self.0.lock().unwrap();
        (guard.clone(), workspace)
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
        self.db_handle.with_db(|db| {
            let source_file = db::SourceFile::new(db, path.to_string(), content);
            db::validate_file(&*db, self.workspace, source_file)
        })
    }
}

impl Default for AnalysisHost {
    fn default() -> Self {
        Self::new()
    }
}
