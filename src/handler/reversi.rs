use crate::{database::reversi::Session, Context};
use anyhow::Result;
use gamie::reversi::{Player, ReversiError};
use tgbot::{
    methods::{AnswerCallbackQuery, EditMessageText, SendMessage},
    types::{
        CallbackQuery, Command, InlineKeyboardButton, InlineKeyboardButtonKind,
        InlineKeyboardMarkup,
    },
};

pub async fn handle_reversi_command(context: &Context, command: &Command) -> Result<bool> {
    if command.get_name() == "/reversi" {
        let msg = command.get_message();
        let chat_id = msg.get_chat_id();
        let msg_id = msg.id;

        let mut pool = context.database.reversi.lock();

        let reversi = Session::new();

        let send_message = SendMessage::new(chat_id, get_game_info(&reversi))
            .reply_markup(get_inline_keyboard(&reversi))
            .reply_to_message_id(msg_id);

        pool.sessions.insert([chat_id, msg_id], reversi);

        drop(pool);

        context.api.execute(send_message).await?;

        return Ok(true);
    }

    Ok(false)
}

pub async fn handle_reversi_callback_query(
    context: &Context,
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
        if let (Some((row, col)), Some(cmd_msg)) = (parse_callback_data(cb_data), &msg.reply_to) {
            let cmd_msg_id = cmd_msg.id;
            let msg_id = msg.id;
            let chat_id = msg.get_chat_id();
            let user_id = user.id;

            if row < 8 && col < 8 {
                let mut pool = context.database.reversi.lock();

                if let Some(reversi) = pool.sessions.get_mut(&[chat_id, cmd_msg_id]) {
                    let next_player = reversi.game.get_next_player();

                    let is_right_player = if reversi
                        .game
                        .is_valid_move(reversi.game.get_next_player(), row, col)
                        .is_ok()
                    {
                        match next_player {
                            Player::Player0 => {
                                if let Some((player_id, _)) = reversi.player_0 {
                                    player_id == user_id
                                } else {
                                    reversi.player_0 = Some((user_id, user.get_full_name()));
                                    true
                                }
                            }
                            Player::Player1 => {
                                if let Some((player_id, _)) = reversi.player_1 {
                                    player_id == user_id
                                } else {
                                    reversi.player_1 = Some((user_id, user.get_full_name()));
                                    true
                                }
                            }
                        }
                    } else {
                        false
                    };

                    if is_right_player {
                        match reversi.game.place(next_player, row, col) {
                            Ok(()) => {
                                let edit_message =
                                    EditMessageText::new(chat_id, msg_id, get_game_info(reversi))
                                        .reply_markup(get_inline_keyboard(reversi));

                                let answer_callback_query = AnswerCallbackQuery::new(id);

                                if reversi.game.is_ended() {
                                    pool.sessions.remove(&[chat_id, cmd_msg_id]);
                                }

                                drop(pool);

                                tokio::try_join!(
                                    context.api.execute(edit_message),
                                    context.api.execute(answer_callback_query)
                                )?;
                            }
                            Err(ReversiError::OccupiedPosition)
                            | Err(ReversiError::InvalidPosition) => {
                                drop(pool);

                                let answer_callback_query = AnswerCallbackQuery::new(id)
                                    .text("无法在此落子")
                                    .show_alert(true);

                                context.api.execute(answer_callback_query).await?;
                            }
                            _ => unreachable!(),
                        }
                    } else {
                        drop(pool);

                        let answer_callback_query = AnswerCallbackQuery::new(id)
                            .text("不是你的回合")
                            .show_alert(true);

                        context.api.execute(answer_callback_query).await?;
                    }
                } else {
                    drop(pool);

                    let answer_callback_query = AnswerCallbackQuery::new(id)
                        .text("找不到游戏")
                        .show_alert(true);

                    context.api.execute(answer_callback_query).await?;
                }
            }

            return Ok(true);
        }
    }

    Ok(false)
}

fn get_inline_keyboard(reversi: &Session) -> InlineKeyboardMarkup {
    let vec = (0..8)
        .map(|row| {
            (0..8)
                .map(|col| {
                    let text = match reversi.game.get(row, col) {
                        Some(Player::Player0) => "⚫",
                        Some(Player::Player1) => "⚪",
                        None => {
                            if !reversi.game.is_ended()
                                && reversi
                                    .game
                                    .is_valid_move(reversi.game.get_next_player(), row, col)
                                    .is_ok()
                            {
                                "➕"
                            } else {
                                "➖"
                            }
                        }
                    };

                    InlineKeyboardButton::new(
                        text,
                        InlineKeyboardButtonKind::CallbackData(format!("reversi-{row}-{col}")),
                    )
                })
                .collect()
        })
        .collect();

    InlineKeyboardMarkup::from_vec(vec)
}

fn parse_callback_data(data: &str) -> Option<(usize, usize)> {
    let mut data = data.split('-');

    if let (Some("reversi"), Some(row), Some(col), None) =
        (data.next(), data.next(), data.next(), data.next())
    {
        if let (Ok(row), Ok(col)) = (row.parse(), col.parse()) {
            return Some((row, col));
        }
    }

    None
}

fn get_game_info(reversi: &Session) -> String {
    let mut info = String::from("黑白棋\n\n");

    if let Some((_, player_0)) = &reversi.player_0 {
        info.push_str("⚫：");
        info.push_str(player_0);
        info.push('\n');
    }

    if let Some((_, player_1)) = &reversi.player_1 {
        info.push_str("⚪：");
        info.push_str(player_1);
        info.push('\n');
    }

    info.push('\n');

    if reversi.game.is_ended() {
        let mut black_factor = 0;
        let mut white_factor = 0;

        for row in 0..8 {
            for col in 0..8 {
                match reversi.game.get(row, col) {
                    Some(Player::Player0) => black_factor += 1,
                    Some(Player::Player1) => white_factor += 1,
                    None => {}
                };
            }
        }

        info.push_str("⚫：");
        info.push_str(&black_factor.to_string());
        info.push(' ');

        info.push_str("⚪：");
        info.push_str(&white_factor.to_string());
        info.push('\n');

        info.push('\n');

        match reversi.game.get_winner() {
            Some(Player::Player0) => {
                let (_, player_0) = unsafe { reversi.player_0.as_ref().unwrap_unchecked() };
                info.push_str(player_0);
                info.push_str(" 赢了");
            }
            Some(Player::Player1) => {
                let (_, player_1) = unsafe { reversi.player_1.as_ref().unwrap_unchecked() };
                info.push_str(player_1);
                info.push_str(" 赢了");
            }
            None => info.push_str("平局"),
        }
    } else {
        info.push_str("轮到：");

        match reversi.game.get_next_player() {
            Player::Player0 => info.push('⚫'),
            Player::Player1 => info.push('⚪'),
        }
    }

    info
}
