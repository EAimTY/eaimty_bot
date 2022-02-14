use gamie::reversi::Reversi;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
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

    pub fn collect_garbage(&mut self, lifetime: Duration) {
        self.sessions
            .retain(|_, Session { create_time, .. }| create_time.elapsed() < lifetime);
    }
}

pub struct Session {
    pub game: Reversi,
    pub player_0: Option<(i64, String)>,
    pub player_1: Option<(i64, String)>,
    create_time: Instant,
}

impl Session {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            game: unsafe { Reversi::new().unwrap_unchecked() },
            player_0: None,
            player_1: None,
            create_time: Instant::now(),
        }
    }
}
