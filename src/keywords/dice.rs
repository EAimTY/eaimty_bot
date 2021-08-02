use crate::Context;
use carapax::{
    ExecuteError, handler,
    methods::SendDice,
    types::{DiceKind, Message}
};
use std::convert::Infallible;

async fn is_dice(_context: &Context, message: &Message) -> Result<bool, Infallible> {
    Ok(message.get_text().map(|text| text.data.contains("骰子")).unwrap_or(false))
}

#[handler(predicate=is_dice)]
pub async fn dice_keyword_handler(context: &Context, message: Message) -> Result<(), ExecuteError> {
    let chat_id = message.get_chat_id();
    let method = SendDice::new(chat_id, DiceKind::Bones);
    context.api.execute(method).await?;
    Ok(())
}