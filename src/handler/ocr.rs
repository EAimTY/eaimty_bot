use crate::{
    database::ocr::{Language, Session},
    Handler,
};
use anyhow::Result;
use bytes::BufMut;
use futures_util::StreamExt;
use leptess::LepTess;
use tgbot::{
    methods::{AnswerCallbackQuery, EditMessageText, GetFile, SendMessage},
    types::{
        CallbackQuery, Command, File, InlineKeyboardButton, InlineKeyboardButtonKind,
        InlineKeyboardMarkup, Message, MessageData, PhotoSize,
    },
};

pub async fn handle_ocr_command(handler: &Handler, command: &Command) -> Result<bool> {
    if command.get_name() == "/ocr" {
        let msg = command.get_message();

        if let Some(user_id) = msg.get_user_id() {
            let chat_id = msg.get_chat_id();
            let msg_id = msg.id;

            let mut pool = handler.database.ocr.lock();
            let session = Session::new(user_id);
            pool.sessions.insert([chat_id, msg_id], session);

            let send_message = SendMessage::new(chat_id, "请选择 OCR 目标语言")
                .reply_markup(get_lang_select_keyboard())
                .reply_to_message_id(msg_id);

            drop(pool);

            handler.api.execute(send_message).await?;
        }

        return Ok(true);
    }

    Ok(false)
}

pub async fn handle_ocr_callback_query(
    handler: &Handler,
    callback_query: &CallbackQuery,
) -> Result<bool> {
    if let CallbackQuery {
        id,
        from: user,
        message: Some(msg),
        data: Some(cb_data),
        ..
    } = callback_query
    {
        if let (Some(data), Some(cmd_msg)) = (parse_callback_data(cb_data), &msg.reply_to) {
            let cmd_msg_id = cmd_msg.id;
            let msg_id = msg.id;
            let chat_id = msg.get_chat_id();
            let user_id = user.id;

            let mut pool = handler.database.ocr.lock();

            if let Some(session) = pool.sessions.get_mut(&[chat_id, cmd_msg_id]) {
                if session.user == user_id {
                    let edit_message = if let CallbackData::Select(lang) = data {
                        session.lang = Some(lang);
                        session.relay = Some([chat_id, msg_id]);
                        pool.relay.insert([chat_id, msg_id], cmd_msg_id);

                        EditMessageText::new(
                            chat_id,
                            msg_id,
                            format!("目标语言：{lang}，请以需要识别的图片回复此条消息（以图片方式发送）"),
                        )
                        .reply_markup(get_lang_unselect_keyboard())
                    } else {
                        session.lang = None;

                        EditMessageText::new(chat_id, msg_id, "请选择 OCR 目标语言")
                            .reply_markup(get_lang_select_keyboard())
                    };

                    let answer_callback_query = AnswerCallbackQuery::new(id);

                    drop(pool);

                    tokio::try_join!(
                        handler.api.execute(edit_message),
                        handler.api.execute(answer_callback_query)
                    )?;
                } else {
                    drop(pool);

                    let answer_callback_query = AnswerCallbackQuery::new(id)
                        .text("不是命令触发者")
                        .show_alert(true);

                    handler.api.execute(answer_callback_query).await?;
                }
            } else {
                drop(pool);

                let answer_callback_query = AnswerCallbackQuery::new(id)
                    .text("找不到会话")
                    .show_alert(true);

                handler.api.execute(answer_callback_query).await?;
            }

            return Ok(true);
        }
    }

    Ok(false)
}

pub async fn handle_ocr_message(handler: &Handler, message: &Message) -> Result<bool> {
    if let (MessageData::Photo { data, .. }, Some(user_id), Some(relay_msg)) = (
        &message.data,
        message.get_user_id(),
        message.reply_to.as_ref(),
    ) {
        let msg_id = message.id;
        let chat_id = message.get_chat_id();
        let relay_msg_id = relay_msg.id;

        let mut pool = handler.database.ocr.lock();

        if let Some(cmd_msg_id) = pool.relay.get(&[chat_id, relay_msg_id]).copied() {
            if let Some(Session {
                user,
                lang: Some(lang),
                ..
            }) = pool.sessions.get(&[chat_id, cmd_msg_id])
            {
                if user_id == *user {
                    let lang = *lang;

                    pool.sessions.remove(&[chat_id, cmd_msg_id]);
                    pool.relay.remove(&[chat_id, relay_msg_id]);

                    drop(pool);

                    let PhotoSize { file_id, .. } = unsafe {
                        data.iter()
                            .max_by(|a, b| (a.width, a.height).cmp(&(b.width, b.height)))
                            .unwrap_unchecked()
                    };

                    let get_file = GetFile::new(file_id);

                    if let File {
                        file_path: Some(path),
                        ..
                    } = handler.api.execute(get_file).await?
                    {
                        let mut stream = handler.api.download_file(path).await?;

                        let mut pic = Vec::new();

                        while let Some(chunk) = stream.next().await {
                            pic.put_slice(&chunk?);
                        }

                        let mut leptess = LepTess::new(None, lang.as_tesseract_data_str())?;
                        leptess.set_image_from_mem(&pic)?;
                        let res = leptess.get_utf8_text()?;

                        let send_message =
                            SendMessage::new(chat_id, res).reply_to_message_id(msg_id);

                        handler.api.execute(send_message).await?;
                    } else {
                        let send_message =
                            SendMessage::new(chat_id, "图片获取失败").reply_to_message_id(msg_id);

                        handler.api.execute(send_message).await?;
                    }

                    return Ok(true);
                }
            }
        }
    }

    Ok(false)
}

fn get_lang_select_keyboard() -> InlineKeyboardMarkup {
    let vec = Language::iter()
        .map(|lang| {
            vec![InlineKeyboardButton::new(
                lang.to_string(),
                InlineKeyboardButtonKind::CallbackData(format!(
                    "ocr-{}",
                    lang.as_tesseract_data_str()
                )),
            )]
        })
        .collect();

    InlineKeyboardMarkup::from_vec(vec)
}

fn get_lang_unselect_keyboard() -> InlineKeyboardMarkup {
    let vec = vec![vec![InlineKeyboardButton::new(
        "重新选择",
        InlineKeyboardButtonKind::CallbackData(String::from("ocr-unselect")),
    )]];

    InlineKeyboardMarkup::from_vec(vec)
}

enum CallbackData {
    Select(Language),
    Unselect,
}

fn parse_callback_data(data: &str) -> Option<CallbackData> {
    let mut data = data.split('-');

    if let (Some("ocr"), Some(target), None) = (data.next(), data.next(), data.next()) {
        if target == "unselect" {
            return Some(CallbackData::Unselect);
        } else if let Some(lang) = Language::from_tesseract_data_str(target) {
            return Some(CallbackData::Select(lang));
        }
    }

    None
}
