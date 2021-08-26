use crate::{context::Context, error::ErrorHandler};
use carapax::{handler, methods::SendMessage, types::Command, HandlerResult};

#[handler(command = "/start")]
pub async fn start_command_handler(
    context: &Context,
    command: Command,
) -> Result<HandlerResult, ErrorHandler> {
    let chat_id = command.get_message().get_chat_id();
    let start = "eaimty_bot\n\
                \n\
                功能：\n\
                \n\
                掷飞标 - /dart 或文字内容包含 飞标 - 掷一枚飞标\n\
                掷骰子 - /dice 或文字内容包含 骰子 - 掷一枚骰子\n\
                扫雷 - /minesweeper - 玩扫雷\n\
                OCR - /ocr - 识别图片中文字（基于 Tesseract）\n\
                黑白棋 - /othello - 玩黑白棋\n\
                没有，没有，没有，通过！ - 文字内容包含 “有没有” - 连续发送 3 次 “没有” 和 1 次 “好，没有，通过！”\n\
                老虎机 - /slot - 转一次老虎机\n\
                Tic-Tac-Toe - `/tictactoe` - 玩 Tic-Tac-Toe";
    let method = SendMessage::new(chat_id, start);
    context.api.execute(method).await?;
    Ok(HandlerResult::Stop)
}
