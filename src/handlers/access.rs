use crate::{context::Context, error::ErrorHandler};
use carapax::{HandlerResult, handler, methods::{GetChat, GetMe}, types::{Chat, Message}};

#[handler]
pub async fn group_message_filter(
    context: &Context,
    message: Message,
) -> Result<HandlerResult, ErrorHandler> {
    // 获取信息所属 chat
    let chat = context.api.execute(GetChat::new(message.get_chat_id())).await?;
    // 只处理群组内消息
    if matches!(chat, Chat::Group(_)) || matches!(chat, Chat::Supergroup(_)) {
        // 若 bot_info 为空，尝试获取并存储
        if let None = *context.bot_info.username.read().await {
            let bot = context.api.execute(GetMe).await?;
            let mut bot_info_id = context.bot_info.id.write().await;
            let mut bot_info_username = context.bot_info.username.write().await;
            *bot_info_id = Some(bot.id);
            *bot_info_username = Some(bot.username);
        }
        if let Some(text) = message.get_text() {
            // 检查消息文字是否以“/”起始
            if text.data.starts_with("/") {
                if let Some(bot_username) = context.bot_info.username.read().await.as_deref() {
                    // 检查该条消息 @ 了 bot
                    if text.data.contains(&format!("@{}", bot_username)) {
                        return Ok(HandlerResult::Continue);
                    }
                }
            } else {
                return Ok(HandlerResult::Continue);
            }
        }
    } else {
        return Ok(HandlerResult::Continue);
    }
    Ok(HandlerResult::Stop)
}
