use crate::{context::Context, error::ErrorHandler};
use carapax::{
    handler,
    methods::{AnswerCallbackQuery, EditMessageReplyMarkup, EditMessageText, SendMessage},
    session::SessionId,
    types::{
        CallbackQuery, Command, InlineKeyboardButton, InlineKeyboardButtonKind,
        InlineKeyboardMarkup, ReplyMarkup, User,
    },
    HandlerResult,
};
use serde::{Deserialize, Serialize};
use tokio::try_join;

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
enum TicTacToePiece {
    Cross,
    Nought,
    Empty,
}

impl TicTacToePiece {
    fn as_str(&self) -> &str {
        match self {
            TicTacToePiece::Cross => "❌",
            TicTacToePiece::Nought => "⭕️",
            TicTacToePiece::Empty => "⬜",
        }
    }
}

enum TicTacToeGameState {
    OnGoing,
    Tie,
    Win,
}

#[derive(Serialize, Deserialize)]
struct TicTacToe {
    id: i64,
    data: [[TicTacToePiece; 3]; 3],
    next: TicTacToePiece,
    player_cross: Option<User>,
    player_nought: Option<User>,
}

impl TicTacToe {
    fn new(id: i64) -> TicTacToe {
        TicTacToe {
            id: id,
            data: [[TicTacToePiece::Empty; 3]; 3],
            next: TicTacToePiece::Cross,
            player_cross: None,
            player_nought: None,
        }
    }

    fn get(&self, pos: &(usize, usize)) -> TicTacToePiece {
        self.data[pos.0][pos.1]
    }

    fn set(&mut self, pos: &(usize, usize), piece: TicTacToePiece) -> bool {
        if self.get(&(pos.0, pos.1)) == TicTacToePiece::Empty {
            self.data[pos.0][pos.1] = piece;
            return true;
        }
        false
    }

    fn next(&mut self) {
        self.next = match self.next {
            TicTacToePiece::Cross => TicTacToePiece::Nought,
            _ => TicTacToePiece::Cross,
        };
    }

