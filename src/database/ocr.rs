use std::{
    collections::HashMap,
    fmt::{Display, Formatter, Result as FmtResult},
    time::{Duration, Instant},
};
use xxhash_rust::xxh3::Xxh3Builder;

pub struct SessionPool {
    pub sessions: HashMap<[i64; 2], Session, Xxh3Builder>,
    pub relay: HashMap<[i64; 2], i64, Xxh3Builder>,
}

impl SessionPool {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            sessions: HashMap::with_hasher(Xxh3Builder::new()),
            relay: HashMap::with_hasher(Xxh3Builder::new()),
        }
    }

    pub fn collect_garbage(&mut self, lifetime: Duration) {
        self.sessions.retain(
            |_,
             Session {
                 create_time, relay, ..
             }| {
                if create_time.elapsed() < lifetime {
                    true
                } else {
                    if let Some(relay) = relay {
                        self.relay.remove(relay);
                    }
                    false
                }
            },
        );
    }
}

pub struct Session {
    pub user: i64,
    pub lang: Option<Language>,
    pub relay: Option<[i64; 2]>,
    create_time: Instant,
}

impl Session {
    #[allow(clippy::new_without_default)]
    pub fn new(user_id: i64) -> Self {
        Self {
            user: user_id,
            lang: None,
            relay: None,
            create_time: Instant::now(),
        }
    }
}

#[derive(Clone, Copy)]
pub enum Language {
    English,
    Japanese,
    SimplifiedChinese,
    TraditionalChinese,
}

impl Language {
    const ENG: &'static str = "eng";
    const JPN: &'static str = "jpn";
    const CHI_SIM: &'static str = "chi_sim";
    const CHI_TRA: &'static str = "chi_tra";

    pub fn iter() -> impl Iterator<Item = Self> {
        [
            Self::English,
            Self::Japanese,
            Self::SimplifiedChinese,
            Self::TraditionalChinese,
        ]
        .into_iter()
    }

    pub fn from_tesseract_data_str(s: &str) -> Option<Self> {
        match s {
            Self::ENG => Some(Self::English),
            Self::JPN => Some(Self::Japanese),
            Self::CHI_SIM => Some(Self::SimplifiedChinese),
            Self::CHI_TRA => Some(Self::TraditionalChinese),
            _ => None,
        }
    }

    pub fn as_tesseract_data_str(&self) -> &'static str {
        match self {
            Self::English => Self::ENG,
            Self::Japanese => Self::JPN,
            Self::SimplifiedChinese => Self::CHI_SIM,
            Self::TraditionalChinese => Self::CHI_TRA,
        }
    }
}

impl Display for Language {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let lang_name = match self {
            Self::English => "English",
            Self::Japanese => "日本語",
            Self::SimplifiedChinese => "简体中文",
            Self::TraditionalChinese => "繁體中文",
        };

        write!(f, "{lang_name}")
    }
}
