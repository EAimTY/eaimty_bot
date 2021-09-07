use crate::{context::Context, error::ErrorHandler};
use carapax::{
    handler,
    methods::{AnswerCallbackQuery, EditMessageText, GetFile, SendMessage},
    session::SessionId,
    types::{
        CallbackQuery, Command, InlineKeyboardButton, InlineKeyboardButtonKind,
        InlineKeyboardMarkup, MessageData, ReplyMarkup, Update, UpdateKind,
    },
    HandlerResult,
};
use leptess::LepTess;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::{fs::File, io::AsyncWriteExt};
use tokio_stream::StreamExt;

// 支持的 OCR 语言列表类型
pub struct OcrLangs {
    langs: HashMap<String, String>,
}

impl OcrLangs {
    fn new() -> Self {
        Self {
            langs: HashMap::new(),
        }
    }

    // 添加 OCR 语言
    fn add(&mut self, lang: &str, name: &str) {
        self.langs.insert(lang.to_string(), name.to_string());
    }

    pub fn init() -> Self {
        // 定义 OCR 语言列表，在此处添加语言，参数一为 Tesseract 语言包名称，参数二为语言显示名称
        let mut ocr_langs = OcrLangs::new();
        ocr_langs.add("eng", "English");
        ocr_langs.add("jpn", "日本語");
        ocr_langs.add("chi_sim", "简体中文");
        ocr_langs.add("chi_tra", "繁體中文");
        ocr_langs
    }

    // 获取语言列表按钮
    fn get_langs_keyboard(&self) -> InlineKeyboardMarkup {
        let mut keyboad: Vec<Vec<InlineKeyboardButton>> = Vec::new();
        for (lang, name) in &self.langs {
            keyboad.push(vec![InlineKeyboardButton::new(
                name,
                InlineKeyboardButtonKind::CallbackData(format!("ocr-{}", lang)),
            )]);
        }
        InlineKeyboardMarkup::from(keyboad)
    }

    // 获取重新选择按钮
    fn get_reselect_keyboard(&self) -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::from(vec![vec![InlineKeyboardButton::new(
            "重新选择",
            InlineKeyboardButtonKind::CallbackData(String::from("ocr-reselect")),
        )]])
    }

    // 尝试解析 callback data，返回用户目标操作
    fn try_parse_callback(&self, data: String) -> Option<Operation> {
        if data.starts_with("ocr-") {
            let mut data = data[4..].split('-');
            if let Some(lang) = data.next() {
                if let None = data.next() {
                    if lang == "reselect" {
                        return Some(Operation::Reselect);
                    } else if self.langs.contains_key(lang) {
                        return Some(Operation::Select(lang.to_string()));
                    }
                }
            }
        }
        None
    }

    // 获取语言的显示名称
    fn get_lang_name(&self, lang: &str) -> String {
        self.langs
            .get(lang)
            .unwrap_or(&String::from(""))
            .to_string()
    }
}

// 用户触发的操作类型
enum Operation {
    Select(String),
    Reselect,
}

// 用于在 session 中存储 OCR 状态的类型
#[derive(Serialize, Deserialize)]
struct Ocr {
    lang: Option<String>,
}

impl Ocr {
    fn new() -> Self {
        Self { lang: None }
    }

    fn get(&self) -> Option<String> {
        self.lang.clone()
    }

    // 设置或取消目标语言
    fn set(&mut self, operation: &Operation) {
        match operation {
            Operation::Select(lang) => self.lang = Some(lang.to_string()),
            Operation::Reselect => self.lang = None,
        }
    }
}

#[handler(command = "/ocr")]
pub async fn ocr_command_handler(
    context: &Context,
    command: Command,
) -> Result<HandlerResult, ErrorHandler> {
    let message = command.get_message();
    let chat_id = message.get_chat_id();
    if let Some(user) = message.get_user() {
        let user_id = user.id;
        // 在 session 中存储 OCR 状态
        let mut session = context
            .session_manager
            .get_session(SessionId::new(chat_id, user_id))?;
        session.set("ocr", &Ocr::new()).await?;
        // 发送语言选择信息
        let method = SendMessage::new(chat_id, "请选择 OCR 目标语言").reply_markup(
            ReplyMarkup::InlineKeyboardMarkup(context.ocr_langs.get_langs_keyboard()),
        );
        context.api.execute(method).await?;
    }
    Ok(HandlerResult::Stop)
}

