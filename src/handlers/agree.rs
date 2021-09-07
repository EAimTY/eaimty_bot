use crate::{context::Context, error::ErrorHandler};
use carapax::{handler, methods::SendMessage, types::Message, HandlerResult};

async fn is_agree(_context: &Context, message: &Message) -> Result<bool, ErrorHandler> {
    Ok(message
        .get_text()
        .map(|text| text.data.contains("有没有"))
        .unwrap_or(false))
}

#[handler(predicate=is_agree)]
pub async fn agree_keyword_handler(
    context: &Context,
    message: Message,
) -> Result<HandlerResult, ErrorHandler> {
    let chat_id = message.get_chat_id();
    let method = SendMessage::new(chat_id, "没有");
    context.api.execute(method).await?;
    let method = SendMessage::new(chat_id, "没有");
    context.api.execute(method).await?;
    let method = SendMessage::new(chat_id, "没有");
    context.api.execute(method).await?;
    let method = SendMessage::new(chat_id, "好，没有，通过！");
    context.api.execute(method).await?;
    Ok(HandlerResult::Continue)
}
