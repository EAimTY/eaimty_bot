use crate::{context::Context, error::ErrorHandler};
use carapax::{
    handler,
    methods::GetMe,
    types::{Message, MessageKind},
    HandlerResult,
};

#[handler]
pub async fn group_message_filter(
    context: &Context,
    message: Message,
) -> Result<HandlerResult, ErrorHandler> {
    // 只处理群组内消息
    if matches!(message.kind, MessageKind::Group { .. })
        || matches!(message.kind, MessageKind::Supergroup { .. })
    {
        // 若 bot_info 为空，尝试获取并存储
        let bot_info_username = (*context.bot_info.username.read().await).clone();
        if let None = bot_info_username {
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
