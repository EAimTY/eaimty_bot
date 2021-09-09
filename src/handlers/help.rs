use crate::{context::Context, error::Error};
use carapax::{handler, methods::SendMessage, types::Command, HandlerResult};

#[handler(command = "/help")]
pub async fn help_command_handler(
    context: &Context,
    command: Command,
) -> Result<HandlerResult, Error> {
    let chat_id = command.get_message().get_chat_id();
    let mut help = String::from("命令列表：\n");
    for command in &context.bot_commands.command_list {
        help.push_str(&format!(
            "/{} - {}\n",
            command.name(),
            command.description()
        ));
    }
    let method = SendMessage::new(chat_id, help);
    context.api.execute(method).await?;
    Ok(HandlerResult::Stop)
}