    fn is_ended(&self) -> TicTacToeGameState {
        for row in 0..3 {
            if self.get(&(row, 0)) != TicTacToePiece::Empty
                && self.get(&(row, 0)) == self.get(&(row, 1))
                && self.get(&(row, 0)) == self.get(&(row, 2))
            {
                return TicTacToeGameState::Win;
            }
        }
        for col in 0..3 {
            if self.get(&(0, col)) != TicTacToePiece::Empty
                && self.get(&(0, col)) == self.get(&(1, col))
                && self.get(&(0, col)) == self.get(&(2, col))
            {
                return TicTacToeGameState::Win;
            }
        }
        if (self.get(&(0, 0)) != TicTacToePiece::Empty
            && self.get(&(0, 0)) == self.get(&(1, 1))
            && self.get(&(0, 0)) == self.get(&(2, 2)))
            || (self.get(&(0, 2)) != TicTacToePiece::Empty
                && self.get(&(0, 2)) == self.get(&(1, 1))
                && self.get(&(0, 2)) == self.get(&(2, 0)))
        {
            return TicTacToeGameState::Win;
        }
        let mut is_full = true;
        for row in 0..3 {
            for col in 0..3 {
                if self.get(&(row, col)) == TicTacToePiece::Empty {
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
        let mut keyboad: Vec<Vec<InlineKeyboardButton>> = Vec::new();
        for col in 0..3 {
            let mut keyboad_col: Vec<InlineKeyboardButton> = Vec::new();
            for row in 0..3 {
                keyboad_col.push(InlineKeyboardButton::new(
                    self.get(&(row, col)).as_str(),
                    InlineKeyboardButtonKind::CallbackData(
                        String::from("tictactoe_") + &row.to_string() + "_" + &col.to_string(),
                    ),
                ));
            }
            keyboad.push(keyboad_col);
        }
        InlineKeyboardMarkup::from(keyboad)
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
pub async fn tictactoe_command_handler(
    context: &Context,
    command: Command,
) -> Result<HandlerResult, ErrorHandler> {
    let message = command.get_message();
    let chat_id = message.get_chat_id();
    if let Some(_) = message.get_user() {
        let method = SendMessage::new(chat_id, "Tic-Tac-Toe").reply_markup(
            ReplyMarkup::InlineKeyboardMarkup(TicTacToe::new(0).get_inline_keyboard()),
        );
        context.api.execute(method).await?;
    }
    Ok(HandlerResult::Stop)
}

#[handler]
pub async fn tictactoe_inlinekeyboard_handler(
    context: &Context,
    query: CallbackQuery,
) -> Result<HandlerResult, ErrorHandler> {
    let data = query.data;
    if let Some(data) = data {
        let cell: Option<(usize, usize)> = match data.as_str() {
            "tictactoe_0_0" => Some((0, 0)),
            "tictactoe_0_1" => Some((0, 1)),
            "tictactoe_0_2" => Some((0, 2)),
            "tictactoe_1_0" => Some((1, 0)),
            "tictactoe_1_1" => Some((1, 1)),
            "tictactoe_1_2" => Some((1, 2)),
            "tictactoe_2_0" => Some((2, 0)),
            "tictactoe_2_1" => Some((2, 1)),
            "tictactoe_2_2" => Some((2, 2)),
            _ => None,
        };
        if let Some(cell) = cell {
            let message = query.message.unwrap();
            let chat_id = message.get_chat_id();
            let message_id = message.id;
            if let Some(message_author) = message.get_user() {
                let user = query.from;
                let mut session = context
                    .session_manager
                    .get_session(SessionId::new(chat_id, message_author.id))?;
                let mut tictactoe = session.get("tictactoe").await?.unwrap_or(Vec::new());
                let index = tictactoe.get_index(message_id);
                let mut edit_message: Option<EditMessageText> = None;
                let mut answer_callback_query: Option<&str> = None;
                match tictactoe[index].next {
                    TicTacToePiece::Cross => match &tictactoe[index].player_cross {
                        Some(player_cross) => {
                            if &user == player_cross {
                                if tictactoe[index].set(&cell, TicTacToePiece::Cross) {
                                    tictactoe[index].next();
                                } else {
                                    answer_callback_query = Some("请在空白处落子");
                                }
                            } else {
                                answer_callback_query = Some("不是您的回合");
                            }
                        }
                        None => {
                            if tictactoe[index].set(&cell, TicTacToePiece::Cross) {
                                tictactoe[index].player_cross = Some(user.clone());
                                tictactoe[index].next();
                                edit_message = Some(EditMessageText::new(
                                    chat_id,
                                    message_id,
                                    String::from("Tic-Tac-Toe\n")
                                        + &tictactoe[index].print_players(),
                                ));
                            } else {
                                answer_callback_query = Some("请在空白处落子");
                            }
                        }
                    },
                    TicTacToePiece::Nought => match &tictactoe[index].player_nought {
                        Some(player_nought) => {
                            if &user == player_nought {
                                if tictactoe[index].set(&cell, TicTacToePiece::Nought) {
                                    tictactoe[index].next();
                                } else {
                                    answer_callback_query = Some("请在空白处落子");
                                }
                            } else {
                                answer_callback_query = Some("不是您的回合");
                            }
                        }
                        None => {
                            if tictactoe[index].set(&cell, TicTacToePiece::Nought) {
                                tictactoe[index].player_nought = Some(user.clone());
                                tictactoe[index].next();
                                edit_message = Some(EditMessageText::new(
                                    chat_id,
                                    message_id,
                                    String::from("Tic-Tac-Toe\n")
                                        + &tictactoe[index].print_players(),
                                ));
                            } else {
                                answer_callback_query = Some("请在空白处落子");
                            }
                        }
                    },
                    _ => (),
                }
                match answer_callback_query {
                    Some(message) => {
                        let method = AnswerCallbackQuery::new(query.id)
                            .text(message)
                            .show_alert(true);
                        context.api.execute(method).await?;
                    }
                    None => {
                        let method = AnswerCallbackQuery::new(query.id);
                        context.api.execute(method).await?;
                    }
                }
                match tictactoe[index].is_ended() {
                    TicTacToeGameState::OnGoing => {
                        session.set("tictactoe", &tictactoe).await?;
                        let edit_reply_markup = EditMessageReplyMarkup::new(chat_id, message_id)
                            .reply_markup(tictactoe[index].get_inline_keyboard());
                        match edit_message {
                            Some(edit_message) => {
                                try_join!(
                                    context.api.execute(edit_message),
                                    context.api.execute(edit_reply_markup)
                                )?;
                            }
                            None => {
                                context.api.execute(edit_reply_markup).await?;
                            }
                        }
                    }
                    TicTacToeGameState::Tie => {
                        let method = EditMessageText::new(
                            chat_id,
                            message_id,
                            String::from("Tic-Tac-Toe\n")
                                + &tictactoe[index].print_players()
                                + &String::from("\n")
                                + &tictactoe[index].print()
                                + &String::from("平局"),
                        );
                        context.api.execute(method).await?;
                        tictactoe.remove(index);
                        if tictactoe.is_empty() {
                            session.remove("tictactoe").await?;
                        } else {
                            session.set("tictactoe", &tictactoe).await?;
                        }
                    }
                    TicTacToeGameState::Win => {
                        let method = EditMessageText::new(
                            chat_id,
                            message_id,
                            String::from("Tic-Tac-Toe\n")
                                + &tictactoe[index].print_players()
                                + &String::from("\n")
                                + &tictactoe[index].print()
                                + &user.username.unwrap_or(String::from(""))
                                + &String::from(" 赢了"),
                        );
                        context.api.execute(method).await?;
                        tictactoe.remove(index);
                        if tictactoe.is_empty() {
                            session.remove("tictactoe").await?;
                        } else {
                            session.set("tictactoe", &tictactoe).await?;
                        }
                    }
                }
            }
        }
    }
    Ok(HandlerResult::Continue)
}
