use crate::{context::Context, error::ErrorHandler};
use carapax::{handler, HandlerResult, methods::SendMessage, types::Command};

#[handler(command = "/about")]
pub async fn about_command_handler(context: &Context, command: Command) -> Result<HandlerResult, ErrorHandler> {
    let chat_id = command.get_message().get_chat_id();
    let about = "eaimty_bot\n\
                \n\
                个人用 Telegram Bot，使用 Rust 编写，基于 carapax\n\
                \n\
                https://github.com/EAimTY/eaimty_bot\n\
                \n\
                以 The GNU General Public License v3.0 许可开源";
    let method = SendMessage::new(chat_id, about);
    context.api.execute(method).await?;
    Ok(HandlerResult::Stop)
}