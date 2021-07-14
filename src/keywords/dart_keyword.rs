use carapax::{handler, methods::SendDice, types::Message, types::DiceKind, Api, ExecuteError};
use std::{convert::Infallible};

async fn is_dart(_api: &Api, message: &Message) -> Result<bool, Infallible> {
    Ok(message.get_text().map(|text| text.data.contains("飞标")).unwrap_or(false))
}

#[handler(predicate=is_dart)]
pub async fn dart_handler(api: &Api, message: Message) -> Result<(), ExecuteError> {
    let chat_id = message.get_chat_id();
    let method = SendDice::new(chat_id, DiceKind::Darts);
    api.execute(method).await?;
    Ok(())
}
