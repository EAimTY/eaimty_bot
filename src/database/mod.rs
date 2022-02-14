use self::{
    connectfour::SessionPool as ConnectFourSessionPool,
    minesweeper::SessionPool as MinesweeperSessionPool, ocr::SessionPool as OcrSessionPool,
    reversi::SessionPool as ReversiSessionPool, tictactoe::SessionPool as TicTacToeSessionPool,
};
use parking_lot::Mutex;

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
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            connectfour: Mutex::new(ConnectFourSessionPool::new()),
            minesweeper: Mutex::new(MinesweeperSessionPool::new()),
            ocr: Mutex::new(OcrSessionPool::new()),
            reversi: Mutex::new(ReversiSessionPool::new()),
            tictactoe: Mutex::new(TicTacToeSessionPool::new()),
        }
    }
}
