use crate::{context::Context, error::ErrorHandler};
use carapax::{
    handler, HandlerResult,
    methods::{AnswerCallbackQuery, EditMessageReplyMarkup, EditMessageText, SendMessage},
    types::{CallbackQuery, Command, InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup, MessageData, ReplyMarkup}
};
use serde::{Deserialize, Serialize};
use tokio::try_join;

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
enum TicTacToePiece {
    Cross,
    Empty,
    Nought
}

impl TicTacToePiece {
    fn as_str(&self) -> &str {
        match self {
            TicTacToePiece::Cross => "❌",
            TicTacToePiece::Empty => "⬜",
            TicTacToePiece::Nought => "⭕️"
        }
    }
}

enum TicTacToeCellRange {
    Part0,
    Part1,
    Part2
}

impl TicTacToeCellRange {
    fn as_usize(&self) -> usize {
        match self {
            TicTacToeCellRange::Part0 => 0,
            TicTacToeCellRange::Part1 => 1,
            TicTacToeCellRange::Part2 => 2
        }
    }
}

#[derive (Serialize, Deserialize)]
struct TicTacToe {
    data: [[TicTacToePiece; 3]; 3],
    next: TicTacToePiece,
    player_cross: Option<i64>,
    player_nought: Option<i64>
}

impl TicTacToe {
    fn new() -> TicTacToe {
        TicTacToe {
            data: [[TicTacToePiece::Empty; 3]; 3],
            next: TicTacToePiece::Cross,
            player_cross: None,
            player_nought: None
        }
    }

    fn get(&self, pos: &(TicTacToeCellRange, TicTacToeCellRange)) -> TicTacToePiece {
        self.data[pos.0.as_usize()][pos.1.as_usize()]
    }

    fn set(&mut self, pos: &(TicTacToeCellRange, TicTacToeCellRange), piece: TicTacToePiece) {
        self.data[pos.0.as_usize()][pos.1.as_usize()] = piece;
    }

    fn is_empty(&self, pos: &(TicTacToeCellRange, TicTacToeCellRange)) -> bool {
        if self.data[pos.0.as_usize()][pos.1.as_usize()] == TicTacToePiece::Empty {
            return true;
        }
        false
    }

    fn next(&mut self){
        self.next = match self.next {
            TicTacToePiece::Cross => TicTacToePiece::Nought,
            _ => TicTacToePiece::Cross
        }
    }

    fn is_game_over(&self) -> bool {
        for row in 0..3 {
            if self.data[row][0] != TicTacToePiece::Empty && self.data[row][0] == self.data[row][1] && self.data[row][0] == self.data[row][2] {
                return true;
            }
        }
        for col in 0..3 {
            if self.data[0][col] != TicTacToePiece::Empty && self.data[0][col] == self.data[1][col] && self.data[0][col] == self.data[2][col] {
                return true;
            }
        }
        if
            (self.data[0][0] != TicTacToePiece::Empty && self.data[0][0] == self.data[1][1] && self.data[0][0] == self.data[2][2]) ||
            (self.data[0][2] != TicTacToePiece::Empty && self.data[0][2] == self.data[1][1] && self.data[0][2] == self.data[2][0])
        {
            return true;
        }
        false
    }

