use crate::{context::Context, error::ErrorHandler};
use carapax::{
    handler, HandlerResult,
    methods::{AnswerCallbackQuery, EditMessageReplyMarkup, EditMessageText, GetFile, SendMessage},
    session::SessionId,
    types::{CallbackQuery, Command, InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup, MessageData, ReplyMarkup, Update, UpdateKind}
};
use tesseract::ocr;
use tokio::{fs::File, io::AsyncWriteExt, try_join};
use tokio_stream::StreamExt;

struct OcrLang {
    name: String,
    title: String
}

fn ocr_select_lang() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::from(vec![
        vec![InlineKeyboardButton::new("English", InlineKeyboardButtonKind::CallbackData(String::from("ocr_lang_eng")))],
        vec![InlineKeyboardButton::new("日本語", InlineKeyboardButtonKind::CallbackData(String::from("ocr_lang_jpn")))],
        vec![InlineKeyboardButton::new("简体中文", InlineKeyboardButtonKind::CallbackData(String::from("ocr_lang_chi_sim")))],
        vec![InlineKeyboardButton::new("繁體中文", InlineKeyboardButtonKind::CallbackData(String::from("ocr_lang_chi_tra")))]
    ])
}

#[handler(command = "/ocr")]
pub async fn ocr_command_handler(context: &Context, command: Command) -> Result<HandlerResult, ErrorHandler> {
    let message = command.get_message();
    let chat_id = message.get_chat_id();
    if let Some(user) = message.get_user() {
        let user_id = user.id;
        let mut session = context.session_manager.get_session(SessionId::new(chat_id, user_id))?;
        session.set("ocr_trigger_user", &user_id).await?;
        let method = SendMessage::new(chat_id, "请选择 OCR 目标语言")
            .reply_markup(ReplyMarkup::InlineKeyboardMarkup(ocr_select_lang()));
        context.api.execute(method).await?;
    }
    Ok(HandlerResult::Stop)
}

#[handler]
pub async fn ocr_inlinekeyboard_handler(context: &Context, query: CallbackQuery) -> Result<HandlerResult, ErrorHandler> {
    let message = query.message;
    if let Some(message) = message {
        let chat_id = message.get_chat_id();
        let user_id = query.from.id;
        let mut session = context.session_manager.get_session(SessionId::new(chat_id, user_id))?;
        let ocr_trigger_user: Option<i64> = session.get("ocr_trigger_user").await?;
        if let Some(ocr_trigger_user) = ocr_trigger_user {
            if user_id == ocr_trigger_user {
                let message_id = message.id;
                let data = query.data;
                if let Some(data) = data {
                    let mut lang: Option<OcrLang> = None;
                    match data.as_str() {
                        "ocr_reselect" => {
                            session.remove("ocr_lang").await?;
                            let edit_message = EditMessageText::new(chat_id, message_id, "请选择 OCR 目标语言");
                            let edit_reply_markup = EditMessageReplyMarkup::new(chat_id, message_id)
                                .reply_markup(ocr_select_lang());
                            let answer_callback_query = AnswerCallbackQuery::new(&query.id);
                            try_join!(context.api.execute(edit_message), context.api.execute(edit_reply_markup), context.api.execute(answer_callback_query))?;
                        },
                        "ocr_lang_eng" => lang = Some(OcrLang{name: String::from("eng"), title: String::from("English")}),
                        "ocr_lang_jpn" => lang = Some(OcrLang{name: String::from("jpn"), title: String::from("日本語")}),
                        "ocr_lang_chi_sim" => lang = Some(OcrLang{name: String::from("chi_sim"), title: String::from("简体中文")}),
                        "ocr_lang_chi_tra" => lang = Some(OcrLang{name: String::from("chi_tra"), title: String::from("繁體中文")}),
                        _ => ()
                    };
                    if let Some(lang) = lang {
                        session.set("ocr_lang", &lang.name).await?;
                        let edit_message = EditMessageText::new(chat_id, message_id, String::from("目标语言：") + &lang.title + "\n请发送需要识别的图片（需以 Telegram 图片方式发送）");
                        let edit_reply_markup = EditMessageReplyMarkup::new(chat_id, message_id)
                            .reply_markup(
                                InlineKeyboardMarkup::from(vec![vec![
                                    InlineKeyboardButton::new("重新选择", InlineKeyboardButtonKind::CallbackData(String::from("ocr_reselect")))
                                ]])
                            );
                        let answer_callback_query = AnswerCallbackQuery::new(&query.id);
                        try_join!(context.api.execute(edit_message), context.api.execute(edit_reply_markup), context.api.execute(answer_callback_query))?;
                    }
                }
            } else {
                let method = AnswerCallbackQuery::new(query.id)
                    .text("您不是命令发起者")
                    .show_alert(true);
                context.api.execute(method).await?;
            }
        }
    }
    Ok(HandlerResult::Continue)
}

#[handler]
pub async fn ocr_image_handler(context: &Context, update: Update) -> Result<HandlerResult, ErrorHandler> {
    if let UpdateKind::Message(_) = &update.kind {
        let message = update.get_message().unwrap();
        let chat_id = message.get_chat_id();
        if let Some(user) = message.get_user() {
            let user_id = user.id;
            let mut session = context.session_manager.get_session(SessionId::new(chat_id, user_id))?;
            let ocr_trigger_user: Option<i64> = session.get("ocr_trigger_user").await?;
            if let Some(ocr_trigger_user) = ocr_trigger_user {
                if user_id == ocr_trigger_user {
                    let ocr_lang: Option<String> = session.get("ocr_lang").await?;
                    if let Some(ocr_lang) = ocr_lang {
                        if let MessageData::Photo {data, ..} =  &message.data {
                            session.remove("ocr_trigger_user").await?;
                            session.remove("ocr_lang").await?;
                            let file_id = &data.last().unwrap().file_id;
                            let method = GetFile::new(file_id);
                            let get_photo = context.api.execute(method).await?;
                            let photo_path = get_photo.file_path.unwrap();
                            let save_path = {
                                let mut path = context.tmpdir.path().to_path_buf().join(file_id);
                                path.set_extension("jpg");
                                path
                            };
                            let mut photo = File::create(&save_path).await?;
                            let mut stream = context.api.download_file(photo_path).await?;
                            while let Some(chunk) = stream.next().await {
                                photo.write_all(&chunk?).await?;
                            }
                            let result = ocr(save_path.to_str().unwrap_or(""), ocr_lang.as_str())?;
                            let method = SendMessage::new(chat_id, result);
                            context.api.execute(method).await?;
                        }
                    }
                }
            }
        }
    }
    Ok(HandlerResult::Continue)
}