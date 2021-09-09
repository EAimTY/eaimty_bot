use crate::{context::Context, error::Error};
use carapax::{handler, methods::SendMessage, types::Command, HandlerResult};

#[handler(command = "/about")]
pub async fn about_command_handler(
    context: &Context,
    command: Command,
) -> Result<HandlerResult, Error> {
    let chat_id = command.get_message().get_chat_id();
    let about = r#"
eaimty_bot

个人用 Telegram Bot

获取帮助信息 /help

源代码：https://github.com/EAimTY/eaimty_bot
"#;
    let method = SendMessage::new(chat_id, about);
    context.api.execute(method).await?;
    Ok(HandlerResult::Stop)
}
