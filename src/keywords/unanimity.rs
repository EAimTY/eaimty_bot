use crate::Context;
use carapax::{
    handler, ExecuteError,
    methods::SendMessage,
    types::Message
};
use std::convert::Infallible;

async fn is_unanimity(_context: &Context, message: &Message) -> Result<bool, Infallible> {
    Ok(message.get_text().map(|text| text.data.contains("有没有")).unwrap_or(false))
}

#[handler(predicate=is_unanimity)]
pub async fn unanimity_keyword_handler(context: &Context, message: Message) -> Result<(), ExecuteError> {
    let chat_id = message.get_chat_id();
    let method = SendMessage::new(chat_id, "没有");
    context.api.execute(method).await?;
    let method = SendMessage::new(chat_id, "没有");
    context.api.execute(method).await?;
    let method = SendMessage::new(chat_id, "没有");
    context.api.execute(method).await?;
    let method = SendMessage::new(chat_id, "好，没有，通过！");
    context.api.execute(method).await?;
    Ok(())
}