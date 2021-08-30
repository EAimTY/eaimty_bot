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
use std::{cmp, collections::HashMap, error::Error, fmt};

// 棋子类型
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
enum Piece {
    Black,
    White,
    Empty,
}

impl Piece {
    fn reverse(&self) -> Self {
        match self {
            Piece::White => Piece::Black,
            Piece::Black => Piece::White,
            Piece::Empty => Piece::Empty,
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Piece::Black => write!(f, "⚫"),
            Piece::White => write!(f, "⚪"),
            Piece::Empty => write!(f, "➖"),
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
}

// 落子失败类型
#[derive(Debug)]
enum ActionError {
    Unplaceable,
    NotYourTurn,
}

impl fmt::Display for ActionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActionError::Unplaceable => write!(f, "无法在此落子"),
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
    data: [[Piece; 8]; 8],
    turn: Piece,
    player_black: Option<User>,
    player_white: Option<User>,
}

impl Game {
    fn new() -> Self {
        Self {
            data: {
                let mut data = [[Piece::Empty; 8]; 8];
                data[3][3] = Piece::Black;
                data[3][4] = Piece::White;
                data[4][3] = Piece::White;
                data[4][4] = Piece::Black;
                data
            },
            turn: Piece::Black,
            player_black: None,
            player_white: None,
        }
    }

    // 获取指定位置的棋子类型
    fn get(&self, pos: PiecePosition) -> Piece {
        self.data[pos.row][pos.col]
    }

