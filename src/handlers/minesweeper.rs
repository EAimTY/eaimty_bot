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
use rand::{distributions::Open01, Rng};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, fmt};

// 地雷情况
#[derive(Clone, Serialize, Deserialize)]
enum BoxType {
    Mine,
    MineCount(u8),
}

// 显示情况
#[derive(Clone, Serialize, Deserialize)]
enum MaskType {
    Masked,
    Unmasked,
    Flagged,
    Exploded,
}

// 块类型
#[derive(Clone, Serialize, Deserialize)]
struct MineBox {
    box_type: BoxType,
    mask_type: MaskType,
}

impl MineBox {
    fn new(is_mine: bool) -> Self {
        Self {
            box_type: if is_mine {
                BoxType::Mine
            } else {
                BoxType::MineCount(0)
            },
            mask_type: MaskType::Masked,
        }
    }

    fn is_mine(&self) -> bool {
        match self.box_type {
            BoxType::Mine => true,
            BoxType::MineCount(_) => false,
        }
    }

    fn set_mine_count(&mut self, mine_count: u8) {
        self.box_type = BoxType::MineCount(mine_count);
    }
}

impl fmt::Display for MineBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.mask_type {
            MaskType::Masked => write!(f, "➕"),
            MaskType::Unmasked => {
                if let BoxType::MineCount(mine_count) = self.box_type {
                    match mine_count {
                        1 => write!(f, "1️⃣"),
                        2 => write!(f, "2️⃣"),
                        3 => write!(f, "3️⃣"),
                        4 => write!(f, "4️⃣"),
                        5 => write!(f, "5️⃣"),
                        6 => write!(f, "6️⃣"),
                        7 => write!(f, "7️⃣"),
                        8 => write!(f, "8️⃣"),
                        _ => write!(f, "➖"),
                    }
                } else {
                    write!(f, "💣")
                }
            }
            MaskType::Flagged => write!(f, "🚩"),
            MaskType::Exploded => write!(f, "💥"),
        }
    }
}

// 位置类型
struct BoxPosition {
    index: Option<usize>,
    row: usize,
    col: usize,
}

impl BoxPosition {
    // 从坐标获取位置
    fn from_coords(coords: (usize, usize), map_size: (usize, usize)) -> Self {
        Self {
            index: Some(coords.0 * map_size.1 + coords.1),
            row: coords.0,
            col: coords.1,
        }
    }

    // 从坐标获取位置
    fn from_coords_no_index(coords: (usize, usize)) -> Self {
        Self {
            index: None,
            row: coords.0,
            col: coords.1,
        }
    }

    // 从下标获取位置
    fn from_index(index: usize, map_size: (usize, usize)) -> Self {
        Self {
            index: Some(index),
            row: index / map_size.1,
            col: index % map_size.1,
        }
    }

