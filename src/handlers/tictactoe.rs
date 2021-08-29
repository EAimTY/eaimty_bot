use crate::{context::Context, error::ErrorHandler};
use carapax::{
    handler,
    methods::{AnswerCallbackQuery, EditMessageText, SendMessage},
    session::SessionId,
    types::{
        CallbackQuery, Command, InlineKeyboardButton, InlineKeyboardButtonKind,
        InlineKeyboardMarkup, ReplyMarkup, User,
    },
    HandlerResult,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, fmt};

// 棋子类型
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
enum Piece {
    Cross,
    Nought,
    Empty,
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Piece::Cross => write!(f, "❌"),
            Piece::Nought => write!(f, "⭕️"),
            Piece::Empty => write!(f, "⬜"),
        }
    }
}

// 棋子位置
#[derive(Clone, Copy)]
struct PiecePosition {
    row: usize,
    col: usize,
}

impl PiecePosition {
    fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

// 棋局状态
enum GameState {
    OnGoing,
    Tie,
    Win,
}

// 落子失败类型
#[derive(Debug)]
enum ActionError {
    CellNotEmpty,
    NotYourTurn,
}

impl fmt::Display for ActionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActionError::CellNotEmpty => write!(f, "请在空白处落子"),
            ActionError::NotYourTurn => write!(f, "不是你的回合"),
        }
    }
}

impl Error for ActionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

// 棋局
#[derive(Clone, Serialize, Deserialize)]
struct Game {
    data: [[Piece; 3]; 3],
    turn: Piece,
    player_cross: Option<User>,
    player_nought: Option<User>,
}

impl Game {
    fn new() -> Self {
        Self {
            data: [[Piece::Empty; 3]; 3],
            turn: Piece::Cross,
            player_cross: None,
            player_nought: None,
        }
    }

    // 获取指定位置的棋子类型
    fn get(&self, pos: PiecePosition) -> Piece {
        self.data[pos.row][pos.col]
    }

    // 设定指定位子的棋子，失败时返回 Err(ActionError::CellNotEmpty)
    fn set(&mut self, pos: PiecePosition, piece: Piece) -> Result<(), ActionError> {
        if self.get(pos) == Piece::Empty {
            self.data[pos.row][pos.col] = piece;
            return Ok(());
        }
        Err(ActionError::CellNotEmpty)
    }

    // 尝试落子，成功时返回 Ok(棋局状态)，失败时返回 Err(ActionError)
    fn try_put(&mut self, pos: PiecePosition, player: User) -> Result<GameState, ActionError> {
        // 轮到 Cross 落子
        if let Piece::Cross = self.turn {
            // 有玩家作为 Cross
            if let Some(player_cross) = &self.player_cross {
                // 验证落子发起者
                if &player == player_cross {
                    match self.set(pos, Piece::Cross) {
                        Ok(_) => self.next_turn(),
                        Err(err) => return Err(err),
                    }
                } else {
                    return Err(ActionError::NotYourTurn);
                }
            // 没有玩家作为 Cross
            } else {
                match self.set(pos, Piece::Cross) {
                    Ok(_) => {
                        self.player_cross = Some(player);
                        self.next_turn();
                    }
                    Err(err) => return Err(err),
                }
            }
        // 轮到 Nought 落子
        } else {
            if let Some(player_nought) = &self.player_nought {
                if &player == player_nought {
                    match self.set(pos, Piece::Nought) {
                        Ok(_) => self.next_turn(),
                        Err(err) => return Err(err),
                    }
                } else {
                    return Err(ActionError::NotYourTurn);
                }
            } else {
                match self.set(pos, Piece::Nought) {
                    Ok(_) => {
                        self.player_nought = Some(player);
                        self.next_turn();
                    }
                    Err(err) => return Err(err),
                }
            }
        }
        // 返回棋局状态
        Ok(self.get_game_state())
    }

    fn next_turn(&mut self) {
        self.turn = match self.turn {
            Piece::Cross => Piece::Nought,
            _ => Piece::Cross,
        };
    }

