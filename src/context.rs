use crate::handlers::access;
use carapax::{
    session::{backend::fs::FilesystemBackend, SessionManager},
    Api,
};
use tempfile::TempDir;
use tokio::sync::RwLock;

pub struct Context {
    pub api: Api,
    pub session_manager: SessionManager<FilesystemBackend>,
    pub tmpdir: TempDir,
    pub bot_info: RwLock<Option<access::BotInfo>>,
}

impl Context {
    pub fn new(
        api: Api,
        session_manager: SessionManager<FilesystemBackend>,
        tmpdir: TempDir,
    ) -> Self {
        Self {
            api,
            session_manager,
            tmpdir,
            bot_info: RwLock::new(None),
        }
    }
}
