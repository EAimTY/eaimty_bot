use crate::Context;
use carapax::{
    ExecuteError, HandlerResult, handler,
    methods::{AnswerCallbackQuery, EditMessageReplyMarkup, EditMessageText, GetFile, SendMessage},
    session::SessionId,
    types::{CallbackQuery, Command, InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup, MessageData, ReplyMarkup, Update, UpdateKind}
};
use tesseract::ocr;
use tokio::{fs::File, io::AsyncWriteExt, try_join};
use tokio_stream::StreamExt;

fn ocr_select_lang() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::from(vec![
        vec![InlineKeyboardButton::new("English", InlineKeyboardButtonKind::CallbackData(String::from("ocr_lang_eng")))],
        vec![InlineKeyboardButton::new("日本語", InlineKeyboardButtonKind::CallbackData(String::from("ocr_lang_jpn")))],
        vec![InlineKeyboardButton::new("简体中文", InlineKeyboardButtonKind::CallbackData(String::from("ocr_lang_chi_sim")))],
        vec![InlineKeyboardButton::new("繁體中文", InlineKeyboardButtonKind::CallbackData(String::from("ocr_lang_chi_tra")))]
    ])
}

#[handler(command = "/ocr")]
pub async fn ocr_command_handler(context: &Context, command: Command) -> Result<HandlerResult, ExecuteError> {
    let message = command.get_message();
    let chat_id = message.get_chat_id();
    let user_id = message.get_user().unwrap().id;
    let mut session = context.session_manager.get_session(SessionId::new(chat_id, user_id)).unwrap();
    session.set("ocr_trigger_user", &user_id).await.unwrap();
    let method = SendMessage::new(chat_id, "请选择 OCR 目标语言")
        .reply_markup(ReplyMarkup::InlineKeyboardMarkup(ocr_select_lang()));
    context.api.execute(method).await?;
    Ok(HandlerResult::Stop)
}

#[handler]
pub async fn ocr_inlinekeyboard_handler(context: &Context, query: CallbackQuery) -> Result<HandlerResult, ExecuteError> {
    let message = query.message.unwrap();
    let chat_id = message.get_chat_id();
    let user_id = query.from.id;
    let mut session = context.session_manager.get_session(SessionId::new(chat_id, user_id)).unwrap();
    let ocr_trigger_user: i64 = session.get("ocr_trigger_user").await.unwrap().unwrap_or(0);
    if ocr_trigger_user != 0 {
        if user_id == ocr_trigger_user {
            let message_id = message.id;
            let data = query.data.unwrap();
            match data.as_str() {
                "ocr_reselect" => {
                    session.remove("ocr_lang").await.unwrap();
                    let edit_text = EditMessageText::new(chat_id, message_id, "请选择 OCR 目标语言");
                    let edit_reply_markup = EditMessageReplyMarkup::new(chat_id, message_id)
                        .reply_markup(ocr_select_lang());
                    try_join!(context.api.execute(edit_text), context.api.execute(edit_reply_markup))?;
                },
                "ocr_lang_eng" => session.set("ocr_lang", &String::from("eng")).await.unwrap(),
                "ocr_lang_jpn" => session.set("ocr_lang", &String::from("jpn")).await.unwrap(),
                "ocr_lang_chi_sim" => session.set("ocr_lang", &String::from("chi_sim")).await.unwrap(),
                "ocr_lang_chi_tra" => session.set("ocr_lang", &String::from("chi_tra")).await.unwrap(),
                _ => ()
            };
            let ocr_lang: String = session.get("ocr_lang").await.unwrap().unwrap_or(String::from(""));
            if !ocr_lang.is_empty() {
                let edit_text = EditMessageText::new(chat_id, message_id, String::from("目标语言：") + &ocr_lang + "\n请发送需要识别的图片（需以 Telegram 图片方式发送）");
                let edit_reply_markup = EditMessageReplyMarkup::new(chat_id, message_id)
                    .reply_markup(
                        InlineKeyboardMarkup::from(vec![vec![
                            InlineKeyboardButton::new("重新选择", InlineKeyboardButtonKind::CallbackData(String::from("ocr_reselect")))
                        ]])
                    );
                try_join!(context.api.execute(edit_text), context.api.execute(edit_reply_markup))?;
            }
        } else {
            let method = AnswerCallbackQuery::new(query.id).text("您不是命令发起者").show_alert(true);
            context.api.execute(method).await?;
        }
    }
    Ok(HandlerResult::Continue)
}

#[handler]
pub async fn ocr_image_handler(context: &Context, update: Update) -> Result<HandlerResult, ExecuteError> {
    if let UpdateKind::Message(_) = &update.kind {
        let message = update.get_message().unwrap();
        let chat_id = message.get_chat_id();
        let user_id = message.get_user().unwrap().id;
        let mut session = context.session_manager.get_session(SessionId::new(chat_id, user_id)).unwrap();
        let ocr_trigger_user: i64 = session.get("ocr_trigger_user").await.unwrap().unwrap_or(0);
        if user_id == ocr_trigger_user {
            let ocr_lang: String = session.get("ocr_lang").await.unwrap().unwrap_or(String::from(""));
            if !ocr_lang.is_empty() {
                if let MessageData::Photo {data, ..} =  &message.data {
                    let file_id = &data.last().unwrap().file_id;
                    let method = GetFile::new(file_id);
                    let get_photo = context.api.execute(method).await.unwrap();
                    let photo_path = get_photo.file_path.unwrap();
                    let save_path = {
                        let mut path = context.tmpdir.path().to_path_buf().join(file_id);
                        path.set_extension("jpg");
                        path
                    };
                    let mut photo = File::create(&save_path).await.unwrap();
                    let mut stream = context.api.download_file(photo_path).await.unwrap();
                    while let Some(chunk) = stream.next().await {
                        photo.write_all(&chunk.unwrap()).await.unwrap();
                    }
                    let result = ocr(save_path.to_str().unwrap(), ocr_lang.as_str()).unwrap();
                    let method = SendMessage::new(chat_id, result);
                    context.api.execute(method).await?;
                    session.remove("ocr_trigger_user").await.unwrap();
                    session.remove("ocr_lang").await.unwrap();
                }
            }
        }
    }
    Ok(HandlerResult::Continue)
}