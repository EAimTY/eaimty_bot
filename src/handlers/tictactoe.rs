use crate::{context::Context, error::ErrorHandler};
use carapax::{
    handler, HandlerResult,
    methods::{AnswerCallbackQuery, EditMessageReplyMarkup, EditMessageText, SendMessage},
    session::SessionId,
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

enum TicTacToeBoardRange {
    Part0,
    Part1,
    Part2
}

impl TicTacToeBoardRange {
    fn as_usize(&self) -> usize {
        match self {
            TicTacToeBoardRange::Part0 => 0,
            TicTacToeBoardRange::Part1 => 1,
            TicTacToeBoardRange::Part2 => 2
        }
    }
}

enum TicTacToeGameState {
    OnGoing,
    Tie,
    Win
}

#[derive(Serialize, Deserialize)]
struct TicTacToe {
    id: i64,
    data: [[TicTacToePiece; 3]; 3],
    next: TicTacToePiece,
    player_cross: Option<User>,
    player_nought: Option<User>
}

impl TicTacToe {
    fn new(id: i64) -> TicTacToe {
        TicTacToe {
            id: id,
            data: [[TicTacToePiece::Empty; 3]; 3],
            next: TicTacToePiece::Cross,
            player_cross: None,
            player_nought: None
        }
    }

    fn get(&self, pos: &(TicTacToeBoardRange, TicTacToeBoardRange)) -> TicTacToePiece {
        self.data[pos.0.as_usize()][pos.1.as_usize()]
    }

    fn set(&mut self, pos: &(TicTacToeBoardRange, TicTacToeBoardRange), piece: TicTacToePiece) {
        self.data[pos.0.as_usize()][pos.1.as_usize()] = piece;
    }

    fn is_empty(&self, pos: &(TicTacToeBoardRange, TicTacToeBoardRange)) -> bool {
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
                    self.get(&(TicTacToeBoardRange::Part0, TicTacToeBoardRange::Part0)).as_str(),
                    InlineKeyboardButtonKind::CallbackData(String::from("tictactoe_left_top"))
                ),
                InlineKeyboardButton::new(
                    self.get(&(TicTacToeBoardRange::Part1, TicTacToeBoardRange::Part0)).as_str(),
                    InlineKeyboardButtonKind::CallbackData(String::from("tictactoe_middle_top"))
                ),
                InlineKeyboardButton::new(
                    self.get(&(TicTacToeBoardRange::Part2, TicTacToeBoardRange::Part0)).as_str(),
                    InlineKeyboardButtonKind::CallbackData(String::from("tictactoe_right_top"))
                )
            ],
            vec![
                InlineKeyboardButton::new(
                    self.get(&(TicTacToeBoardRange::Part0, TicTacToeBoardRange::Part1)).as_str(),
                    InlineKeyboardButtonKind::CallbackData(String::from("tictactoe_left_middle"))
                ),
                InlineKeyboardButton::new(
                    self.get(&(TicTacToeBoardRange::Part1, TicTacToeBoardRange::Part1)).as_str(),
                    InlineKeyboardButtonKind::CallbackData(String::from("tictactoe_middle_middle"))
                ),
                InlineKeyboardButton::new(
                    self.get(&(TicTacToeBoardRange::Part2, TicTacToeBoardRange::Part1)).as_str(),
                    InlineKeyboardButtonKind::CallbackData(String::from("tictactoe_right_middle"))
                )
            ],
            vec![
                InlineKeyboardButton::new(
                    self.get(&(TicTacToeBoardRange::Part0, TicTacToeBoardRange::Part2)).as_str(),
                    InlineKeyboardButtonKind::CallbackData(String::from("tictactoe_left_bottom"))
                ),
                InlineKeyboardButton::new(
                    self.get(&(TicTacToeBoardRange::Part1, TicTacToeBoardRange::Part2)).as_str(),
                    InlineKeyboardButtonKind::CallbackData(String::from("tictactoe_middle_bottom"))
                ),
                InlineKeyboardButton::new(
                    self.get(&(TicTacToeBoardRange::Part2, TicTacToeBoardRange::Part2)).as_str(),
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

trait TicTacToeVec {
    fn get_index(&mut self, id: i64) -> usize;
}

impl TicTacToeVec for Vec<TicTacToe> {
    fn get_index(&mut self, id: i64) -> usize {
        match self.iter().position(|v| v.id == id) {
            Some(index) => index,
            None => {
                self.push(TicTacToe::new(id));
                self.len() - 1
            }
        }
    }
}

#[handler(command = "/tictactoe")]
pub async fn tictactoe_command_handler(context: &Context, command: Command) -> Result<HandlerResult, ErrorHandler> {
    let message = command.get_message();
    let chat_id = message.get_chat_id();
    if let Some(_) = message.get_user() {
        let method = SendMessage::new(chat_id, "Tic-Tac-Toe")
            .reply_markup(ReplyMarkup::InlineKeyboardMarkup(TicTacToe::new(0).get_inline_keyboard()));
        context.api.execute(method).await?;
    }
    Ok(HandlerResult::Stop)
}

#[handler]
pub async fn tictactoe_inlinekeyboard_handler(context: &Context, query: CallbackQuery) -> Result<HandlerResult, ErrorHandler> {
    let data = query.data;
    if let Some(data) = data {
        let cell: Option<(TicTacToeBoardRange, TicTacToeBoardRange)> = match data.as_str() {
            "tictactoe_left_top" => Some((TicTacToeBoardRange::Part0, TicTacToeBoardRange::Part0)),
            "tictactoe_left_middle" => Some((TicTacToeBoardRange::Part0, TicTacToeBoardRange::Part1)),
            "tictactoe_left_bottom" => Some((TicTacToeBoardRange::Part0, TicTacToeBoardRange::Part2)),
            "tictactoe_middle_top" => Some((TicTacToeBoardRange::Part1, TicTacToeBoardRange::Part0)),
            "tictactoe_middle_middle" => Some((TicTacToeBoardRange::Part1, TicTacToeBoardRange::Part1)),
            "tictactoe_middle_bottom" => Some((TicTacToeBoardRange::Part1, TicTacToeBoardRange::Part2)),
            "tictactoe_right_top" => Some((TicTacToeBoardRange::Part2, TicTacToeBoardRange::Part0)),
            "tictactoe_right_middle" => Some((TicTacToeBoardRange::Part2, TicTacToeBoardRange::Part1)),
            "tictactoe_right_bottom" => Some((TicTacToeBoardRange::Part2, TicTacToeBoardRange::Part2)),
            _ => None
        };
        if let Some(cell) = cell {
            let message = query.message.unwrap();
            let chat_id = message.get_chat_id();
            let message_id = message.id;
            if let Some(message_author) = message.get_user() {
                let user = query.from;
                let mut session = context.session_manager.get_session(SessionId::new(chat_id, message_author.id))?;
                let mut tictactoe_vec = session.get("tictactoe_vec").await?.unwrap_or(Vec::new());
                let index = tictactoe_vec.get_index(message_id);
                let mut edit_message: Option<EditMessageText> = None;
                let mut answer_callback_query: Option<&str> = None;
                match tictactoe_vec[index].next {
                    TicTacToePiece::Cross => {
                        match &tictactoe_vec[index].player_cross {
                            Some(player_cross) => {
                                if &user == player_cross {
                                    if tictactoe_vec[index].is_empty(&cell) {
                                        tictactoe_vec[index].set(&cell, TicTacToePiece::Cross);
                                        tictactoe_vec[index].next();
                                    } else {
                                        answer_callback_query = Some("请在空白处落子");
                                    }
                                } else {
                                    answer_callback_query = Some("不是您的回合");
                                }
                            },
                            None => {
                                if tictactoe_vec[index].is_empty(&cell) {
                                    tictactoe_vec[index].player_cross = Some(user.clone());
                                    tictactoe_vec[index].set(&cell, TicTacToePiece::Cross);
                                    tictactoe_vec[index].next();
                                    edit_message = Some(EditMessageText::new(
                                        chat_id, message_id,
                                        String::from("Tic-Tac-Toe\n") + &tictactoe_vec[index].print_players()
                                    ));
                                } else {
                                    answer_callback_query = Some("请在空白处落子");
                                }
                            }
                        }
                    },
                    TicTacToePiece::Nought => {
                        match &tictactoe_vec[index].player_nought {
                            Some(player_nought) => {
                                if &user == player_nought {
                                    if tictactoe_vec[index].is_empty(&cell) {
                                        tictactoe_vec[index].set(&cell, TicTacToePiece::Nought);
                                        tictactoe_vec[index].next();
                                    } else {
                                        answer_callback_query = Some("请在空白处落子");
                                    }
                                } else {
                                    answer_callback_query = Some("不是您的回合");
                                }
                            },
                            None => {
                                if tictactoe_vec[index].is_empty(&cell) {
                                    tictactoe_vec[index].player_nought = Some(user.clone());
                                    tictactoe_vec[index].set(&cell, TicTacToePiece::Nought);
                                    tictactoe_vec[index].next();
                                    edit_message = Some(EditMessageText::new(
                                        chat_id, message_id,
                                        String::from("Tic-Tac-Toe\n") + &tictactoe_vec[index].print_players()
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
                match tictactoe_vec[index].is_ended() {
                    TicTacToeGameState::OnGoing => {
                        session.set("tictactoe_vec", &tictactoe_vec).await?;
                        let edit_reply_markup = EditMessageReplyMarkup::new(chat_id, message_id)
                            .reply_markup(tictactoe_vec[index].get_inline_keyboard());
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
                        let method = EditMessageText::new(
                            chat_id, message_id,
                            String::from("Tic-Tac-Toe\n") +
                            &tictactoe_vec[index].print_players() +
                            &String::from("\n") +
                            &tictactoe_vec[index].print() +
                            &String::from("平局")
                        );
                        context.api.execute(method).await?;
                        tictactoe_vec.remove(index);
                        if tictactoe_vec.is_empty() {
                            session.remove("tictactoe_vec").await?;
                        } else {
                            session.set("tictactoe_vec", &tictactoe_vec).await?;
                        }
                    },
                    TicTacToeGameState::Win => {
                        let method = EditMessageText::new(
                            chat_id, message_id,
                            String::from("Tic-Tac-Toe\n") +
                            &tictactoe_vec[index].print_players() +
                            &String::from("\n") +
                            &tictactoe_vec[index].print() +
                            &user.username.unwrap_or(String::from("")) +
                            &String::from(" 赢了")
                        );
                        context.api.execute(method).await?;
                        tictactoe_vec.remove(index);
                        if tictactoe_vec.is_empty() {
                            session.remove("tictactoe_vec").await?;
                        } else {
                            session.set("tictactoe_vec", &tictactoe_vec).await?;
                        }
                    }
                }
            }
        }
    }
    Ok(HandlerResult::Continue)
}