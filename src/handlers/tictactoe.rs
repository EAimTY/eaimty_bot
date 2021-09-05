use crate::{context::Context, error::ErrorHandler};
use carapax::{
    handler,
    methods::{AnswerCallbackQuery, EditMessageText, SendMessage},
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
    fn from(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    // 尝试解析 callback data，返回目标落子位置
    fn try_parse_callback(data: String) -> Option<Self> {
        if data.starts_with("tictactoe_") {
            let mut data = data[10..].split('_');
            if let Some(row) = data.next() {
                if let Ok(row) = row.parse::<usize>() {
                    if let Some(col) = data.next() {
                        if let Ok(col) = col.parse::<usize>() {
                            if row < 3 && col < 3 {
                                if let None = data.next() {
                                    return Some(Self::from(row, col));
                                }
                            }
                        }
                    }
                }
            }
        }
        None
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

// 存储玩家信息
#[derive(Clone, Serialize, Deserialize)]
struct Player {
    id: i64,
    name: String,
}

// 自动转换 carapax::types::User 到 Player
impl From<&User> for Player {
    fn from(user: &User) -> Self {
        Self {
            id: user.id,
            name: user.get_full_name(),
        }
    }
}

// 棋局
#[derive(Clone, Serialize, Deserialize)]
struct Game {
    data: [[Piece; 3]; 3],
    turn: Piece,
    player_cross: Option<Player>,
    player_nought: Option<Player>,
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

    // 设定指定位子的棋子，失败时返回 Err(ActionError)
    fn set(&mut self, pos: PiecePosition, piece: Piece) -> Result<(), ActionError> {
        if self.get(pos) == Piece::Empty {
            self.data[pos.row][pos.col] = piece;
            return Ok(());
        }
        Err(ActionError::CellNotEmpty)
    }

    // 尝试落子，成功时返回 Ok(棋局状态)，失败时返回 Err(ActionError)
    fn try_put(&mut self, pos: PiecePosition, user: &User) -> Result<GameState, ActionError> {
        // 轮到 Cross 落子
        if let Piece::Cross = self.turn {
            // 有玩家作为 Cross
            if let Some(player_cross) = &self.player_cross {
                // 验证落子发起者
                if user.id == player_cross.id {
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
                        self.player_cross = Some(user.into());
                        self.next_turn();
                    }
                    Err(err) => return Err(err),
                }
            }
        // 轮到 Nought 落子
        } else {
            if let Some(player_nought) = &self.player_nought {
                if user.id == player_nought.id {
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
                        self.player_nought = Some(user.into());
                        self.next_turn();
                    }
                    Err(err) => return Err(err),
                }
            }
        }
        // 返回棋局状态
        Ok(self.get_game_state())
    }

    // 设定下一位轮到的玩家
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
            if self.get(PiecePosition::from(row, 0)) != Piece::Empty
                && self.get(PiecePosition::from(row, 0)) == self.get(PiecePosition::from(row, 1))
                && self.get(PiecePosition::from(row, 0)) == self.get(PiecePosition::from(row, 2))
            {
                return GameState::Win;
            }
        }
        // 横向检查
        for col in 0..3 {
            if self.get(PiecePosition::from(0, col)) != Piece::Empty
                && self.get(PiecePosition::from(0, col)) == self.get(PiecePosition::from(1, col))
                && self.get(PiecePosition::from(0, col)) == self.get(PiecePosition::from(2, col))
            {
                return GameState::Win;
            }
        }
        // 对角线检查
        if (self.get(PiecePosition::from(0, 0)) != Piece::Empty
            && self.get(PiecePosition::from(0, 0)) == self.get(PiecePosition::from(1, 1))
            && self.get(PiecePosition::from(0, 0)) == self.get(PiecePosition::from(2, 2)))
            || (self.get(PiecePosition::from(0, 2)) != Piece::Empty
                && self.get(PiecePosition::from(0, 2)) == self.get(PiecePosition::from(1, 1))
                && self.get(PiecePosition::from(0, 2)) == self.get(PiecePosition::from(2, 0)))
        {
            return GameState::Win;
        }
        // 检查棋盘是否已满
        let mut is_all_filled = true;
        for row in 0..3 {
            for col in 0..3 {
                if self.get(PiecePosition::from(row, col)) == Piece::Empty {
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
                    self.get(PiecePosition::from(row, col)).to_string(),
                    InlineKeyboardButtonKind::CallbackData(format!("tictactoe_{}_{}", row, col)),
                ));
            }
            keyboad.push(keyboad_col);
        }
        InlineKeyboardMarkup::from(keyboad)
    }

    // 获取双方玩家
    fn get_players(&self) -> String {
        let mut players = String::new();
        if let Some(player_cross) = &self.player_cross {
            players.push_str("❌：");
            players += &player_cross.name;
            if let Some(player_nought) = &self.player_nought {
                players.push_str("\n⭕️：");
                players += &player_nought.name;
            }
        }
        players
    }

    // 获取下一位轮到的玩家
    fn get_next_player(&self) -> String {
        match self.turn {
            Piece::Cross => {
                if let Some(player) = &self.player_cross {
                    player.name.clone()
                } else {
                    Piece::Cross.to_string()
                }
            }
            _ => {
                if let Some(player) = &self.player_nought {
                    player.name.clone()
                } else {
                    Piece::Nought.to_string()
                }
            }
        }
    }
}

