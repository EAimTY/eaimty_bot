use gamie::minesweeper::Minesweeper;
use rand::rngs::OsRng;
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
    pub game: Minesweeper<OsRng>,
    pub players: HashMap<i64, Player, Xxh3Builder>,
    pub start_time: Option<Instant>,
    pub trigger: Option<String>,
    create_time: Instant,
}

impl Session {
    #[allow(clippy::new_without_default)]
    pub fn new(height: usize, width: usize, mines: usize) -> Self {
        Self {
            game: unsafe {
                Minesweeper::new(height, width, mines, OsRng::default()).unwrap_unchecked()
            },
            players: HashMap::with_hasher(Xxh3Builder::new()),
            start_time: None,
            trigger: None,
            create_time: Instant::now(),
        }
    }
}

pub struct Player {
    pub name: String,
    pub step: usize,
}

impl Player {
    pub fn new(name: String) -> Self {
        Self { name, step: 1 }
    }
}
