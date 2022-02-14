use crate::Database;
use anyhow::Result;
use futures_util::future::BoxFuture;
use std::sync::Arc;
use tgbot::{
    types::{CallbackQuery, Command, Message, MessageKind, Update, UpdateKind},
    Api, UpdateHandler,
};

mod about;
mod agree;
mod connectfour;
mod dart;
mod dice;
mod help;
mod minesweeper;
mod ocr;
mod reversi;
mod slot;
mod start;
mod tictactoe;

#[derive(Clone)]
pub struct Handler {
    api: Arc<Api>,
    database: Arc<Database>,
    username: Arc<String>,
}

impl Handler {
    pub fn new(database: Arc<Database>, api: Api, username: String) -> Self {
        Self {
            database,
            api: Arc::new(api),
            username: Arc::new(username),
        }
    }
}

impl UpdateHandler for Handler {
    type Future = BoxFuture<'static, ()>;

    fn handle(&self, update: Update) -> Self::Future {
        let handler = self.clone();

        Box::pin(async move {
            match handle_update(handler, update).await {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("{err}");
                }
            }
        })
    }
}

async fn handle_update(handler: Handler, update: Update) -> Result<()> {
    match update.kind {
        UpdateKind::Message(msg) => handle_message(handler, msg).await?,
        UpdateKind::CallbackQuery(query) => handle_callback_query(handler, query).await?,
        _ => {}
    }

    Ok(())
}

async fn handle_message(handler: Handler, message: Message) -> Result<()> {
    if !matches!(message.kind, MessageKind::Private { .. })
        && !message
            .get_text()
            .map_or(false, |text| text.data.contains(handler.username.as_ref()))
    {
        return Ok(());
    }

    if agree::handle_agree_message(&handler, &message).await?
        || dart::handle_dart_message(&handler, &message).await?
        || dice::handle_dice_message(&handler, &message).await?
        || ocr::handle_ocr_message(&handler, &message).await?
        || slot::handle_slot_message(&handler, &message).await?
    {
        return Ok(());
    }

    if let Ok(cmd) = Command::try_from(message) {
        if about::handle_about_command(&handler, &cmd).await?
            || connectfour::handle_connectfour_command(&handler, &cmd).await?
            || dart::handle_dart_command(&handler, &cmd).await?
            || dice::handle_dice_command(&handler, &cmd).await?
            || help::handle_help_command(&handler, &cmd).await?
            || minesweeper::handle_minesweeper_command(&handler, &cmd).await?
            || ocr::handle_ocr_command(&handler, &cmd).await?
            || reversi::handle_reversi_command(&handler, &cmd).await?
            || slot::handle_slot_command(&handler, &cmd).await?
            || start::handle_start_command(&handler, &cmd).await?
            || tictactoe::handle_tictactoe_command(&handler, &cmd).await?
        {
            return Ok(());
        }
    }

    Ok(())
}

async fn handle_callback_query(handler: Handler, callback_query: CallbackQuery) -> Result<()> {
    if connectfour::handle_connectfour_callback_query(&handler, &callback_query).await?
        || minesweeper::handle_minesweeper_callback_query(&handler, &callback_query).await?
        || ocr::handle_ocr_callback_query(&handler, &callback_query).await?
        || reversi::handle_reversi_callback_query(&handler, &callback_query).await?
        || tictactoe::handle_tictactoe_callback_query(&handler, &callback_query).await?
    {
        return Ok(());
    }

    Ok(())
}
