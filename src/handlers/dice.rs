use crate::{context::Context, error::Error};
use carapax::{
    handler,
    methods::SendDice,
    types::{Command, DiceKind, Message},
    HandlerResult,
};

#[handler(command = "/dice")]
pub async fn dice_command_handler(
    context: &Context,
    command: Command,
) -> Result<HandlerResult, Error> {
    let chat_id = command.get_message().get_chat_id();
    let method = SendDice::new(chat_id, DiceKind::Bones);
    context.api.execute(method).await?;
    Ok(HandlerResult::Stop)
}

async fn is_dice(_context: &Context, message: &Message) -> Result<bool, Error> {
    Ok(message
        .get_text()
        .map(|text| text.data.contains("骰子"))
        .unwrap_or(false))
}

#[handler(predicate=is_dice)]
pub async fn dice_keyword_handler(
    context: &Context,
    message: Message,
) -> Result<HandlerResult, Error> {
    let chat_id = message.get_chat_id();
    let method = SendDice::new(chat_id, DiceKind::Bones);
    context.api.execute(method).await?;
    Ok(HandlerResult::Continue)
}