    fn get_inline_keyboard(&self) -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::from(vec![
            vec![
                InlineKeyboardButton::new(
                    self.get(&(TicTacToeCellRange::Part0, TicTacToeCellRange::Part0)).as_str(),
                    InlineKeyboardButtonKind::CallbackData(String::from("tictactoe_left_top"))
                ),
                InlineKeyboardButton::new(
                    self.get(&(TicTacToeCellRange::Part1, TicTacToeCellRange::Part0)).as_str(),
                    InlineKeyboardButtonKind::CallbackData(String::from("tictactoe_middle_top"))
                ),
                InlineKeyboardButton::new(
                    self.get(&(TicTacToeCellRange::Part2, TicTacToeCellRange::Part0)).as_str(),
                    InlineKeyboardButtonKind::CallbackData(String::from("tictactoe_right_top"))
                )
            ],
            vec![
                InlineKeyboardButton::new(
                    self.get(&(TicTacToeCellRange::Part0, TicTacToeCellRange::Part1)).as_str(),
                    InlineKeyboardButtonKind::CallbackData(String::from("tictactoe_left_middle"))
                ),
                InlineKeyboardButton::new(
                    self.get(&(TicTacToeCellRange::Part1, TicTacToeCellRange::Part1)).as_str(),
                    InlineKeyboardButtonKind::CallbackData(String::from("tictactoe_middle_middle"))
                ),
                InlineKeyboardButton::new(
                    self.get(&(TicTacToeCellRange::Part2, TicTacToeCellRange::Part1)).as_str(),
                    InlineKeyboardButtonKind::CallbackData(String::from("tictactoe_right_middle"))
                )
            ],
            vec![
                InlineKeyboardButton::new(
                    self.get(&(TicTacToeCellRange::Part0, TicTacToeCellRange::Part2)).as_str(),
                    InlineKeyboardButtonKind::CallbackData(String::from("tictactoe_left_bottom"))
                ),
                InlineKeyboardButton::new(
                    self.get(&(TicTacToeCellRange::Part1, TicTacToeCellRange::Part2)).as_str(),
                    InlineKeyboardButtonKind::CallbackData(String::from("tictactoe_middle_bottom"))
                ),
                InlineKeyboardButton::new(
                    self.get(&(TicTacToeCellRange::Part2, TicTacToeCellRange::Part2)).as_str(),
                    InlineKeyboardButtonKind::CallbackData(String::from("tictactoe_right_bottom"))
                )
            ]
        ])
    }

    fn print(&self) -> String {
        let mut print = String::new();
        for col in 0..3 {
            for row in 0..3 {
                print.push_str(self.data[row][col].as_str());
            }
            print.push_str("\n");
        }
        print
    }
}

async fn tictactoe_answer_query(context: &Context, query_id: String, message: &str, show_alert: bool) -> Result<(), ErrorHandler> {
    let method = AnswerCallbackQuery::new(query_id)
        .text(message)
        .show_alert(show_alert);
    context.api.execute(method).await?;
    Ok(())
}

#[handler(command = "/tictactoe")]
pub async fn tictactoe_command_handler(context: &Context, command: Command) -> Result<HandlerResult, ErrorHandler> {
    let message = command.get_message();
    let chat_id = message.get_chat_id();
    if let Some(_) = message.get_user() {
        let method = SendMessage::new(chat_id, "Tic-Tac-Toe")
            .reply_markup(ReplyMarkup::InlineKeyboardMarkup(TicTacToe::new().get_inline_keyboard()));
        context.api.execute(method).await?;
    }
    Ok(HandlerResult::Stop)
}

