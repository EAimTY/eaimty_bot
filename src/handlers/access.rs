use crate::{context::Context, error::ErrorHandler};
use carapax::{
    handler,
    methods::GetMe,
    types::{Chat, Message},
    HandlerResult,
};

static mut BOT_NAME: Option<String> = None;

#[handler]
pub async fn group_message_filter(
    context: &Context,
    message: Message,
) -> Result<HandlerResult, ErrorHandler> {
    if let Some(chat) = &message.sender_chat {
        if matches!(chat, Chat::Group(_)) || matches!(chat, Chat::Supergroup(_)) {
            unsafe {
                if let None = &BOT_NAME {
                    let bot = context.api.execute(GetMe).await?;
                    BOT_NAME = Some(bot.username);
                }
                if let Some(text) = message.get_text() {
                    if !text.data.contains(&BOT_NAME.clone().unwrap()) {
                        return Ok(HandlerResult::Stop);
                    }
                }
            }
        }
    }
    Ok(HandlerResult::Continue)
}