    // 计算棋局状态
    fn get_game_state(&self) -> GameState {
        // 纵向检查
        for row in 0..3 {
            if self.get(PiecePosition::new(row, 0)) != Piece::Empty
                && self.get(PiecePosition::new(row, 0)) == self.get(PiecePosition::new(row, 1))
                && self.get(PiecePosition::new(row, 0)) == self.get(PiecePosition::new(row, 2))
            {
                return GameState::Win;
            }
        }
        // 横向检查
        for col in 0..3 {
            if self.get(PiecePosition::new(0, col)) != Piece::Empty
                && self.get(PiecePosition::new(0, col)) == self.get(PiecePosition::new(1, col))
                && self.get(PiecePosition::new(0, col)) == self.get(PiecePosition::new(2, col))
            {
                return GameState::Win;
            }
        }
        // 对角线检查
        if (self.get(PiecePosition::new(0, 0)) != Piece::Empty
            && self.get(PiecePosition::new(0, 0)) == self.get(PiecePosition::new(1, 1))
            && self.get(PiecePosition::new(0, 0)) == self.get(PiecePosition::new(2, 2)))
            || (self.get(PiecePosition::new(0, 2)) != Piece::Empty
                && self.get(PiecePosition::new(0, 2)) == self.get(PiecePosition::new(1, 1))
                && self.get(PiecePosition::new(0, 2)) == self.get(PiecePosition::new(2, 0)))
        {
            return GameState::Win;
        }
        // 检查棋盘是否已满
        let mut is_all_filled = true;
        for row in 0..3 {
            for col in 0..3 {
                if self.get(PiecePosition::new(row, col)) == Piece::Empty {
                    is_all_filled = false;
                }
            }
        }
        if is_all_filled {
            return GameState::Tie;
        }
        GameState::OnGoing
    }

    // 获取按钮列表
    fn get_inline_keyboard(&self) -> InlineKeyboardMarkup {
        let mut keyboad: Vec<Vec<InlineKeyboardButton>> = Vec::new();
        for col in 0..3 {
            let mut keyboad_col: Vec<InlineKeyboardButton> = Vec::new();
            for row in 0..3 {
                keyboad_col.push(InlineKeyboardButton::new(
                    self.get(PiecePosition::new(row, col)).to_string(),
                    InlineKeyboardButtonKind::CallbackData(format!("tictactoe_{}_{}", row, col)),
                ));
            }
            keyboad.push(keyboad_col);
        }
        InlineKeyboardMarkup::from(keyboad)
    }

    // 获取玩家
    fn get_players(&self) -> String {
        let mut players = String::new();
        if let Some(player_cross) = &self.player_cross {
            players.push_str("❌：");
            players += &player_cross.first_name;
            if let Some(player_nought) = &self.player_nought {
                players.push_str("\n⭕️：");
                players += &player_nought.first_name;
            }
        }
        players
    }

    // 获取玩家
    fn get_next_player(&self) -> String {
        self.turn.to_string()
    }

    // 获取棋盘
    fn get_game_board(&self) -> String {
        let mut board = String::new();
        for col in 0..3 {
            for row in 0..3 {
                board.push_str(&self.data[row][col].to_string());
            }
            board.push_str("\n");
        }
        board
    }
}

// session 中正在进行的棋局列表
#[derive(Serialize, Deserialize)]
struct GameList {
    list: HashMap<i64, Game>,
}

impl GameList {
    fn new() -> Self {
        Self {
            list: HashMap::new(),
        }
    }

    fn get(&mut self, id: i64) -> Game {
        self.list.entry(id).or_insert(Game::new()).clone()
    }

    fn update_and_check_empty(&mut self, id: i64, game: Option<Game>) -> bool {
        if let Some(game) = game {
            self.list.insert(id, game);
            false
        } else {
            self.list.remove(&id);
            self.list.is_empty()
        }
    }
}