    // 尝试解析 callback data，返回目标坐标（可能超出棋盘）
    fn try_parse_callback(data: String) -> Option<Self> {
        if data.starts_with("minesweeper_") {
            let mut data = data[12..].split('_');
            if let Some(row) = data.next() {
                if let Ok(row) = row.parse::<usize>() {
                    if let Some(col) = data.next() {
                        if let Ok(col) = col.parse::<usize>() {
                            if let None = data.next() {
                                return Some(Self::from_coords_no_index((row, col)));
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn get_index(&self) -> usize {
        self.index.unwrap_or(0)
    }

    fn get_row(&self) -> usize {
        self.row
    }

    fn get_col(&self) -> usize {
        self.col
    }
}

// 将位置类型作为 Vec<MineBox> 下标
impl std::ops::Index<BoxPosition> for Vec<MineBox> {
    type Output = MineBox;

    fn index(&self, index: BoxPosition) -> &MineBox {
        &self[index.get_index()]
    }
}

impl std::ops::IndexMut<BoxPosition> for Vec<MineBox> {
    fn index_mut(&mut self, index: BoxPosition) -> &mut MineBox {
        &mut self[index.get_index()]
    }
}

// 用于迭代周围块的类型
struct BoxAround {
    // 保存可能相邻的 8 个位置的元组数组
    around: [(i8, i8); 8],
    // 迭代器位置
    offset: usize,
    // 地图大小
    map_height: usize,
    map_width: usize,
}

impl BoxAround {
    fn from(position: BoxPosition, map_size: (usize, usize)) -> Self {
        Self {
            around: {
                // 通过输入的位置计算出可能相邻的 8 个位置
                let (row, col) = (position.get_row() as i8, position.get_col() as i8);
                [
                    (row - 1, col - 1),
                    (row, col - 1),
                    (row + 1, col - 1),
                    (row - 1, col),
                    (row + 1, col),
                    (row - 1, col + 1),
                    (row, col + 1),
                    (row + 1, col + 1),
                ]
            },
            offset: 0,
            map_height: map_size.0,
            map_width: map_size.1,
        }
    }
}

// 周围块迭代器实现
impl Iterator for BoxAround {
    type Item = BoxPosition;

    fn next(&mut self) -> Option<Self::Item> {
        // 从下标为 offset 处开始遍历可能相邻的位置
        for (index, (row, col)) in self.around[self.offset..].iter().enumerate() {
            // 判断位置合法
            if row >= &0
                && row < &(self.map_height as i8)
                && col >= &0
                && col < &(self.map_width as i8)
            {
                // 更新 offset 并返回位置
                self.offset += index;
                self.offset += 1;
                return Some(BoxPosition::from_coords(
                    (*row as usize, *col as usize),
                    (self.map_height, self.map_width),
                ));
            }
        }
        None
    }
}

enum GameState {
    Failed,
    OnGoing,
    Succeeded,
}

// 地图
#[derive(Clone, Serialize, Deserialize)]
struct Game {
    map: Vec<MineBox>,
    height: usize,
    width: usize,
    mine_count: usize,
}

impl Game {
    fn new(map_size: (usize, usize), mine_count: usize) -> Self {
        let (height, width) = map_size;
        Self {
            map: {
                // 新建一个大小为 height * width，头部 mine_count 块为地雷的地图
                let mut map = vec![MineBox::new(true); mine_count];
                map.append(&mut vec![MineBox::new(false); height * width - mine_count]);
                // 打乱地雷位置并计算每块周围的地雷数量
                Self::map_calc_mine_count(Self::map_reorder(map, map_size), map_size)
            },
            height,
            width,
            mine_count,
        }
    }

    // 打乱地雷位置
    fn map_reorder(mut map: Vec<MineBox>, map_size: (usize, usize)) -> Vec<MineBox> {
        let (height, width) = map_size;
        for pos in 0..height * width {
            map.swap(pos, rand::thread_rng().gen_range(0..height * width));
        }
        map
    }

    // 计算每块周围的地雷数量
    fn map_calc_mine_count(mut map: Vec<MineBox>, map_size: (usize, usize)) -> Vec<MineBox> {
        let (height, width) = map_size;
        for pos in 0..height * width {
            if !map[pos].is_mine() {
                let mut counter: u8 = 0;
                // 遍历周围块
                for around_pos in BoxAround::from(BoxPosition::from_index(pos, map_size), map_size)
                {
                    if map[around_pos].is_mine() {
                        counter += 1;
                    }
                }
                map[pos].set_mine_count(counter);
            }
        }
        map
    }

    // 重新生成地图
    fn regenerate_map(mut self) {
        let map_size = (self.height, self.width);
        self.map = Self::map_calc_mine_count(Self::map_reorder(self.map, map_size), map_size);
    }

    // 获取目标块
    fn get(&self, position: BoxPosition) -> MineBox {
        self.map[position].clone()
    }

    // 检查地图中有目标块
    fn contains(&self, position: &BoxPosition) -> bool {
        if self.height > position.get_row() && self.width > position.get_col() {
            return true;
        }
        false
    }

    // 点击地图中目标块
    fn click(&mut self, position: BoxPosition) -> GameState {
        // TODO

        GameState::OnGoing
    }

    // 获取按钮列表
    fn get_inline_keyboard(&self) -> InlineKeyboardMarkup {
        let mut keyboad: Vec<Vec<InlineKeyboardButton>> = Vec::new();
        for col in 0..self.height {
            let mut keyboad_col: Vec<InlineKeyboardButton> = Vec::new();
            for row in 0..self.width {
                keyboad_col.push(InlineKeyboardButton::new(
                    self.get(BoxPosition::from_coords(
                        (row, col),
                        (self.height, self.width),
                    ))
                    .to_string(),
                    InlineKeyboardButtonKind::CallbackData(format!("minesweeper_{}_{}", row, col)),
                ));
            }
            keyboad.push(keyboad_col);
        }
        InlineKeyboardMarkup::from(keyboad)
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
        self.list.entry(id).or_insert(Game::new((8, 8), 8)).clone()
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

#[handler(command = "/minesweeper")]
pub async fn minesweeper_command_handler(
    context: &Context,
    command: Command,
) -> Result<HandlerResult, ErrorHandler> {
    let message = command.get_message();
    let chat_id = message.get_chat_id();
    // 创建新游戏
    let game = Game::new((8, 8), 8);
    // 从 session 获取正在进行的游戏列表
    let mut session = context
        .session_manager
        .get_session(SessionId::new(chat_id, 0))?;
    let mut game_list = session.get("minesweeper").await?.unwrap_or(GameList::new());
    // 向列表中添加游戏
    game_list.update_and_check_empty(message.id, Some(game.clone()));
    session.set("minesweeper", &game_list).await?;
    // 发送游戏地图
    let method = SendMessage::new(chat_id, "扫雷").reply_markup(ReplyMarkup::InlineKeyboardMarkup(
        game.get_inline_keyboard(),
    ));
    context.api.execute(method).await?;
    Ok(HandlerResult::Stop)
}

#[handler]
pub async fn minesweeper_inlinekeyboard_handler(
    context: &Context,
    query: CallbackQuery,
) -> Result<HandlerResult, ErrorHandler> {
    // 检查非空 query
    if let Some(data) = query.data {
        // 尝试 parse callback data
        if let Some(pos) = BoxPosition::try_parse_callback(data) {
            let message = query.message.unwrap();
            let chat_id = message.get_chat_id();
            let message_id = message.id;
            let user = query.from;
            // 从 session 获取游戏
            let mut session = context
                .session_manager
                .get_session(SessionId::new(chat_id, 0))?;
            let mut game_list = session.get("minesweeper").await?.unwrap_or(GameList::new());
            let mut game = game_list.get(message_id);
            // 检查操作目标块在游戏地图范围内
            if game.contains(&pos) {
                // 操作地图并检查游戏是否结束
                match game.click(pos) {
                    // 游戏失败
                    GameState::Failed => {
                        // 清理棋局列表
                        if game_list.update_and_check_empty(message_id, None) {
                            session.remove("minesweeper").await?;
                        } else {
                            session.set("minesweeper", &game_list).await?;
                        }
                    }
                    // 游戏正在进行
                    GameState::OnGoing => {
                        // 存储棋局
                        game_list.update_and_check_empty(message_id, Some(game.clone()));
                        session.set("minesweeper", &game_list).await?;
                    }
                    // 游戏成功
                    GameState::Succeeded => {
                        // 清理棋局列表
                        if game_list.update_and_check_empty(message_id, None) {
                            session.remove("minesweeper").await?;
                        } else {
                            session.set("minesweeper", &game_list).await?;
                        }
                    }
                }
                // 回应 callback
                let method = AnswerCallbackQuery::new(query.id);
                context.api.execute(method).await?;
                return Ok(HandlerResult::Stop);
            }
        }
    }
    Ok(HandlerResult::Continue)
}