#[handler]
pub async fn tictactoe_inlinekeyboard_handler(context: &Context, query: CallbackQuery) -> Result<HandlerResult, ErrorHandler> {
    let data = query.data;
    if let Some(data) = data {
        let cell: Option<(TicTacToeCellRange, TicTacToeCellRange)> = match data.as_str() {
            "tictactoe_left_top" => Some((TicTacToeCellRange::Part0, TicTacToeCellRange::Part0)),
            "tictactoe_left_middle" => Some((TicTacToeCellRange::Part0, TicTacToeCellRange::Part1)),
            "tictactoe_left_bottom" => Some((TicTacToeCellRange::Part0, TicTacToeCellRange::Part2)),
            "tictactoe_middle_top" => Some((TicTacToeCellRange::Part1, TicTacToeCellRange::Part0)),
            "tictactoe_middle_middle" => Some((TicTacToeCellRange::Part1, TicTacToeCellRange::Part1)),
            "tictactoe_middle_bottom" => Some((TicTacToeCellRange::Part1, TicTacToeCellRange::Part2)),
            "tictactoe_right_top" => Some((TicTacToeCellRange::Part2, TicTacToeCellRange::Part0)),
            "tictactoe_right_middle" => Some((TicTacToeCellRange::Part2, TicTacToeCellRange::Part1)),
            "tictactoe_right_bottom" => Some((TicTacToeCellRange::Part2, TicTacToeCellRange::Part2)),
            _ => None
        };
        if let Some(cell) = cell {
            let message = query.message;
            if let Some(message) = message {
                let mut session = context.session_manager.get_session(&message)?;
                let mut tictactoe = session.get("tictactoe").await?.unwrap_or(TicTacToe::new());
                let chat_id = message.get_chat_id();
                let user_id = query.from.id;
                let user_name = query.from.username.unwrap_or(String::from(""));
                let message_id = message.id;
                let original_message = if let MessageData::Text(text) = message.data { text.data } else { String::from("") };
                let mut edit_message: Option<EditMessageText> = None;
                match tictactoe.next {
                    TicTacToePiece::Cross => {
                        match tictactoe.player_cross {
                            Some(player_cross) => {
                                if user_id == player_cross {
                                    if tictactoe.is_empty(&cell) {
                                        tictactoe.set(&cell, TicTacToePiece::Cross);
                                        tictactoe.next();
                                    } else {
                                        tictactoe_answer_query(context, query.id, "请在空白处落子", true).await?;
                                    }
                                } else {
                                    tictactoe_answer_query(context, query.id, "不是您的回合", true).await?;
                                }
                            },
                            None => {
                                if tictactoe.is_empty(&cell) {
                                    edit_message = Some(EditMessageText::new(
                                        chat_id,
                                        message_id,
                                        original_message.clone() + "\n❌：" + &user_name)
                                    );
                                    tictactoe.player_cross = Some(user_id);
                                    tictactoe.set(&cell, TicTacToePiece::Cross);
                                    tictactoe.next();
                                } else {
                                    tictactoe_answer_query(context, query.id, "请在空白处落子", true).await?;
                                }
                            }
                        }
                    },
                    TicTacToePiece::Nought => {
                        match tictactoe.player_nought {
                            Some(player_nought) => {
                                if user_id == player_nought {
                                    if tictactoe.is_empty(&cell) {
                                        tictactoe.set(&cell, TicTacToePiece::Nought);
                                        tictactoe.next();
                                    } else {
                                        tictactoe_answer_query(context, query.id, "请在空白处落子", true).await?;
                                    }
                                } else {
                                    tictactoe_answer_query(context, query.id, "不是您的回合", true).await?;
                                }
                            },
                            None => {
                                if tictactoe.is_empty(&cell) {
                                    edit_message = Some(EditMessageText::new(
                                        chat_id,
                                        message_id,
                                        original_message.clone() + "\n⭕️：" + &user_name)
                                    );
                                    tictactoe.player_nought = Some(user_id);
                                    tictactoe.set(&cell, TicTacToePiece::Nought);
                                    tictactoe.next();
                                } else {
                                    tictactoe_answer_query(context, query.id, "请在空白处落子", true).await?;
                                }
                            }
                        }
                    },
                    _ => ()
                }
                if tictactoe.is_game_over() {
                    session.remove("tictactoe").await?;
                    let method = EditMessageText::new(
                        chat_id,
                        message_id,
                        original_message.clone() + "\n\n" + &tictactoe.print() + "\n" + &user_name + " 赢了"
                    );
                    context.api.execute(method).await?;
                } else {
                    session.set("tictactoe", &tictactoe).await?;
                    let edit_reply_markup = EditMessageReplyMarkup::new(chat_id, message_id)
                        .reply_markup(tictactoe.get_inline_keyboard());
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
    }
    Ok(HandlerResult::Continue)
}