// 尝试解析 callback data，返回目标落子位置
fn try_parse_callback(data: String) -> Option<PiecePosition> {
    if data.starts_with("tictactoe_") {
        let mut data = data[10..].split('_');
        if let Some(row) = data.next() {
            if let Ok(row) = row.parse::<usize>() {
                if let Some(col) = data.next() {
                    if let Ok(col) = col.parse::<usize>() {
                        if row < 3 && col < 3 {
                            if let None = data.next() {
                                return Some(PiecePosition::new(row, col));
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

#[handler(command = "/tictactoe")]
pub async fn tictactoe_command_handler(
    context: &Context,
    command: Command,
) -> Result<HandlerResult, ErrorHandler> {
    let message = command.get_message();
    let chat_id = message.get_chat_id();
    let method = SendMessage::new(chat_id, "Tic-Tac-Toe").reply_markup(
        ReplyMarkup::InlineKeyboardMarkup(Game::new().get_inline_keyboard()),
    );
    context.api.execute(method).await?;
    Ok(HandlerResult::Stop)
}

#[handler]
pub async fn tictactoe_inlinekeyboard_handler(
    context: &Context,
    query: CallbackQuery,
) -> Result<HandlerResult, ErrorHandler> {
    // 检查非空 query
    if let Some(data) = query.data {
        // 尝试 parse callback data
        if let Some(pos) = try_parse_callback(data) {
            let message = query.message.unwrap();
            let chat_id = message.get_chat_id();
            let message_id = message.id;
            let user = query.from;
            // 从 session 获取棋局
            let mut session = context
                .session_manager
                .get_session(SessionId::new(chat_id, 0))?;
            let mut game_list = session.get("tictactoe").await?.unwrap_or(GameList::new());
            let mut game = game_list.get(message_id);
            // 尝试落子
            match game.try_put(pos, user.clone()) {
                // 落子成功
                Ok(game_state) => {
                    let method: EditMessageText;
                    // 匹配棋局状态
                    match game_state {
                        // 棋局正在进行
                        GameState::OnGoing => {
                            method = EditMessageText::new(
                                chat_id,
                                message_id,
                                format!(
                                    "Tic-Tac-Toe\n\n{}\n\n轮到：{}",
                                    game.get_players(),
                                    game.get_next_player()
                                ),
                            )
                            .reply_markup(game.get_inline_keyboard());
                            game_list.update_and_check_empty(message_id, Some(game.clone()));
                            session.set("tictactoe", &game_list).await?;
                        }
                        // 平局
                        GameState::Tie => {
                            method = EditMessageText::new(
                                chat_id,
                                message_id,
                                format!(
                                    "Tic-Tac-Toe\n\n{}\n\n{}平局",
                                    game.get_players(),
                                    game.get_game_board()
                                ),
                            );
                            if game_list.update_and_check_empty(message_id, None) {
                                session.remove("tictactoe").await?;
                            } else {
                                session.set("tictactoe", &game_list).await?;
                            }
                        }
                        // 玩家获胜
                        GameState::Win => {
                            method = EditMessageText::new(
                                chat_id,
                                message_id,
                                format!(
                                    "Tic-Tac-Toe\n\n{}\n\n{}\n\n{} 赢了",
                                    game.get_players(),
                                    game.get_game_board(),
                                    user.first_name
                                ),
                            );
                            if game_list.update_and_check_empty(message_id, None) {
                                session.remove("tictactoe").await?;
                            } else {
                                session.set("tictactoe", &game_list).await?;
                            }
                        }
                    }
                    context.api.execute(method).await?;
                    // 回应 callback
                    let method = AnswerCallbackQuery::new(query.id);
                    context.api.execute(method).await?;
                }
                // 落子失败
                Err(err) => {
                    // 以错误提示回应 callback
                    let method = AnswerCallbackQuery::new(query.id)
                        .text(err.to_string())
                        .show_alert(true);
                    context.api.execute(method).await?;
                }
            }
            return Ok(HandlerResult::Stop);
        }
    }
    Ok(HandlerResult::Continue)
}