// 正在进行的棋局列表
#[derive(Serialize, Deserialize)]
struct GameList {
    list: HashMap<i64, Game>,
}

#[handler(command = "/tictactoe")]
pub async fn tictactoe_command_handler(
    context: &Context,
    command: Command,
) -> Result<HandlerResult, ErrorHandler> {
    let message = command.get_message();
    let chat_id = message.get_chat_id();
    // 创建新游戏
    let game = Game::new();
    // 向 session 存储游戏
    let mut session = context.session_manager.get_session(message)?;
    session
        .set(format!("tictactoe_{}", message.id), &game)
        .await?;
    // 发送游戏地图
    let method = SendMessage::new(chat_id, "Tic-Tac-Toe")
        .reply_markup(ReplyMarkup::InlineKeyboardMarkup(
            game.get_inline_keyboard(),
        ))
        .reply_to_message_id(message.id);
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
        if let Some(pos) = PiecePosition::try_parse_callback(data) {
            let message = query.message.unwrap();
            // 用于回应 Callback Query 的信息
            let mut answer_callback_query = None;
            // 尝试获取触发游戏的原命令消息
            if let Some(command_message) = &message.reply_to {
                // 尝试从 session 获取游戏
                let mut session = context
                    .session_manager
                    .get_session(command_message.as_ref())?;
                let game: Option<Game> = session
                    .get(format!("tictactoe_{}", command_message.id))
                    .await?;
                if let Some(mut game) = game {
                    let chat_id = message.get_chat_id();
                    let user = query.from;
                    // 尝试操作棋局
                    match game.try_put(pos, &user) {
                        // 操作成功
                        Ok(game_state) => {
                            let edit_message_text;
                            // 匹配棋局状态
                            match game_state {
                                // 棋局正在进行
                                GameState::OnGoing => {
                                    edit_message_text = EditMessageText::new(
                                        chat_id,
                                        message.id,
                                        format!(
                                            "Tic-Tac-Toe\n\n{}\n\n轮到：{}",
                                            game.get_players(),
                                            game.get_next_player()
                                        ),
                                    )
                                    .reply_markup(game.get_inline_keyboard());
                                    // 存储棋局
                                    session
                                        .set(format!("tictactoe_{}", command_message.id), &game)
                                        .await?;
                                }
                                // 平局
                                GameState::Tie => {
                                    edit_message_text = EditMessageText::new(
                                        chat_id,
                                        message.id,
                                        format!("Tic-Tac-Toe\n\n{}\n平局", game.get_players()),
                                    )
                                    .reply_markup(game.get_inline_keyboard());
                                    // 删除棋局
                                    session
                                        .remove(format!("tictactoe_{}", command_message.id))
                                        .await?;
                                }
                                // 玩家获胜
                                GameState::Win => {
                                    edit_message_text = EditMessageText::new(
                                        chat_id,
                                        message.id,
                                        format!(
                                            "Tic-Tac-Toe\n\n{}\n{} 赢了",
                                            game.get_players(),
                                            user.get_full_name()
                                        ),
                                    )
                                    .reply_markup(game.get_inline_keyboard());
                                    // 删除棋局
                                    session
                                        .remove(format!("tictactoe_{}", command_message.id))
                                        .await?;
                                }
                            }
                            context.api.execute(edit_message_text).await?;
                            answer_callback_query = Some(AnswerCallbackQuery::new(&query.id));
                        }
                        // 操作失败
                        Err(err) => {
                            answer_callback_query = Some(
                                AnswerCallbackQuery::new(&query.id)
                                    .text(err.to_string())
                                    .show_alert(true),
                            );
                        }
                    }
                }
            }
            // 回应 callback
            context
                .api
                .execute(
                    answer_callback_query.unwrap_or(
                        AnswerCallbackQuery::new(&query.id)
                            .text("找不到游戏")
                            .show_alert(true),
                    ),
                )
                .await?;
            return Ok(HandlerResult::Stop);
        }
    }
    Ok(HandlerResult::Continue)
}
