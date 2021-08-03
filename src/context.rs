use carapax::{
    Api,
    session::{backend::fs::FilesystemBackend, SessionManager}
};
use tempfile::TempDir;

pub struct Context {
    pub api: Api,
    pub session_manager: SessionManager<FilesystemBackend>,
    pub tmpdir: TempDir
}