use crate::{context::Context, error::ErrorHandler};
use carapax::{
    handler, HandlerResult,
    methods::SendDice,
    types::{Command, DiceKind, Message}
};

#[handler(command = "/dice")]
pub async fn dice_command_handler(context: &Context, command: Command) -> Result<HandlerResult, ErrorHandler> {
    let chat_id = command.get_message().get_chat_id();
    let method = SendDice::new(chat_id, DiceKind::Bones);
    context.api.execute(method).await?;
    Ok(HandlerResult::Stop)
}

async fn is_dice(_context: &Context, message: &Message) -> Result<bool, ErrorHandler> {
    Ok(message.get_text().map(|text| text.data.contains("骰子")).unwrap_or(false))
}

#[handler(predicate=is_dice)]
pub async fn dice_keyword_handler(context: &Context, message: Message) -> Result<(), ErrorHandler> {
    let chat_id = message.get_chat_id();
    let method = SendDice::new(chat_id, DiceKind::Bones);
    context.api.execute(method).await?;
    Ok(())
}