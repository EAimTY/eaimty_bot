use crate::{context::Context, error::ErrorHandler};
use carapax::{
    handler,
    methods::SendDice,
    types::{Command, DiceKind, Message},
    HandlerResult,
};

#[handler(command = "/dart")]
pub async fn dart_command_handler(
    context: &Context,
    command: Command,
) -> Result<HandlerResult, ErrorHandler> {
    let chat_id = command.get_message().get_chat_id();
    let method = SendDice::new(chat_id, DiceKind::Darts);
    context.api.execute(method).await?;
    Ok(HandlerResult::Stop)
}

async fn is_dart(_context: &Context, message: &Message) -> Result<bool, ErrorHandler> {
    Ok(message
        .get_text()
        .map(|text| text.data.contains("飞标"))
        .unwrap_or(false))
}

#[handler(predicate=is_dart)]
pub async fn dart_keyword_handler(context: &Context, message: Message) -> Result<(), ErrorHandler> {
    let chat_id = message.get_chat_id();
    let method = SendDice::new(chat_id, DiceKind::Darts);
    context.api.execute(method).await?;
    Ok(())
}
