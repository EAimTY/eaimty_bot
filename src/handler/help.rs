use crate::Handler;
use anyhow::Result;
use tgbot::{methods::SendMessage, types::Command};

pub async fn handle_help_command(handler: &Handler, command: &Command) -> Result<bool> {
    if command.get_name() == "/help" {
        let msg = command.get_message();
        let chat_id = msg.get_chat_id();
        let msg_id = msg.id;

        let help = r#"
/about - 关于
/connectfour - 玩四子棋
/dart - 掷一枚飞标
/dice - 掷一枚骰子
/minesweeper [棋盘高] [棋盘宽] [地雷数] - 玩扫雷
/ocr - 识别图片中文字
/reversi - 玩黑白棋
/slot - 转一次老虎机
/tictactoe - 玩 Tic-Tac-Toe
/help - 帮助信息
"#;

        let send_message = SendMessage::new(chat_id, help).reply_to_message_id(msg_id);
        handler.api.execute(send_message).await?;

        return Ok(true);
    }

    Ok(false)
}
