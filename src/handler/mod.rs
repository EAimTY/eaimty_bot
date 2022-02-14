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

pub struct Context {
    api: Api,
    database: Database,
    username: String,
}

#[derive(Clone)]
pub struct Handler(Arc<Context>);

impl Handler {
    pub fn new(api: Api, database: Database, username: String) -> Self {
        Self(Arc::new(Context {
            api,
            database,
            username,
        }))
    }
}

impl UpdateHandler for Handler {
    type Future = BoxFuture<'static, ()>;

    fn handle(&self, update: Update) -> Self::Future {
        let cx = self.0.clone();

        Box::pin(async move {
            match handle_update(cx, update).await {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("{err}");
                }
            }
        })
    }
}

async fn handle_update(context: Arc<Context>, update: Update) -> Result<()> {
    match update.kind {
        UpdateKind::Message(msg) => handle_message(context, msg).await?,
        UpdateKind::CallbackQuery(query) => handle_callback_query(context, query).await?,
        _ => {}
    }

    Ok(())
}

async fn handle_message(context: Arc<Context>, message: Message) -> Result<()> {
    if !matches!(message.kind, MessageKind::Private { .. })
        && !message
            .get_text()
            .map_or(false, |text| text.data.contains(&context.username))
    {
        return Ok(());
    }

    if agree::handle_agree_message(&context, &message).await?
        || dart::handle_dart_message(&context, &message).await?
        || dice::handle_dice_message(&context, &message).await?
        || ocr::handle_ocr_message(&context, &message).await?
        || slot::handle_slot_message(&context, &message).await?
    {
        return Ok(());
    }

    if let Ok(cmd) = Command::try_from(message) {
        if about::handle_about_command(&context, &cmd).await?
            || connectfour::handle_connectfour_command(&context, &cmd).await?
            || dart::handle_dart_command(&context, &cmd).await?
            || dice::handle_dice_command(&context, &cmd).await?
            || help::handle_help_command(&context, &cmd).await?
            || minesweeper::handle_minesweeper_command(&context, &cmd).await?
            || ocr::handle_ocr_command(&context, &cmd).await?
            || reversi::handle_reversi_command(&context, &cmd).await?
            || slot::handle_slot_command(&context, &cmd).await?
            || start::handle_start_command(&context, &cmd).await?
            || tictactoe::handle_tictactoe_command(&context, &cmd).await?
        {
            return Ok(());
        }
    }

    Ok(())
}

async fn handle_callback_query(context: Arc<Context>, callback_query: CallbackQuery) -> Result<()> {
    if connectfour::handle_connectfour_callback_query(&context, &callback_query).await?
        || minesweeper::handle_minesweeper_callback_query(&context, &callback_query).await?
        || ocr::handle_ocr_callback_query(&context, &callback_query).await?
        || reversi::handle_reversi_callback_query(&context, &callback_query).await?
        || tictactoe::handle_tictactoe_callback_query(&context, &callback_query).await?
    {
        return Ok(());
    }

    Ok(())
}
