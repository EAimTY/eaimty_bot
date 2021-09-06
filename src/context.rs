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
    // 惰性获取并缓存 bot 相关信息
    pub bot_info: RwLock<Option<BotInfo>>,
}

#[derive(Clone)]
pub struct BotInfo {
    pub id: i64,
    pub username: String,
}

impl BotInfo {
    pub fn from(id: i64, username: String) -> Self {
        Self { id, username }
    }
}