#[handler]
pub async fn ocr_inlinekeyboard_handler(
    context: &Context,
    query: CallbackQuery,
) -> Result<HandlerResult, ErrorHandler> {
    // 检查非空 query
    if let Some(data) = query.data {
        // 尝试 parse callback data
        if let Some(operation) = context.ocr_langs.try_parse_callback(data) {
            let message = query.message.unwrap();
            let chat_id = message.get_chat_id();
            let user_id = query.from.id;
            // 从 session 获取 OCR 状态
            let mut session = context
                .session_manager
                .get_session(SessionId::new(chat_id, user_id))?;
            let ocr: Option<Ocr> = session.get("ocr").await?;
            // 检查该用户是否触发过 /ocr 指令
            if let Some(mut ocr) = ocr {
                // 用户触发过指令，保存用户的目标操作
                ocr.set(&operation);
                session.set("ocr", &ocr).await?;
                let method: EditMessageText;
                // 检查用户目标操作是否是选择语言
                if let Operation::Select(lang) = operation {
                    // 用户目标操作是选择语言
                    method = EditMessageText::new(
                        chat_id,
                        message.id,
                        format!("OCR 目标语言：{}\n\n请发送需要识别的图片（需以 Telegram 图片方式发送）", context.ocr_langs.get_lang_name(&lang)),
                    )
                    .reply_markup(context.ocr_langs.get_reselect_keyboard());
                } else {
                    // 用户目标操作是重新选择语言
                    method = EditMessageText::new(chat_id, message.id, "请选择 OCR 目标语言：")
                        .reply_markup(context.ocr_langs.get_langs_keyboard());
                }
                context.api.execute(method).await?;
                // 回应 callback
                let method = AnswerCallbackQuery::new(query.id);
                context.api.execute(method).await?;
            } else {
                // 用户没有触发过指令，以错误提示回应 callback
                let method = AnswerCallbackQuery::new(query.id)
                    .text("如需图片识别，请使用 /ocr 命令")
                    .show_alert(true);
                context.api.execute(method).await?;
            }
            return Ok(HandlerResult::Stop);
        }
    }
    Ok(HandlerResult::Continue)
}

#[handler]
pub async fn ocr_image_handler(
    context: &Context,
    update: Update,
) -> Result<HandlerResult, ErrorHandler> {
    // 检查 Update 类型为 Message
    if let UpdateKind::Message(message) = &update.kind {
        // 检查 Message 类型为 Photo 并获取 photo data
        if let MessageData::Photo { data, .. } = &message.data {
            let chat_id = message.get_chat_id();
            // 获取 Photo 发送者
            if let Some(user) = message.get_user() {
                // 从 session 获取 OCR 状态
                let mut session = context
                    .session_manager
                    .get_session(SessionId::new(chat_id, user.id))?;
                let ocr: Option<Ocr> = session.get("ocr").await?;
                // 检查该用户是否触发过 /ocr 指令
                if let Some(ocr) = ocr {
                    // 检查该用户是否已经选择过 OCR 目标语言
                    if let Some(lang) = ocr.get() {
                        // 获取图片 URL
                        let file_id = &data.last().unwrap().file_id;
                        let method = GetFile::new(file_id);
                        let photo = context.api.execute(method).await?;
                        let photo_url = photo.file_path.unwrap();
                        // 下载图片
                        let photo_save_path = {
                            let mut path = context.tmpdir.path().to_path_buf().join(file_id);
                            path.set_extension("jpg");
                            path
                        };
                        let mut photo = File::create(&photo_save_path).await?;
                        let mut stream = context.api.download_file(photo_url).await?;
                        while let Some(chunk) = stream.next().await {
                            photo.write_all(&chunk?).await?;
                        }
                        // 使用 LepTess 识别图片
                        let mut leptess = LepTess::new(None, &lang)?;
                        leptess.set_image(photo_save_path)?;
                        let result = leptess.get_utf8_text().unwrap_or(String::from("识别失败"));
                        // 发送结果
                        let method = SendMessage::new(chat_id, result);
                        context.api.execute(method).await?;
                        return Ok(HandlerResult::Stop);
                    }
                }
            }
        }
    }
    Ok(HandlerResult::Continue)
}
