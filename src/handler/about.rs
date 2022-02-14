use crate::Handler;
use anyhow::Result;
use tgbot::{methods::SendMessage, types::Command};

pub async fn handle_about_command(handler: &Handler, command: &Command) -> Result<bool> {
    if command.get_name() == "/about" {
        let msg = command.get_message();
        let chat_id = msg.get_chat_id();
        let msg_id = msg.id;

        let about = r#"
eaimty_bot

个人用 Telegram Bot

获取帮助信息 /help

源代码：https://github.com/EAimTY/eaimty_bot
"#;

        let send_message = SendMessage::new(chat_id, about).reply_to_message_id(msg_id);
        handler.api.execute(send_message).await?;

        return Ok(true);
    }

    Ok(false)
}
