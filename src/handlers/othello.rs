use crate::{context::Context, error::ErrorHandler};
use carapax::{
    handler, HandlerResult,
    methods::{AnswerCallbackQuery, EditMessageReplyMarkup, EditMessageText, SendMessage},
    session::SessionId,
    types::{CallbackQuery, Command, InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup, ReplyMarkup, User}
};
use serde::{Deserialize, Serialize};
use std::cmp::{max, min};
use tokio::try_join;

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
enum OthelloPiece {
    Black,
    Empty,
    White
}

impl OthelloPiece {
    fn as_str(&self) -> &str {
        match self {
            OthelloPiece::Black => "⚫",
            OthelloPiece::Empty => "",
            OthelloPiece::White => "⚪"
        }
    }

    fn reverse(&self) -> OthelloPiece {
        match self {
            OthelloPiece::Black => OthelloPiece::White,
            OthelloPiece::Empty => OthelloPiece::Empty,
            OthelloPiece::White => OthelloPiece::Black
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Othello {
    id: i64,
    data: [[OthelloPiece; 8]; 8],
    next: OthelloPiece,
    player_black: Option<User>,
    player_white: Option<User>
}

impl Othello {
    fn new(id: i64) -> Othello {
        Othello {
            id: id,
            data: {
                let mut data = [[OthelloPiece::Empty; 8]; 8];
                data[3][3] = OthelloPiece::Black;
                data[3][4] = OthelloPiece::White;
                data[4][3] = OthelloPiece::White;
                data[4][4] = OthelloPiece::Black;
                data
            },
            next: OthelloPiece::Black,
            player_black: None,
            player_white: None
        }
    }

    fn get(&self, pos: &(usize, usize)) -> OthelloPiece {
        self.data[pos.0][pos.1]
    }

    fn set(&mut self, pos: &(usize, usize), piece: OthelloPiece) -> bool {
        let mut is_changed = false;
        if self.data[pos.0][pos.1] == OthelloPiece::Empty {
            // 左
            if pos.0 > 1 && self.get(&(pos.0 - 1, pos.1)) == piece.reverse() {
                for row in (0..(pos.0 - 1)).rev() {
                    if self.get(&(row, pos.1)) == piece {
                        self.data[pos.0][pos.1] = piece;
                        let mut row_rev = row + 1;
                        loop {
                            if self.get(&(row_rev, pos.1)) == piece.reverse() {
                                self.data[row_rev][pos.1] = piece;
                                row_rev += 1;
                            } else {
                                break;
                            }
                        }
                        is_changed = true;
                        break;
                    }
                }
            }
            if pos.0 < 6 && self.get(&(pos.0 + 1, pos.1)) == piece.reverse() {
                for row in (pos.0 + 1)..8 {
                    if self.get(&(row, pos.1)) == piece {
                        self.data[pos.0][pos.1] = piece;
                        let mut row_rev = row - 1;
                        loop {
                            if self.get(&(row_rev, pos.1)) == piece.reverse() {
                                self.data[row_rev][pos.1] = piece;
                                row_rev -= 1;
                            } else {
                                break;
                            }
                        }
                        is_changed = true;
                        break;
                    }
                }
            }
            if pos.1 > 1 && self.get(&(pos.0, pos.1 - 1)) == piece.reverse() {
                for col in (0..(pos.1 - 1)).rev() {
                    if self.get(&(pos.0, col)) == piece {
                        self.data[pos.0][pos.1] = piece;
                        let mut col_rev = col + 1;
                        loop {
                            if self.get(&(pos.0, col_rev)) == piece.reverse() {
                                self.data[pos.0][col_rev] = piece;
                                col_rev += 1;
                            } else {
                                break;
                            }
                        }
                        is_changed = true;
                        break;
                    }
                }
            }
            if pos.1 < 6 && self.get(&(pos.0, pos.1 + 1)) == piece.reverse() {
                for col in (pos.1 + 1)..8 {
                    if self.get(&(pos.0, col)) == piece {
                        self.data[pos.0][pos.1] = piece;
                        let mut col_rev = col - 1;
                        loop {
                            if self.get(&(pos.0, col_rev)) == piece.reverse() {
                                self.data[pos.0][col_rev] = piece;
                                col_rev -= 1;
                            } else {
                                break;
                            }
                        }
                        is_changed = true;
                        break;
                    }
                }
            }
            if pos.0 > 1 && pos.1 > 1 && self.get(&(pos.0 - 1, pos.1 - 1)) == piece.reverse() {
                for n in 0..(min(pos.0, pos.1) - 1) {
                    if self.get(&(pos.0 - n - 2, pos.1 - n - 2)) == piece {
                        self.data[pos.0][pos.1] = piece;
                        let mut n_rev = n + 1;
                        loop {
                            if self.get(&(pos.0 - n_rev, pos.1 - n_rev)) == piece.reverse() {
                                self.data[pos.0 - n_rev][pos.1 - n_rev] = piece;
                                n_rev -= 1;
                            } else {
                                break;
                            }
                        }
                        is_changed = true;
                        break;
                    }
                }
            }
            if pos.0 > 1 && pos.1 < 6 && self.get(&(pos.0 - 1, pos.1 + 1)) == piece.reverse() {
                for n in 0..(min(pos.0, 7 - pos.1) - 1) {
                    if self.get(&(pos.0 - n - 2, pos.1 + n + 2)) == piece {
                        self.data[pos.0][pos.1] = piece;
                        let mut n_rev = n + 1;
                        loop {
                            if self.get(&(pos.0 - n_rev, pos.1 + n_rev)) == piece.reverse() {
                                self.data[pos.0 - n_rev][pos.1 + n_rev] = piece;
                                n_rev -= 1;
                            } else {
                                break;
                            }
                        }
                        is_changed = true;
                        break;
                    }
                }
            }
            if pos.0 < 6 && pos.1 > 1 && self.get(&(pos.0 + 1, pos.1 - 1)) == piece.reverse() {
                for n in 0..(min(7 - pos.0, pos.1) - 1) {
                    if self.get(&(pos.0 + n + 2, pos.1 - n - 2)) == piece {
                        self.data[pos.0][pos.1] = piece;
                        let mut n_rev = n + 1;
                        loop {
                            if self.get(&(pos.0 + n_rev, pos.1 - n_rev)) == piece.reverse() {
                                self.data[pos.0 + n_rev][pos.1 - n_rev] = piece;
                                n_rev -= 1;
                            } else {
                                break;
                            }
                        }
                        is_changed = true;
                        break;
                    }
                }
            }
            if pos.0 < 6 && pos.1 < 6 && self.get(&(pos.0 + 1, pos.1 + 1)) == piece.reverse() {
                for n in 0..(6 - max(pos.0, pos.1)) {
                    if self.get(&(pos.0 + n + 2, pos.1 + n + 2)) == piece {
                        self.data[pos.0][pos.1] = piece;
                        let mut n_rev = n + 1;
                        loop {
                            if self.get(&(pos.0 + n_rev, pos.1 + n_rev)) == piece.reverse() {
                                self.data[pos.0 + n_rev][pos.1 + n_rev] = piece;
                                n_rev -= 1;
                            } else {
                                break;
                            }
                        }
                        is_changed = true;
                        break;
                    }
                }
            }
        }
        is_changed
    }

    fn next(&mut self) {
        self.next = match self.next {
            OthelloPiece::Black => OthelloPiece::White,
            _ => OthelloPiece::Black
        };
    }

    fn is_ended(&self) -> bool {
        // TODO
        false
    }

    fn get_inline_keyboard(&self) -> InlineKeyboardMarkup {
        let mut keyboad: Vec<Vec<InlineKeyboardButton>> = Vec::new();
        for col in 0..8 {
            let mut keyboad_col: Vec<InlineKeyboardButton> = Vec::new();
            for row in 0..8 {
                keyboad_col.push(
                    InlineKeyboardButton::new(
                        self.get(&(row, col)).as_str(),
                        InlineKeyboardButtonKind::CallbackData(String::from("othello_") + &row.to_string() + "_" + &col.to_string())
                    )
                );
            }
            keyboad.push(keyboad_col);
        }
        InlineKeyboardMarkup::from(keyboad)
    }

    fn print_players(&self) -> String {
        let mut players = String::new();
        if let Some(player_black) = &self.player_black {
            players.push_str("⚫：");
            if let Some(username) = &player_black.username {
                players += &username;
            }
            if let Some(player_white) = &self.player_white {
                players.push_str("\n⚪：");
                if let Some(username) = &player_white.username {
                    players += &username;
                }
            }
        }
        players
    }

    fn print(&self) -> String {
        let mut board = String::from("\n");
        for col in 0..8 {
            for row in 0..8 {
                board.push_str(self.data[row][col].as_str());
            }
            board.push_str("\n");
        }
        board.push_str("\n");
        board
    }
}

trait OthelloVec {
    fn get_index(&mut self, id: i64) -> usize;
}

impl OthelloVec for Vec<Othello> {
    fn get_index(&mut self, id: i64) -> usize {
        match self.iter().position(|v| v.id == id) {
            Some(index) => index,
            None => {
                self.push(Othello::new(id));
                self.len() - 1
            }
        }
    }
}

#[handler(command = "/othello")]
pub async fn othello_command_handler(context: &Context, command: Command) -> Result<HandlerResult, ErrorHandler> {
    let message = command.get_message();
    let chat_id = message.get_chat_id();
    if let Some(_) = message.get_user() {
        let method = SendMessage::new(chat_id, "黑白棋")
            .reply_markup(ReplyMarkup::InlineKeyboardMarkup(Othello::new(0).get_inline_keyboard()));
        context.api.execute(method).await?;
    }
    Ok(HandlerResult::Stop)
}

#[handler]
pub async fn othello_inlinekeyboard_handler(context: &Context, query: CallbackQuery) -> Result<HandlerResult, ErrorHandler> {
    let data = query.data;
    if let Some(data) = data {
        let cell: Option<(usize, usize)> = match data.as_str() {
            "othello_0_0" => Some((0, 0)),
            "othello_0_1" => Some((0, 1)),
            "othello_0_2" => Some((0, 2)),
            "othello_0_3" => Some((0, 3)),
            "othello_0_4" => Some((0, 4)),
            "othello_0_5" => Some((0, 5)),
            "othello_0_6" => Some((0, 6)),
            "othello_0_7" => Some((0, 7)),
            "othello_1_0" => Some((1, 0)),
            "othello_1_1" => Some((1, 1)),
            "othello_1_2" => Some((1, 2)),
            "othello_1_3" => Some((1, 3)),
            "othello_1_4" => Some((1, 4)),
            "othello_1_5" => Some((1, 5)),
            "othello_1_6" => Some((1, 6)),
            "othello_1_7" => Some((1, 7)),
            "othello_2_0" => Some((2, 0)),
            "othello_2_1" => Some((2, 1)),
            "othello_2_2" => Some((2, 2)),
            "othello_2_3" => Some((2, 3)),
            "othello_2_4" => Some((2, 4)),
            "othello_2_5" => Some((2, 5)),
            "othello_2_6" => Some((2, 6)),
            "othello_2_7" => Some((2, 7)),
            "othello_3_0" => Some((3, 0)),
            "othello_3_1" => Some((3, 1)),
            "othello_3_2" => Some((3, 2)),
            "othello_3_3" => Some((3, 3)),
            "othello_3_4" => Some((3, 4)),
            "othello_3_5" => Some((3, 5)),
            "othello_3_6" => Some((3, 6)),
            "othello_3_7" => Some((3, 7)),
            "othello_4_0" => Some((4, 0)),
            "othello_4_1" => Some((4, 1)),
            "othello_4_2" => Some((4, 2)),
            "othello_4_3" => Some((4, 3)),
            "othello_4_4" => Some((4, 4)),
            "othello_4_5" => Some((4, 5)),
            "othello_4_6" => Some((4, 6)),
            "othello_4_7" => Some((4, 7)),
            "othello_5_0" => Some((5, 0)),
            "othello_5_1" => Some((5, 1)),
            "othello_5_2" => Some((5, 2)),
            "othello_5_3" => Some((5, 3)),
            "othello_5_4" => Some((5, 4)),
            "othello_5_5" => Some((5, 5)),
            "othello_5_6" => Some((5, 6)),
            "othello_5_7" => Some((5, 7)),
            "othello_6_0" => Some((6, 0)),
            "othello_6_1" => Some((6, 1)),
            "othello_6_2" => Some((6, 2)),
            "othello_6_3" => Some((6, 3)),
            "othello_6_4" => Some((6, 4)),
            "othello_6_5" => Some((6, 5)),
            "othello_6_6" => Some((6, 6)),
            "othello_6_7" => Some((6, 7)),
            "othello_7_0" => Some((7, 0)),
            "othello_7_1" => Some((7, 1)),
            "othello_7_2" => Some((7, 2)),
            "othello_7_3" => Some((7, 3)),
            "othello_7_4" => Some((7, 4)),
            "othello_7_5" => Some((7, 5)),
            "othello_7_6" => Some((7, 6)),
            "othello_7_7" => Some((7, 7)),
            _ => None
        };
        if let Some(cell) = cell {
            let message = query.message.unwrap();
            let chat_id = message.get_chat_id();
            let message_id = message.id;
            if let Some(message_author) = message.get_user() {
                let user = query.from;
                let mut session = context.session_manager.get_session(SessionId::new(chat_id, message_author.id))?;
                let mut othello = session.get("othello").await?.unwrap_or(Vec::new());
                let index = othello.get_index(message_id);
                let mut edit_message: Option<EditMessageText> = None;
                let mut answer_callback_query: Option<&str> = None;
                match othello[index].next {
                    OthelloPiece::Black => {
                        match &othello[index].player_black {
                            Some(player_black) => {
                                if &user == player_black {
                                    if othello[index].set(&cell, OthelloPiece::Black) {
                                        othello[index].next();
                                    } else {
                                        answer_callback_query = Some("无法在此落子");
                                    }
                                } else {
                                    answer_callback_query = Some("不是您的回合");
                                }
                            },
                            None => {
                                if othello[index].set(&cell, OthelloPiece::Black) {
                                    othello[index].player_black = Some(user.clone());
                                    othello[index].next();
                                    edit_message = Some(EditMessageText::new(
                                        chat_id, message_id,
                                        String::from("黑白棋\n") + &othello[index].print_players()
                                    ));
                                } else {
                                    answer_callback_query = Some("无法在此落子");
                                }
                            }
                        }
                    },
                    OthelloPiece::White => {
                        match &othello[index].player_white {
                            Some(player_white) => {
                                if &user == player_white {
                                    if othello[index].set(&cell, OthelloPiece::White) {
                                        othello[index].next();
                                    } else {
                                        answer_callback_query = Some("无法在此落子");
                                    }
                                } else {
                                    answer_callback_query = Some("不是您的回合");
                                }
                            },
                            None => {
                                if othello[index].set(&cell, OthelloPiece::White) {
                                    othello[index].player_white = Some(user.clone());
                                    othello[index].next();
                                    edit_message = Some(EditMessageText::new(
                                        chat_id, message_id,
                                        String::from("黑白棋\n") + &othello[index].print_players()
                                    ));
                                } else {
                                    answer_callback_query = Some("无法在此落子");
                                }
                            }
                        }
                    },
                    _ => ()
                }
                match answer_callback_query {
                    Some(message) => {
                        let method = AnswerCallbackQuery::new(query.id)
                            .text(message)
                            .show_alert(true);
                        context.api.execute(method).await?;
                    },
                    None => {
                        let method = AnswerCallbackQuery::new(query.id);
                        context.api.execute(method).await?;
                    }
                }
                session.set("othello", &othello).await?;
                let edit_reply_markup = EditMessageReplyMarkup::new(chat_id, message_id)
                    .reply_markup(othello[index].get_inline_keyboard());
                match edit_message {
                    Some(edit_message) => {
                        try_join!(context.api.execute(edit_message), context.api.execute(edit_reply_markup))?;
                    },
                    None => {
                        context.api.execute(edit_reply_markup).await?;
                    }
                }
            }
        }
    }
    Ok(HandlerResult::Continue)
}