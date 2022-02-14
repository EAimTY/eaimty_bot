use crate::{database::connectfour::Session, Handler};
use anyhow::Result;
use gamie::connect_four::{ConnectFourError, Player};
use tgbot::{
    methods::{AnswerCallbackQuery, EditMessageText, SendMessage},
    types::{
        CallbackQuery, Command, InlineKeyboardButton, InlineKeyboardButtonKind,
        InlineKeyboardMarkup,
    },
};

pub async fn handle_connectfour_command(handler: &Handler, command: &Command) -> Result<bool> {
    if command.get_name() == "/connectfour" {
        let msg = command.get_message();
        let chat_id = msg.get_chat_id();
        let msg_id = msg.id;

        let mut pool = handler.database.connectfour.lock();

        let connectfour = Session::new();

        let send_message = SendMessage::new(chat_id, get_game_info(&connectfour))
            .reply_markup(get_inline_keyboard(&connectfour))
            .reply_to_message_id(msg_id);

        pool.sessions.insert([chat_id, msg_id], connectfour);

        drop(pool);

        handler.api.execute(send_message).await?;

        return Ok(true);
    }

    Ok(false)
}

pub async fn handle_connectfour_callback_query(
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
        if let (Some(col), Some(cmd_msg)) = (parse_callback_data(cb_data), &msg.reply_to) {
            let cmd_msg_id = cmd_msg.id;
            let msg_id = msg.id;
            let chat_id = msg.get_chat_id();
            let user_id = user.id;

            if col < 7 {
                let mut pool = handler.database.connectfour.lock();

                if let Some(connectfour) = pool.sessions.get_mut(&[chat_id, cmd_msg_id]) {
                    let next_player = connectfour.game.get_next_player();

                    let is_right_player = match next_player {
                        Player::Player0 => {
                            if let Some((player_id, _)) = connectfour.player_0 {
                                player_id == user_id
                            } else {
                                connectfour.player_0 = Some((user_id, user.get_full_name()));
                                true
                            }
                        }
                        Player::Player1 => {
                            if let Some((player_id, _)) = connectfour.player_1 {
                                player_id == user_id
                            } else {
                                connectfour.player_1 = Some((user_id, user.get_full_name()));
                                true
                            }
                        }
                    };

                    if is_right_player {
                        match connectfour.game.put(next_player, col) {
                            Ok(()) => {
                                let edit_message = EditMessageText::new(
                                    chat_id,
                                    msg_id,
                                    get_game_info(connectfour),
                                )
                                .reply_markup(get_inline_keyboard(connectfour));

                                let answer_callback_query = AnswerCallbackQuery::new(id);

                                if connectfour.game.is_ended() {
                                    pool.sessions.remove(&[chat_id, cmd_msg_id]);
                                }

                                drop(pool);

                                tokio::try_join!(
                                    handler.api.execute(edit_message),
                                    handler.api.execute(answer_callback_query)
                                )?;
                            }
                            Err(ConnectFourError::ColumnFilled) => {
                                drop(pool);

                                let answer_callback_query = AnswerCallbackQuery::new(id)
                                    .text("无法在此落子")
                                    .show_alert(true);

                                handler.api.execute(answer_callback_query).await?;
                            }
                            _ => unreachable!(),
                        }
                    } else {
                        drop(pool);

                        let answer_callback_query = AnswerCallbackQuery::new(id)
                            .text("不是你的回合")
                            .show_alert(true);

                        handler.api.execute(answer_callback_query).await?;
                    }
                } else {
                    drop(pool);

                    let answer_callback_query = AnswerCallbackQuery::new(id)
                        .text("找不到游戏")
                        .show_alert(true);

                    handler.api.execute(answer_callback_query).await?;
                }
            }

            return Ok(true);
        }
    }

    Ok(false)
}

fn get_inline_keyboard(connectfour: &Session) -> InlineKeyboardMarkup {
    let vec = (0..6)
        .map(|row| {
            (0..7)
                .map(|col| {
                    let text = match connectfour.game.get(row, col) {
                        Some(Player::Player0) => "🔴",
                        Some(Player::Player1) => "🟡",
                        None => {
                            if connectfour.game.is_ended() {
                                "➖"
                            } else {
                                "➕"
                            }
                        }
                    };

                    InlineKeyboardButton::new(
                        text,
                        InlineKeyboardButtonKind::CallbackData(format!("connectfour-{col}")),
                    )
                })
                .collect()
        })
        .collect();

    InlineKeyboardMarkup::from_vec(vec)
}

fn parse_callback_data(data: &str) -> Option<usize> {
    let mut data = data.split('-');

    if let (Some("connectfour"), Some(col), None) = (data.next(), data.next(), data.next()) {
        return col.parse().ok();
    }

    None
}

fn get_game_info(connectfour: &Session) -> String {
    let mut info = String::from("四子棋\n\n");

    if let Some((_, player_0)) = &connectfour.player_0 {
        info.push_str("🔴：");
        info.push_str(player_0);
        info.push('\n');
    }

    if let Some((_, player_1)) = &connectfour.player_1 {
        info.push_str("🟡：");
        info.push_str(player_1);
        info.push('\n');
    }

    info.push('\n');

    if connectfour.game.is_ended() {
        match connectfour.game.get_winner() {
            Some(Player::Player0) => {
                let (_, player_0) = unsafe { connectfour.player_0.as_ref().unwrap_unchecked() };
                info.push_str(player_0);
                info.push_str(" 赢了");
            }
            Some(Player::Player1) => {
                let (_, player_1) = unsafe { connectfour.player_1.as_ref().unwrap_unchecked() };
                info.push_str(player_1);
                info.push_str(" 赢了");
            }
            None => info.push_str("平局"),
        }
    } else {
        info.push_str("轮到：");

        match connectfour.game.get_next_player() {
            Player::Player0 => info.push('🔴'),
            Player::Player1 => info.push('🟡'),
        }
    }

    info
}
