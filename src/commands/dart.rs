use crate::Context;
use carapax::{
    ExecuteError, HandlerResult, handler,
    methods::SendDice,
    types::{Command, DiceKind}
};

#[handler(command = "/dart")]
pub async fn dart_command_handler(context: &Context, command: Command) -> Result<HandlerResult, ExecuteError> {
    let chat_id = command.get_message().get_chat_id();
    let method = SendDice::new(chat_id, DiceKind::Darts);
    context.api.execute(method).await?;
    Ok(HandlerResult::Stop)
}