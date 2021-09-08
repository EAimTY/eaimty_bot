use crate::{context::Context, error::Error};
use carapax::{
    handler,
    methods::SendDice,
    types::{Command, DiceKind},
    HandlerResult,
};

#[handler(command = "/slot")]
pub async fn slot_command_handler(
    context: &Context,
    command: Command,
) -> Result<HandlerResult, Error> {
    let chat_id = command.get_message().get_chat_id();
    let method = SendDice::new(chat_id, DiceKind::SlotMachine);
    context.api.execute(method).await?;
    Ok(HandlerResult::Stop)
}
