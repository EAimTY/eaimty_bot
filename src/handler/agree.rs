use crate::Context;
use anyhow::Result;
use tgbot::{methods::SendMessage, types::Message};

pub async fn handle_agree_message(context: &Context, message: &Message) -> Result<bool> {
    if let Some(text) = message.get_text() {
        if text.data.contains("有没有") {
            let chat_id = message.get_chat_id();
            let msg_id = message.id;

            let send_message = SendMessage::new(chat_id, "没有").reply_to_message_id(msg_id);
            context.api.execute(send_message).await?;

            let send_message = SendMessage::new(chat_id, "没有").reply_to_message_id(msg_id);
            context.api.execute(send_message).await?;

            let send_message = SendMessage::new(chat_id, "没有").reply_to_message_id(msg_id);
            context.api.execute(send_message).await?;

            let send_message =
                SendMessage::new(chat_id, "好，没有，通过！").reply_to_message_id(msg_id);
            context.api.execute(send_message).await?;

            return Ok(true);
        }
    }

    Ok(false)
}