    // 设定指定位子的棋子，失败时返回 Err(ActionError::Unplaceable)
    fn set(&mut self, pos: PiecePosition, piece: Piece) -> Result<(), ActionError> {
        let mut is_changed = false;
        if self.data[pos.row][pos.col] == Piece::Empty {
            // 向上查找
            if pos.row > 1 && self.get(PiecePosition::from(pos.row - 1, pos.col)) == piece.reverse()
            {
                for n in (0..(pos.row - 1)).rev() {
                    if self.get(PiecePosition::from(n, pos.col)) == Piece::Empty {
                        break;
                    }
                    if self.get(PiecePosition::from(n, pos.col)) == piece {
                        self.data[pos.row][pos.col] = piece;
                        let mut n_rev = n + 1;
                        loop {
                            if self.get(PiecePosition::from(n_rev, pos.col)) == piece.reverse() {
                                self.data[n_rev][pos.col] = piece;
                                n_rev += 1;
                            } else {
                                break;
                            }
                        }
                        is_changed = true;
                        break;
                    }
                }
            }
            // 向下查找
            if pos.row < 6 && self.get(PiecePosition::from(pos.row + 1, pos.col)) == piece.reverse()
            {
                for n in (pos.row + 1)..8 {
                    if self.get(PiecePosition::from(n, pos.col)) == Piece::Empty {
                        break;
                    }
                    if self.get(PiecePosition::from(n, pos.col)) == piece {
                        self.data[pos.row][pos.col] = piece;
                        let mut n_rev = n - 1;
                        loop {
                            if self.get(PiecePosition::from(n_rev, pos.col)) == piece.reverse() {
                                self.data[n_rev][pos.col] = piece;
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
            // 向左查找
            if pos.col > 1 && self.get(PiecePosition::from(pos.row, pos.col - 1)) == piece.reverse()
            {
                for n in (0..(pos.col - 1)).rev() {
                    if self.get(PiecePosition::from(pos.row, n)) == Piece::Empty {
                        break;
                    }
                    if self.get(PiecePosition::from(pos.row, n)) == piece {
                        self.data[pos.row][pos.col] = piece;
                        let mut n_rev = n + 1;
                        loop {
                            if self.get(PiecePosition::from(pos.row, n_rev)) == piece.reverse() {
                                self.data[pos.row][n_rev] = piece;
                                n_rev += 1;
                            } else {
                                break;
                            }
                        }
                        is_changed = true;
                        break;
                    }
                }
            }
            // 向右查找
            if pos.col < 6 && self.get(PiecePosition::from(pos.row, pos.col + 1)) == piece.reverse()
            {
                for n in (pos.col + 1)..8 {
                    if self.get(PiecePosition::from(pos.row, n)) == Piece::Empty {
                        break;
                    }
                    if self.get(PiecePosition::from(pos.row, n)) == piece {
                        self.data[pos.row][pos.col] = piece;
                        let mut n_rev = n - 1;
                        loop {
                            if self.get(PiecePosition::from(pos.row, n_rev)) == piece.reverse() {
                                self.data[pos.row][n_rev] = piece;
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
            // 向左上查找
            if pos.row > 1
                && pos.col > 1
                && self.get(PiecePosition::from(pos.row - 1, pos.col - 1)) == piece.reverse()
            {
                for n in 0..(cmp::min(pos.row, pos.col) - 1) {
                    if self.get(PiecePosition::from(pos.row - n - 2, pos.col - n - 2))
                        == Piece::Empty
                    {
                        break;
                    }
                    if self.get(PiecePosition::from(pos.row - n - 2, pos.col - n - 2)) == piece {
                        self.data[pos.row][pos.col] = piece;
                        let mut n_rev = n + 1;
                        loop {
                            if self.get(PiecePosition::from(pos.row - n_rev, pos.col - n_rev))
                                == piece.reverse()
                            {
                                self.data[pos.row - n_rev][pos.col - n_rev] = piece;
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
            // 向左下查找
            if pos.row > 1
                && pos.col < 6
                && self.get(PiecePosition::from(pos.row - 1, pos.col + 1)) == piece.reverse()
            {
                for n in 0..(cmp::min(pos.row, 7 - pos.col) - 1) {
                    if self.get(PiecePosition::from(pos.row - n - 2, pos.col + n + 2))
                        == Piece::Empty
                    {
                        break;
                    }
                    if self.get(PiecePosition::from(pos.row - n - 2, pos.col + n + 2)) == piece {
                        self.data[pos.row][pos.col] = piece;
                        let mut n_rev = n + 1;
                        loop {
                            if self.get(PiecePosition::from(pos.row - n_rev, pos.col + n_rev))
                                == piece.reverse()
                            {
                                self.data[pos.row - n_rev][pos.col + n_rev] = piece;
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
            // 向右上查找
            if pos.row < 6
                && pos.col > 1
                && self.get(PiecePosition::from(pos.row + 1, pos.col - 1)) == piece.reverse()
            {
                for n in 0..(cmp::min(7 - pos.row, pos.col) - 1) {
                    if self.get(PiecePosition::from(pos.row + n + 2, pos.col - n - 2))
                        == Piece::Empty
                    {
                        break;
                    }
                    if self.get(PiecePosition::from(pos.row + n + 2, pos.col - n - 2)) == piece {
                        self.data[pos.row][pos.col] = piece;
                        let mut n_rev = n + 1;
                        loop {
                            if self.get(PiecePosition::from(pos.row + n_rev, pos.col - n_rev))
                                == piece.reverse()
                            {
                                self.data[pos.row + n_rev][pos.col - n_rev] = piece;
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
            // 向右下查找
            if pos.row < 6
                && pos.col < 6
                && self.get(PiecePosition::from(pos.row + 1, pos.col + 1)) == piece.reverse()
            {
                for n in 0..(6 - cmp::max(pos.row, pos.col)) {
                    if self.get(PiecePosition::from(pos.row + n + 2, pos.col + n + 2))
                        == Piece::Empty
                    {
                        break;
                    }
                    if self.get(PiecePosition::from(pos.row + n + 2, pos.col + n + 2)) == piece {
                        self.data[pos.row][pos.col] = piece;
                        let mut n_rev = n + 1;
                        loop {
                            if self.get(PiecePosition::from(pos.row + n_rev, pos.col + n_rev))
                                == piece.reverse()
                            {
                                self.data[pos.row + n_rev][pos.col + n_rev] = piece;
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
        if is_changed {
            Ok(())
        } else {
            Err(ActionError::Unplaceable)
        }
    }

    // 检查某一方是否可以落子
    fn is_able_to_put(&self, piece: Piece) -> bool {
        for row in 0..8 {
            for col in 0..8 {
                if self.get(PiecePosition::from(row, col)) == Piece::Empty {
                    // 向上查找
                    if row > 1 && self.get(PiecePosition::from(row - 1, col)) == piece.reverse() {
                        for n in (0..(row - 1)).rev() {
                            if self.get(PiecePosition::from(n, col)) == Piece::Empty {
                                break;
                            }
                            if self.get(PiecePosition::from(n, col)) == piece {
                                return true;
                            }
                        }
                    }
                    // 向下查找
                    if row < 6 && self.get(PiecePosition::from(row + 1, col)) == piece.reverse() {
                        for n in (row + 1)..8 {
                            if self.get(PiecePosition::from(n, col)) == Piece::Empty {
                                break;
                            }
                            if self.get(PiecePosition::from(n, col)) == piece {
                                return true;
                            }
                        }
                    }
                    // 向左查找
                    if col > 1 && self.get(PiecePosition::from(row, col - 1)) == piece.reverse() {
                        for n in (0..(col - 1)).rev() {
                            if self.get(PiecePosition::from(row, n)) == Piece::Empty {
                                break;
                            }
                            if self.get(PiecePosition::from(row, n)) == piece {
                                return true;
                            }
                        }
                    }
                    // 向右查找
                    if col < 6 && self.get(PiecePosition::from(row, col + 1)) == piece.reverse() {
                        for n in (col + 1)..8 {
                            if self.get(PiecePosition::from(row, n)) == Piece::Empty {
                                break;
                            }
                            if self.get(PiecePosition::from(row, n)) == piece {
                                return true;
                            }
                        }
                    }
                    // 向左上查找
                    if row > 1
                        && col > 1
                        && self.get(PiecePosition::from(row - 1, col - 1)) == piece.reverse()
                    {
                        for n in 0..(cmp::min(row, col) - 1) {
                            if self.get(PiecePosition::from(row - n - 2, col - n - 2))
                                == Piece::Empty
                            {
                                break;
                            }
                            if self.get(PiecePosition::from(row - n - 2, col - n - 2)) == piece {
                                return true;
                            }
                        }
                    }
                    // 向左下查找
                    if row > 1
                        && col < 6
                        && self.get(PiecePosition::from(row - 1, col + 1)) == piece.reverse()
                    {
                        for n in 0..(cmp::min(row, 7 - col) - 1) {
                            if self.get(PiecePosition::from(row - n - 2, col + n + 2))
                                == Piece::Empty
                            {
                                break;
                            }
                            if self.get(PiecePosition::from(row - n - 2, col + n + 2)) == piece {
                                return true;
                            }
                        }
                    }
                    // 向右上查找
                    if row < 6
                        && col > 1
                        && self.get(PiecePosition::from(row + 1, col - 1)) == piece.reverse()
                    {
                        for n in 0..(cmp::min(7 - row, col) - 1) {
                            if self.get(PiecePosition::from(row + n + 2, col - n - 2))
                                == Piece::Empty
                            {
                                break;
                            }
                            if self.get(PiecePosition::from(row + n + 2, col - n - 2)) == piece {
                                return true;
                            }
                        }
                    }
                    // 向右下查找
                    if row < 6
                        && col < 6
                        && self.get(PiecePosition::from(row + 1, col + 1)) == piece.reverse()
                    {
                        for n in 0..(6 - cmp::max(row, col)) {
                            if self.get(PiecePosition::from(row + n + 2, col + n + 2))
                                == Piece::Empty
                            {
                                break;
                            }
                            if self.get(PiecePosition::from(row + n + 2, col + n + 2)) == piece {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    // 检查棋局是否结束
    fn is_ended(&self) -> bool {
        if self.is_able_to_put(self.turn) {
            return false;
        } else {
            if self.is_able_to_put(self.turn.reverse()) {
                return false;
            }
        }
        true
    }

    // 尝试落子，成功时返回 Ok(棋局是否结束)，失败时返回 Err(ActionError)
    fn try_put(&mut self, pos: PiecePosition, player: User) -> Result<bool, ActionError> {
        // 轮到 Black 落子
        if let Piece::Black = self.turn {
            // 有玩家作为 Black
            if let Some(player_black) = &self.player_black {
                // 验证落子发起者
                if &player == player_black {
                    match self.set(pos, Piece::Black) {
                        Ok(_) => {
                            // 检查 White 是否可以落子
                            if self.is_able_to_put(Piece::White) {
                                self.next_turn()
                            }
                        }
                        Err(err) => return Err(err),
                    }
                } else {
                    return Err(ActionError::NotYourTurn);
                }
            // 没有玩家作为 Black
            } else {
                match self.set(pos, Piece::Black) {
                    Ok(_) => {
                        self.player_black = Some(player);
                        self.next_turn();
                    }
                    Err(err) => return Err(err),
                }
            }
        // 轮到 White 落子
        } else {
            if let Some(player_white) = &self.player_white {
                if &player == player_white {
                    match self.set(pos, Piece::White) {
                        Ok(_) => {
                            if self.is_able_to_put(Piece::Black) {
                                self.next_turn()
                            }
                        }
                        Err(err) => return Err(err),
                    }
                } else {
                    return Err(ActionError::NotYourTurn);
                }
            } else {
                match self.set(pos, Piece::White) {
                    Ok(_) => {
                        self.player_white = Some(player);
                        self.next_turn();
                    }
                    Err(err) => return Err(err),
                }
            }
        }
        // 返回棋局状态
        Ok(self.is_ended())
    }

    // 设定下一位轮到的玩家
    fn next_turn(&mut self) {
        self.turn = match self.turn {
            Piece::Black => Piece::White,
            _ => Piece::Black,
        };
    }

    // 获取按钮列表
    fn get_inline_keyboard(&self) -> InlineKeyboardMarkup {
        let mut keyboad: Vec<Vec<InlineKeyboardButton>> = Vec::new();
        for col in 0..8 {
            let mut keyboad_col: Vec<InlineKeyboardButton> = Vec::new();
            for row in 0..8 {
                keyboad_col.push(InlineKeyboardButton::new(
                    self.get(PiecePosition::from(row, col)).to_string(),
                    InlineKeyboardButtonKind::CallbackData(format!("othello_{}_{}", row, col)),
                ));
            }
            keyboad.push(keyboad_col);
        }
        InlineKeyboardMarkup::from(keyboad)
    }

    // 获取双方玩家
    fn get_players(&self) -> String {
        let mut players = String::new();
        if let Some(player_black) = &self.player_black {
            players.push_str("⚫：");
            players += &player_black.first_name;
            if let Some(player_white) = &self.player_white {
                players.push_str("\n⚪：");
                players += &player_white.first_name;
            }
        }
        players
    }

    // 获取下一位轮到的玩家
    fn get_next_player(&self) -> String {
        self.turn.to_string()
    }

    // 获取棋局结果
    fn get_game_result(&self) -> String {
        let mut board = String::new();
        let mut black_count: u8 = 0;
        let mut white_count: u8 = 0;
        for col in 0..8 {
            for row in 0..8 {
                match self.get(PiecePosition::from(row, col)) {
                    Piece::Black => black_count += 1,
                    Piece::White => white_count += 1,
                    _ => (),
                }
                board.push_str(&self.data[row][col].to_string());
            }
            board.push_str("\n");
        }
        match black_count.cmp(&white_count) {
            cmp::Ordering::Less => format!(
                "{}\n⚫：{} ⚪：{}\n\n⚪ 赢了",
                board, black_count, white_count
            ),
            cmp::Ordering::Greater => format!(
                "{}\n⚫：{} ⚪：{}\n\n⚫ 赢了",
                board, black_count, white_count
            ),
            cmp::Ordering::Equal => format!(
                "{}\n⚫：{} ⚪：{}\n\n平局",
                board, black_count, white_count
            ),
        }
    }
}

// 正在进行的棋局列表
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
    if data.starts_with("othello_") {
        let mut data = data[8..].split('_');
        if let Some(row) = data.next() {
            if let Ok(row) = row.parse::<usize>() {
                if let Some(col) = data.next() {
                    if let Ok(col) = col.parse::<usize>() {
                        if row < 8 && col < 8 {
                            if let None = data.next() {
                                return Some(PiecePosition::from(row, col));
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

#[handler(command = "/othello")]
pub async fn othello_command_handler(
    context: &Context,
    command: Command,
) -> Result<HandlerResult, ErrorHandler> {
    let message = command.get_message();
    let chat_id = message.get_chat_id();
    let method = SendMessage::new(chat_id, "黑白棋").reply_markup(
        ReplyMarkup::InlineKeyboardMarkup(Game::new().get_inline_keyboard()),
    );
    context.api.execute(method).await?;
    Ok(HandlerResult::Stop)
}

#[handler]
pub async fn othello_inlinekeyboard_handler(
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
            let mut game_list = session.get("othello").await?.unwrap_or(GameList::new());
            let mut game = game_list.get(message_id);
            // 尝试操作棋局
            match game.try_put(pos, user.clone()) {
                // 操作成功
                Ok(is_ended) => {
                    let method: EditMessageText;
                    // 棋局是否结束
                    if is_ended {
                        method = EditMessageText::new(
                            chat_id,
                            message_id,
                            format!(
                                "黑白棋\n\n{}\n\n{}",
                                game.get_players(),
                                game.get_game_result()
                            ),
                        );
                        // 清理棋局列表
                        if game_list.update_and_check_empty(message_id, None) {
                            session.remove("othello").await?;
                        } else {
                            session.set("othello", &game_list).await?;
                        }
                    } else {
                        method = EditMessageText::new(
                            chat_id,
                            message_id,
                            format!(
                                "黑白棋\n\n{}\n\n轮到：{}",
                                game.get_players(),
                                game.get_next_player()
                            ),
                        )
                        .reply_markup(game.get_inline_keyboard());
                        // 存储棋局
                        game_list.update_and_check_empty(message_id, Some(game.clone()));
                        session.set("othello", &game_list).await?;
                    }
                    context.api.execute(method).await?;
                    // 回应 callback
                    let method = AnswerCallbackQuery::new(query.id);
                    context.api.execute(method).await?;
                }
                // 操作失败
                Err(err) => {
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
