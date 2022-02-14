use crate::{
    database::minesweeper::{Player, Session},
    Context,
};
use anyhow::Result;
use gamie::minesweeper::{Cell, Status};
use std::{collections::hash_map::Entry, time::Instant};
use tgbot::{
    methods::{AnswerCallbackQuery, EditMessageText, SendMessage},
    types::{
        CallbackQuery, Command, InlineKeyboardButton, InlineKeyboardButtonKind,
        InlineKeyboardMarkup,
    },
};

pub async fn handle_minesweeper_command(context: &Context, command: &Command) -> Result<bool> {
    if command.get_name() == "/minesweeper" {
        let msg = command.get_message();
        let chat_id = msg.get_chat_id();
        let msg_id = msg.id;

        let args = command
            .get_args()
            .iter()
            .filter(|arg| !arg.contains(&context.username));

        fn get_args<'a>(
            mut args: impl Iterator<Item = &'a String>,
        ) -> Option<(usize, usize, usize)> {
            match (args.next(), args.next(), args.next(), args.next()) {
                (Some(height), Some(width), Some(mines), None) => {
                    if let (Ok(height), Ok(width), Ok(mines)) =
                        (height.parse(), width.parse(), mines.parse())
                    {
                        if height <= 8 && width <= 8 && height * width > mines {
                            return Some((height, width, mines));
                        }
                    }
                }
                (None, None, None, None) => {
                    return Some((8, 8, 9));
                }
                _ => {}
            }

            None
        }

        if let Some((height, width, mines)) = get_args(args) {
            let mut pool = context.database.minesweeper.lock();

            let minesweeper = Session::new(height, width, mines);

            let send_message = SendMessage::new(chat_id, get_game_info(&minesweeper))
                .reply_markup(get_inline_keyboard(&minesweeper))
                .reply_to_message_id(msg_id);

            pool.sessions.insert([chat_id, msg_id], minesweeper);

            drop(pool);

            context.api.execute(send_message).await?;
        } else {
            let send_message = SendMessage::new(chat_id, "参数错误").reply_to_message_id(msg_id);
            context.api.execute(send_message).await?;
        }

        return Ok(true);
    }

    Ok(false)
}

pub async fn handle_minesweeper_callback_query(
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

            let mut pool = context.database.minesweeper.lock();

            if let Some(minesweeper) = pool.sessions.get_mut(&[chat_id, cmd_msg_id]) {
                if row < minesweeper.game.get_height() && col < minesweeper.game.get_width() {
                    if let Ok(true) = minesweeper.game.click(row, col, true) {
                        if minesweeper.game.get_step_count() == 1 {
                            minesweeper.start_time = Some(Instant::now());
                        }

                        if let Entry::Vacant(entry) = minesweeper.players.entry(user_id) {
                            entry.insert(Player::new(user.get_full_name()));
                        } else {
                            let player =
                                unsafe { minesweeper.players.get_mut(&user_id).unwrap_unchecked() };
                            player.step += 1;
                        }

                        if let Status::Exploded(_) = minesweeper.game.get_game_status() {
                            minesweeper.trigger = Some(user.get_full_name());
                        }

                        let edit_message =
                            EditMessageText::new(chat_id, msg_id, get_game_info(minesweeper))
                                .reply_markup(get_inline_keyboard(minesweeper));

                        let answer_callback_query = AnswerCallbackQuery::new(id);

                        if minesweeper.game.is_ended() {
                            pool.sessions.remove(&[chat_id, cmd_msg_id]);
                        }

                        drop(pool);

                        tokio::try_join!(
                            context.api.execute(edit_message),
                            context.api.execute(answer_callback_query)
                        )?;
                    } else {
                        drop(pool);

                        let answer_callback_query = AnswerCallbackQuery::new(id);
                        context.api.execute(answer_callback_query).await?;
                    }
                }
            } else {
                drop(pool);

                let answer_callback_query = AnswerCallbackQuery::new(id)
                    .text("找不到游戏")
                    .show_alert(true);

                context.api.execute(answer_callback_query).await?;
            }

            return Ok(true);
        }
    }

    Ok(false)
}

fn get_revealed_cell_emoji(cell: &Cell) -> &'static str {
    if cell.is_mine {
        "💣"
    } else {
        match cell.mine_adjacent {
            0 => "➖",
            1 => "1️⃣",
            2 => "2️⃣",
            3 => "3️⃣",
            4 => "4️⃣",
            5 => "5️⃣",
            6 => "6️⃣",
            7 => "7️⃣",
            8 => "8️⃣",
            _ => unreachable!(),
        }
    }
}

fn get_inline_keyboard(minesweeper: &Session) -> InlineKeyboardMarkup {
    let mut vec = (0..minesweeper.game.get_height())
        .map(|row| {
            (0..minesweeper.game.get_width())
                .map(|col| {
                    let cell = minesweeper.game.get(row, col);

                    let text = if cell.is_flagged {
                        "🚩"
                    } else if cell.is_revealed || minesweeper.game.is_ended() {
                        get_revealed_cell_emoji(cell)
                    } else {
                        "➕"
                    };

                    InlineKeyboardButton::new(
                        text,
                        InlineKeyboardButtonKind::CallbackData(format!("minesweeper-{row}-{col}")),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    if let Status::Exploded(exploded) = minesweeper.game.get_game_status() {
        for (row, col) in exploded {
            vec[*row][*col] = InlineKeyboardButton::new(
                "💥",
                InlineKeyboardButtonKind::CallbackData(format!("minesweeper-{row}-{col}")),
            );
        }
    }

    InlineKeyboardMarkup::from_vec(vec)
}

fn parse_callback_data(data: &str) -> Option<(usize, usize)> {
    let mut data = data.split('-');

    if let (Some("minesweeper"), Some(row), Some(col), None) =
        (data.next(), data.next(), data.next(), data.next())
    {
        if let (Ok(row), Ok(col)) = (row.parse(), col.parse()) {
            return Some((row, col));
        }
    }

    None
}

fn get_game_info(minesweeper: &Session) -> String {
    let mut info = String::from("扫雷\n\n");

    for Player { name, step } in minesweeper.players.values() {
        info.push_str(name);
        info.push('：');
        info.push_str(&step.to_string());
        info.push_str(" 次操作\n");
    }

    info.push('\n');

    fn get_time(start: Instant) -> (u64, u64) {
        let now = Instant::now();
        let time = now.duration_since(start).as_secs();
        (time / 60, time % 60)
    }

    match minesweeper.game.get_game_status() {
        Status::Win => {
            let start_time = unsafe { minesweeper.start_time.unwrap_unchecked() };
            let (minute, second) = get_time(start_time);

            info.push_str("用时：");
            info.push_str(&minute.to_string());
            info.push_str(" 分 ");
            info.push_str(&second.to_string());
            info.push_str(" 秒\n");

            info.push_str("扫雷成功");
        }
        Status::Exploded(_) => {
            let start_time = unsafe { minesweeper.start_time.unwrap_unchecked() };
            let (minute, second) = get_time(start_time);

            info.push_str("用时：");
            info.push_str(&minute.to_string());
            info.push_str(" 分 ");
            info.push_str(&second.to_string());
            info.push_str(" 秒\n");

            let trigger = unsafe { minesweeper.trigger.as_ref().unwrap_unchecked() };
            info.push_str(trigger);
            info.push_str(" 引爆了地雷");
        }
        Status::InProgress => {}
    }

    info
}
