use crate::{context::Context, error::ErrorHandler};
use carapax::{
    handler, HandlerResult,
    methods::{AnswerCallbackQuery, EditMessageReplyMarkup, EditMessageText, SendMessage},
    types::{CallbackQuery, Command, InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup, ReplyMarkup, User}
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

enum TicTacToeGameState {
    OnGoing,
    Tie,
    Win
}

#[derive (Serialize, Deserialize)]
struct TicTacToe {
    data: [[TicTacToePiece; 3]; 3],
    next: TicTacToePiece,
    player_cross: Option<User>,
    player_nought: Option<User>
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

    fn is_ended(&self) -> TicTacToeGameState {
        for row in 0..3 {
            if
                self.data[row][0] != TicTacToePiece::Empty &&
                self.data[row][0] == self.data[row][1] &&
                self.data[row][0] == self.data[row][2]
            {
                return TicTacToeGameState::Win;
            }
        }
        for col in 0..3 {
            if
                self.data[0][col] != TicTacToePiece::Empty &&
                self.data[0][col] == self.data[1][col] &&
                self.data[0][col] == self.data[2][col]
            {
                return TicTacToeGameState::Win;
            }
        }
        if
            (self.data[0][0] != TicTacToePiece::Empty && self.data[0][0] == self.data[1][1] && self.data[0][0] == self.data[2][2]) ||
            (self.data[0][2] != TicTacToePiece::Empty && self.data[0][2] == self.data[1][1] && self.data[0][2] == self.data[2][0])
        {
            return TicTacToeGameState::Win;
        }
        let mut is_full = true;
        for row in 0..3 {
            for col in 0..3 {
                if self.data[row][col] == TicTacToePiece::Empty {
                    is_full = false;
                }
            }
        }
        if is_full {
            return TicTacToeGameState::Tie;
        }
        TicTacToeGameState::OnGoing
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

    fn print_players(&self) -> String {
        let mut players = String::new();
        if let Some(player_cross) = &self.player_cross {
            players.push_str("❌：");
            if let Some(username) = &player_cross.username {
                players += &username;
            }
            if let Some(player_nought) = &self.player_nought {
                players.push_str("\n⭕️：");
                if let Some(username) = &player_nought.username {
                    players += &username;
                }
            }
        }
        players
    }

    fn print(&self) -> String {
        let mut board = String::from("\n");
        for col in 0..3 {
            for row in 0..3 {
                board.push_str(self.data[row][col].as_str());
            }
            board.push_str("\n");
        }
        board.push_str("\n");
        board
    }
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
            let message = query.message.unwrap();
            let mut session = context.session_manager.get_session(&message)?;
            let mut tictactoe = session.get("tictactoe").await?.unwrap_or(TicTacToe::new());
            let chat_id = message.get_chat_id();
            let user = query.from;
            let message_id = message.id;
            let mut edit_message: Option<EditMessageText> = None;
            let mut answer_callback_query: Option<&str> = None;
            match tictactoe.next {
                TicTacToePiece::Cross => {
                    match &tictactoe.player_cross {
                        Some(player_cross) => {
                            if &user == player_cross {
                                if tictactoe.is_empty(&cell) {
                                    tictactoe.set(&cell, TicTacToePiece::Cross);
                                    tictactoe.next();
                                } else {
                                    answer_callback_query = Some("请在空白处落子");
                                }
                            } else {
                                answer_callback_query = Some("不是您的回合");
                            }
                        },
                        None => {
                            if tictactoe.is_empty(&cell) {
                                tictactoe.player_cross = Some(user.clone());
                                tictactoe.set(&cell, TicTacToePiece::Cross);
                                tictactoe.next();
                                edit_message = Some(EditMessageText::new(
                                    chat_id, message_id,
                                    String::from("Tic-Tac-Toe\n") + &tictactoe.print_players()
                                ));
                            } else {
                                answer_callback_query = Some("请在空白处落子");
                            }
                        }
                    }
                },
                TicTacToePiece::Nought => {
                    match &tictactoe.player_nought {
                        Some(player_nought) => {
                            if &user == player_nought {
                                if tictactoe.is_empty(&cell) {
                                    tictactoe.set(&cell, TicTacToePiece::Nought);
                                    tictactoe.next();
                                } else {
                                    answer_callback_query = Some("请在空白处落子");
                                }
                            } else {
                                answer_callback_query = Some("不是您的回合");
                            }
                        },
                        None => {
                            if tictactoe.is_empty(&cell) {
                                tictactoe.player_nought = Some(user.clone());
                                tictactoe.set(&cell, TicTacToePiece::Nought);
                                tictactoe.next();
                                edit_message = Some(EditMessageText::new(
                                    chat_id, message_id,
                                    String::from("Tic-Tac-Toe\n") + &tictactoe.print_players()
                                ));
                            } else {
                                answer_callback_query = Some("请在空白处落子");
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
            match tictactoe.is_ended() {
                TicTacToeGameState::OnGoing => {
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
                },
                TicTacToeGameState::Tie => {
                    session.remove("tictactoe").await?;
                    let method = EditMessageText::new(
                        chat_id, message_id,
                        String::from("Tic-Tac-Toe\n") +
                        &tictactoe.print_players() +
                        &String::from("\n") +
                        &tictactoe.print() +
                        &String::from("平局")
                    );
                    context.api.execute(method).await?;
                },
                TicTacToeGameState::Win => {
                    session.remove("tictactoe").await?;
                    let method = EditMessageText::new(
                        chat_id, message_id,
                        String::from("Tic-Tac-Toe\n") +
                        &tictactoe.print_players() +
                        &String::from("\n") +
                        &tictactoe.print() +
                        &user.username.unwrap_or(String::from("")) +
                        &String::from("赢了")
                    );
                    context.api.execute(method).await?;
                }
            }
        }
    }
    Ok(HandlerResult::Continue)
}