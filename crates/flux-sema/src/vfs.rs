use dashmap::DashMap;
use parking_lot::RwLock;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Unique identifier for a file in the VFS
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(pub u32);

impl FileId {
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

/// Virtual File System for managing source files
/// Supports both disk-based files and in-memory "unsaved" buffers
pub struct Vfs {
    files: DashMap<FileId, Arc<FileData>>,
    path_to_id: DashMap<PathBuf, FileId>,
    next_id: RwLock<u32>,
    std_lib: DashMap<String, Arc<FileData>>,
}

#[derive(Debug, Clone)]
pub struct FileData {
    pub id: FileId,
    pub path: PathBuf,
    pub content: String,
    pub version: u32,
}

impl Vfs {
    pub fn new() -> Self {
        Self {
            files: DashMap::new(),
            path_to_id: DashMap::new(),
            next_id: RwLock::new(1),
            std_lib: DashMap::new(),
        }
    }

    fn next_id(&self) -> FileId {
        let mut next = self.next_id.write();
        let id = *next;
        *next += 1;
        FileId(id)
    }

    /// Load a file from disk or return existing FileId
    pub fn load_file(&self, path: &Path) -> std::io::Result<FileId> {
        // Check if already loaded
        if let Some(id) = self.path_to_id.get(path) {
            return Ok(*id);
        }

        // Read from disk
        let content = std::fs::read_to_string(path)?;
        let file_id = self.next_id();

        let file_data = Arc::new(FileData {
            id: file_id,
            path: path.to_path_buf(),
            content,
            version: 1,
        });

        self.files.insert(file_id, file_data);
        self.path_to_id.insert(path.to_path_buf(), file_id);

        Ok(file_id)
    }

    /// Create or update an in-memory file (for unsaved editor buffers)
    pub fn set_file_content(&self, path: &Path, content: String) -> FileId {
        if let Some(id) = self.path_to_id.get(path) {
            let file_id = *id;
            // Update existing file
            if let Some(mut entry) = self.files.get_mut(&file_id) {
                let old_version = entry.version;
                let new_data = Arc::new(FileData {
                    id: file_id,
                    path: path.to_path_buf(),
                    content,
                    version: old_version + 1,
                });
                *entry = new_data;
            }
            file_id
        } else {
            // Create new file
            let file_id = self.next_id();
            let file_data = Arc::new(FileData {
                id: file_id,
                path: path.to_path_buf(),
                content,
                version: 1,
            });

            self.files.insert(file_id, file_data);
            self.path_to_id.insert(path.to_path_buf(), file_id);
            file_id
        }
    }

    /// Get file content by FileId
    pub fn get_file(&self, file_id: FileId) -> Option<Arc<FileData>> {
        self.files.get(&file_id).map(|entry| entry.clone())
    }

    /// Resolve a module path to FileId (supports virtual std library)
    pub fn resolve_module(&self, module: &str) -> Option<FileId> {
        if let Some(file_data) = self.std_lib.get(module) {
            return Some(file_data.id);
        }
        None
    }

    /// Get file by path
    pub fn get_file_by_path(&self, path: &Path) -> Option<FileId> {
        self.path_to_id.get(path).map(|id| *id)
    }
}

impl Default for Vfs {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vfs_create_file() {
        let vfs = Vfs::new();
        let path = PathBuf::from("test.flux");
        let content = "fn test() {}".to_string();

        let file_id = vfs.set_file_content(&path, content.clone());
        let file_data = vfs.get_file(file_id).unwrap();

        assert_eq!(file_data.content, content);
        assert_eq!(file_data.version, 1);
    }

    #[test]
    fn test_vfs_update_file() {
        let vfs = Vfs::new();
        let path = PathBuf::from("test.flux");

        let file_id1 = vfs.set_file_content(&path, "version 1".to_string());
        let file_id2 = vfs.set_file_content(&path, "version 2".to_string());

        assert_eq!(file_id1, file_id2);

        let file_data = vfs.get_file(file_id2).unwrap();
        assert_eq!(file_data.content, "version 2");
        assert_eq!(file_data.version, 2);
    }
}
