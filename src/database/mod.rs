use self::{
    connectfour::SessionPool as ConnectFourSessionPool,
    minesweeper::SessionPool as MinesweeperSessionPool, ocr::SessionPool as OcrSessionPool,
    reversi::SessionPool as ReversiSessionPool, tictactoe::SessionPool as TicTacToeSessionPool,
};
use parking_lot::Mutex;
use std::{future::Future, sync::Arc, time::Duration};
use tokio::time;

pub mod connectfour;
pub mod minesweeper;
pub mod ocr;
pub mod reversi;
pub mod tictactoe;

pub struct Database {
    pub connectfour: Mutex<ConnectFourSessionPool>,
    pub minesweeper: Mutex<MinesweeperSessionPool>,
    pub ocr: Mutex<OcrSessionPool>,
    pub reversi: Mutex<ReversiSessionPool>,
    pub tictactoe: Mutex<TicTacToeSessionPool>,
}

impl Database {
    pub fn init(lifetime: Duration, gc_period: Duration) -> (Arc<Self>, impl Future<Output = ()>) {
        let db = Arc::new(Self {
            connectfour: Mutex::new(ConnectFourSessionPool::new()),
            minesweeper: Mutex::new(MinesweeperSessionPool::new()),
            ocr: Mutex::new(OcrSessionPool::new()),
            reversi: Mutex::new(ReversiSessionPool::new()),
            tictactoe: Mutex::new(TicTacToeSessionPool::new()),
        });

        (db.clone(), db.collect_garbage(lifetime, gc_period))
    }

    async fn collect_garbage(self: Arc<Self>, lifetime: Duration, gc_period: Duration) {
        let mut interval = time::interval(gc_period);

        loop {
            interval.tick().await;

            self.connectfour.lock().collect_garbage(lifetime);
            self.minesweeper.lock().collect_garbage(lifetime);
            self.ocr.lock().collect_garbage(lifetime);
            self.reversi.lock().collect_garbage(lifetime);
            self.tictactoe.lock().collect_garbage(lifetime);
        }
    }
}
