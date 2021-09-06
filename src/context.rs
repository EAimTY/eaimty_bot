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
    pub bot_info: BotInfo,
}

pub struct BotInfo {
    pub id: RwLock<Option<i64>>,
    pub username: RwLock<Option<String>>,
}
