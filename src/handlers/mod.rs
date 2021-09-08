pub mod about;
pub mod access;
pub mod agree;
pub mod dart;
pub mod dice;
pub mod minesweeper;
pub mod ocr;
pub mod othello;
pub mod slot;
pub mod start;
pub mod tictactoe;

use async_trait::async_trait;
use carapax::ErrorPolicy;

pub struct ErrorHandler;

#[async_trait]
impl carapax::ErrorHandler for ErrorHandler {
    async fn handle(&mut self, err: carapax::HandlerError) -> ErrorPolicy {
        eprintln!("Error: {}", err);
        ErrorPolicy::Stop
    }
}
