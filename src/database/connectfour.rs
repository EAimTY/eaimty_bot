use gamie::connect_four::ConnectFour;
use std::collections::HashMap;
use xxhash_rust::xxh3::Xxh3Builder;

pub struct SessionPool {
    pub sessions: HashMap<[i64; 2], Session, Xxh3Builder>,
}

impl SessionPool {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            sessions: HashMap::with_hasher(Xxh3Builder::new()),
        }
    }
}

pub struct Session {
    pub game: ConnectFour,
    pub player_0: Option<(i64, String)>,
    pub player_1: Option<(i64, String)>,
}

impl Session {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            game: unsafe { ConnectFour::new().unwrap_unchecked() },
            player_0: None,
            player_1: None,
        }
    }
}
