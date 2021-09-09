use crate::{context::Context, error::Error};
use carapax::{handler, methods::SendMessage, types::Command, HandlerResult};

#[handler(command = "/start")]
pub async fn start_command_handler(
    context: &Context,
    command: Command,
) -> Result<HandlerResult, Error> {
    let chat_id = command.get_message().get_chat_id();
    let start = r#"
eaimty_bot

个人用 Telegram Bot

获取帮助信息 /help
"#;
    let method = SendMessage::new(chat_id, start);
    context.api.execute(method).await?;
    Ok(HandlerResult::Stop)
}
