pub mod about;
pub mod agree;
pub mod dart;
pub mod dice;
pub mod help;
pub mod minesweeper;
pub mod ocr;
pub mod othello;
pub mod slot;
pub mod start;
pub mod tictactoe;

use crate::{
    context::{BotInfo, Context},
    error::Error,
};
use async_trait::async_trait;
use carapax::{
    handler,
    methods::{GetMe, SetMyCommands},
    types::{Message, MessageKind, Update},
    ErrorPolicy, HandlerResult,
};
use chrono::Local;

// 错误处理
pub struct ErrorHandler;

#[async_trait]
impl carapax::ErrorHandler for ErrorHandler {
    async fn handle(&mut self, err: carapax::HandlerError) -> ErrorPolicy {
        // 打印错误至 stderr
        eprintln!("[{}]{}", Local::now().format("%F %T %z").to_string(), err);
        ErrorPolicy::Stop
    }
}

// 更新 bot 命令列表
#[handler]
pub async fn set_bot_command(context: &Context, _update: Update) -> Result<HandlerResult, Error> {
    // 在首次收到 update 时向 Telegram 更新 bot 命令列表
    let is_bot_command_set = *context.bot_commands.is_set.read().await;
    if !is_bot_command_set {
        let set_my_commands = SetMyCommands::new(context.bot_commands.command_list.clone());
        context.api.execute(set_my_commands).await?;
        let mut is_bot_command_set = context.bot_commands.is_set.write().await;
        *is_bot_command_set = true;
    }
    Ok(HandlerResult::Continue)
}

// 群组内消息过滤器
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
