use crate::{context::Context, error::ErrorHandler};
use carapax::{
    handler,
    methods::{AnswerCallbackQuery, EditMessageText, SendMessage},
    session::SessionId,
    types::{
        CallbackQuery, Command, InlineKeyboardButton, InlineKeyboardButtonKind,
        InlineKeyboardMarkup, ReplyMarkup,
    },
    HandlerResult,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    fmt,
};

// 地雷情况
#[derive(Copy, Clone, Serialize, Deserialize)]
enum BoxType {
    Mine,
    MineCount(u8),
}

// 显示情况
#[derive(Copy, Clone, Serialize, Deserialize)]
enum MaskType {
    Masked,
    Unmasked,
    Flagged,
    Exploded,
}

// 块类型
#[derive(Copy, Clone, Serialize, Deserialize)]
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

    fn get_box_type(&self) -> BoxType {
        self.box_type
    }

    fn set_mine_count(&mut self, mine_count: u8) {
        self.box_type = BoxType::MineCount(mine_count);
    }

    fn get_mask_type(&self) -> MaskType {
        self.mask_type
    }

    fn set_mask_type(&mut self, mask_type: MaskType) {
        self.mask_type = mask_type;
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

    // 从下标获取位置
    fn from_index(index: usize, map_size: (usize, usize)) -> Self {
        Self {
            index: Some(index),
            row: index / map_size.1,
            col: index % map_size.1,
        }
    }

    // 从坐标获取位置（不获取地图大小，故返回一个没有下标的 BoxPosition）
    fn from_coords_no_index(coords: (usize, usize)) -> Self {
        Self {
            index: None,
            row: coords.0,
            col: coords.1,
        }
    }

    // 通过输入的地图大小计算下标
    fn set_index(&mut self, map_size: (usize, usize)) {
        self.index = Some(self.row * map_size.1 + self.col);
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
impl std::ops::Index<&BoxPosition> for Vec<MineBox> {
    type Output = MineBox;

    fn index(&self, index: &BoxPosition) -> &MineBox {
        &self[index.get_index()]
    }
}

impl std::ops::IndexMut<&BoxPosition> for Vec<MineBox> {
    fn index_mut(&mut self, index: &BoxPosition) -> &mut MineBox {
        &mut self[index.get_index()]
    }
}

// 用于迭代周围块的类型
struct BoxesAround {
    // 保存可能相邻的 8 个位置的元组数组
    around: [(i8, i8); 8],
    // 迭代器位置
    offset: usize,
    // 地图大小
    map_height: usize,
    map_width: usize,
}

impl BoxesAround {
    fn from(pos: &BoxPosition, map_size: (usize, usize)) -> Self {
        Self {
            around: {
                // 通过输入的位置计算出可能相邻的 8 个位置
                let (row, col) = (pos.get_row() as i8, pos.get_col() as i8);
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
impl Iterator for BoxesAround {
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
        for index in 0..height * width {
            map.swap(index, rand::thread_rng().gen_range(0..height * width));
        }
        map
    }

    // 计算每块周围的地雷数量
    fn map_calc_mine_count(mut map: Vec<MineBox>, map_size: (usize, usize)) -> Vec<MineBox> {
        let (height, width) = map_size;
        for index in 0..height * width {
            if let BoxType::MineCount(_) = map[index].get_box_type() {
                let mut counter: u8 = 0;
                // 遍历周围块
                for around_pos in
                    BoxesAround::from(&BoxPosition::from_index(index, map_size), map_size)
                {
                    if let BoxType::Mine = map[&around_pos].get_box_type() {
                        counter += 1;
                    }
                }
                map[index].set_mine_count(counter);
            }
        }
        map
    }

    // 重新生成地图
    fn regenerate_map(mut self) {
        let map_size = (self.height, self.width);
        self.map = Self::map_calc_mine_count(Self::map_reorder(self.map, map_size), map_size);
    }

    // 检查地图中有目标块
    fn contains(&self, pos: &BoxPosition) -> bool {
        if self.height > pos.get_row() && self.width > pos.get_col() {
            return true;
        }
        false
    }

    // Unmask 所有块
    fn unmask_all(&mut self) {
        for index in 0..self.height * self.width {
            let mut mine_box = self.map[index];
            if let MaskType::Masked = mine_box.get_mask_type() {
                self.map[index] = {
                    mine_box.set_mask_type(MaskType::Unmasked);
                    mine_box
                };
            }
        }
    }

    // 检查游戏是否成功
    fn is_succeeded(&self) -> bool {
        for index in 0..self.height * self.width {
            if let MaskType::Masked = self.map[index].get_mask_type() {
                if let BoxType::MineCount(_) = self.map[index].get_box_type() {
                    return false;
                }
            }
        }
        true
    }

    // 点击地图中目标块
    fn click(&mut self, mut pos: BoxPosition) -> GameState {
        let game_state;
        // 为目标位置计算下标
        pos.set_index((self.height, self.width));
        // 获取目标块并处理
        let mut mine_box = self.map[&pos];
        match mine_box.get_mask_type() {
            MaskType::Masked => {
                // 判断是否点击了地雷
                if let BoxType::MineCount(mine_count) = mine_box.get_box_type() {
                    if mine_count > 0 {
                        // 块周围有地雷，仅 Unmask 块本身
                        self.map[&pos] = {
                            mine_box.set_mask_type(MaskType::Unmasked);
                            mine_box
                        }
                    } else {
                        // 块周围没有地雷，继续遍历周围块的周围块
                        // 创建待遍历队列
                        let mut queue = VecDeque::new();
                        queue.push_back(pos);
                        // 待遍历队列不为空时，遍历队列头部周围的位置
                        while let Some(pos) = queue.pop_front() {
                            // Unmask 当前块
                            self.map[&pos] = {
                                let mut mine_box = self.map[&pos];
                                mine_box.set_mask_type(MaskType::Unmasked);
                                mine_box
                            };
                            // 遍历当前块的周围块
                            for around_pos in BoxesAround::from(&pos, (self.height, self.width)) {
                                let mut mine_box = self.map[&around_pos];
                                // 仅处理 Masked 块
                                if let MaskType::Masked = mine_box.get_mask_type() {
                                    if let BoxType::MineCount(mine_count) = mine_box.get_box_type()
                                    {
                                        if mine_count > 0 {
                                            // 块周围有地雷，仅 Unmask 块本身
                                            self.map[&around_pos] = {
                                                mine_box.set_mask_type(MaskType::Unmasked);
                                                mine_box
                                            }
                                        } else {
                                            // 块周围没有地雷，入待遍历队列
                                            queue.push_back(around_pos);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // 检查游戏是否已经成功
                    if self.is_succeeded() {
                        self.unmask_all();
                        game_state = GameState::Succeeded;
                    } else {
                        game_state = GameState::OnGoing;
                    }
                } else {
                    // 点击了地雷，游戏失败，标记目标块为爆炸
                    self.map[&pos] = {
                        mine_box.set_mask_type(MaskType::Exploded);
                        mine_box
                    };
                    self.unmask_all();
                    game_state = GameState::Failed;
                }
            }
            MaskType::Unmasked => {
                // 判断是否可插旗标记
                game_state = GameState::OnGoing;
            }
            _ => {
                // 不处理对已插旗块或已爆炸块的操作
                game_state = GameState::OnGoing;
            }
        }
        game_state
    }

    // 获取按钮列表
    fn get_inline_keyboard(&self) -> InlineKeyboardMarkup {
        let mut keyboad: Vec<Vec<InlineKeyboardButton>> = Vec::new();
        for col in 0..self.height {
            let mut keyboad_col: Vec<InlineKeyboardButton> = Vec::new();
            for row in 0..self.width {
                keyboad_col.push(InlineKeyboardButton::new(
                    self.map[&BoxPosition::from_coords((row, col), (self.height, self.width))]
                        .to_string(),
                    InlineKeyboardButtonKind::CallbackData(format!("minesweeper_{}_{}", row, col)),
                ));
            }
            keyboad.push(keyboad_col);
        }
        InlineKeyboardMarkup::from(keyboad)
    }

    // 获取文字形式的地图
    fn get_game_board(&self) -> String {
        let mut map = String::new();
        for col in 0..self.width {
            for row in 0..self.height {
                map.push_str(
                    &self.map[&BoxPosition::from_coords((row, col), (self.height, self.width))]
                        .to_string(),
                );
            }
            map.push_str("\n");
        }
        map
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
                let method;
                // 操作地图并检查游戏是否结束
                match game.click(pos) {
                    // 游戏失败
                    GameState::Failed => {
                        method = EditMessageText::new(
                            chat_id,
                            message_id,
                            format!("扫雷失败\n\n{}", game.get_game_board()),
                        );
                        // 清理游戏列表
                        if game_list.update_and_check_empty(message_id, None) {
                            session.remove("minesweeper").await?;
                        } else {
                            session.set("minesweeper", &game_list).await?;
                        }
                    }
                    // 游戏正在进行
                    GameState::OnGoing => {
                        method = EditMessageText::new(chat_id, message_id, "扫雷")
                            .reply_markup(game.get_inline_keyboard());
                        // 存储游戏
                        game_list.update_and_check_empty(message_id, Some(game.clone()));
                        session.set("minesweeper", &game_list).await?;
                    }
                    // 游戏成功
                    GameState::Succeeded => {
                        method = EditMessageText::new(
                            chat_id,
                            message_id,
                            format!("扫雷成功\n\n{}", game.get_game_board()),
                        );
                        // 清理游戏列表
                        if game_list.update_and_check_empty(message_id, None) {
                            session.remove("minesweeper").await?;
                        } else {
                            session.set("minesweeper", &game_list).await?;
                        }
                    }
                }
                context.api.execute(method).await?;
                // 回应 callback
                let method = AnswerCallbackQuery::new(query.id);
                context.api.execute(method).await?;
                return Ok(HandlerResult::Stop);
            }
        }
    }
    Ok(HandlerResult::Continue)
}
