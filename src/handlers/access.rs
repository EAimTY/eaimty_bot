use crate::{context::Context, error::Error};
use carapax::{
    handler,
    methods::{GetMe, SetMyCommands},
    types::{Message, MessageKind, Update},
    HandlerResult,
};

#[derive(Clone)]
pub struct BotInfo {
    pub id: i64,
    pub username: String,
}

impl BotInfo {
    pub fn from(id: i64, username: String) -> Self {
        Self { id, username }
    }
}

#[handler]
pub async fn set_bot_command(context: &Context, _update: Update) -> Result<HandlerResult, Error> {
    // 在首次收到 update 时向 Telegram 更新 bot 命令列表
    let is_bot_command_set = context.bot_commands.read().await.is_set;
    if !is_bot_command_set {
        let command_list = (*context.bot_commands.read().await).command_list.clone();
        let set_my_commands = SetMyCommands::new(command_list);
        context.api.execute(set_my_commands).await?;
        let mut bot_commands = context.bot_commands.write().await;
        bot_commands.is_set = true;
    }
    Ok(HandlerResult::Continue)
}

#[handler]
pub async fn group_message_filter(
    context: &Context,
    message: Message,
) -> Result<HandlerResult, Error> {
    // 只处理群组内消息
    if matches!(message.kind, MessageKind::Group { .. })
        || matches!(message.kind, MessageKind::Supergroup { .. })
    {
        // 若 bot_info 为空，尝试获取并存储
        let bot_info = (*context.bot_info.read().await).clone();
        if let None = bot_info {
            let bot = context.api.execute(GetMe).await?;
            let mut bot_info = context.bot_info.write().await;
            *bot_info = Some(BotInfo::from(bot.id, bot.username));
        }
        if let Some(text) = message.get_text() {
            // 检查消息文字是否以“/”起始
            if text.data.starts_with("/") {
                if let Some(bot_info) = context.bot_info.read().await.as_ref() {
                    // 检查该条消息 @ 了 bot
                    if text.data.contains(&format!("@{}", bot_info.username)) {
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
