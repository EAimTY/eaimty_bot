use crate::Handler;
use anyhow::Result;
use tgbot::{
    methods::SendDice,
    types::{Command, DiceKind, Message},
};

pub async fn handle_slot_message(handler: &Handler, message: &Message) -> Result<bool> {
    if let Some(text) = message.get_text() {
        if text.data.contains("老虎机") {
            let chat_id = message.get_chat_id();
            let msg_id = message.id;

            let send_dice =
                SendDice::new(chat_id, DiceKind::SlotMachine).reply_to_message_id(msg_id);
            handler.api.execute(send_dice).await?;

            return Ok(true);
        }
    }

    Ok(false)
}

pub async fn handle_slot_command(handler: &Handler, command: &Command) -> Result<bool> {
    if command.get_name() == "/slot" {
        let msg = command.get_message();
        let chat_id = msg.get_chat_id();
        let msg_id = msg.id;

        let send_dice = SendDice::new(chat_id, DiceKind::SlotMachine).reply_to_message_id(msg_id);
        handler.api.execute(send_dice).await?;

        return Ok(true);
    }

    Ok(false)
}
