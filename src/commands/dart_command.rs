use carapax::{handler, methods::SendDice, types::Command, types::DiceKind, Api, ExecuteError, HandlerResult};

#[handler(command = "/dart")]
pub async fn handle_dart(api: &Api, command: Command) -> Result<HandlerResult, ExecuteError> {
    let chat_id = command.get_message().get_chat_id();
    let method = SendDice::new(chat_id, DiceKind::Darts);
    api.execute(method).await?;
    Ok(HandlerResult::Stop)
}
