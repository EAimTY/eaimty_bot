use crate::{context::Context, error::ErrorHandler};

use carapax::{
    handler,
    methods::{EditMessageReplyMarkup, EditMessageText, SendMessage},
    session::SessionId,
    types::{
        CallbackQuery, Command, InlineKeyboardButton, InlineKeyboardButtonKind,
        InlineKeyboardMarkup, ReplyMarkup, User,
    },
    HandlerResult,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt;
use tokio::try_join;

const ARROUND: [(i32, i32); 8] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
enum MineBoxes {
    Mine,
    Num(u8),
    Explode,
}

impl fmt::Display for MineBoxes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MineBoxes::Explode => write!(f, "💥"),
            MineBoxes::Mine => write!(f, "💣"),
            MineBoxes::Num(0) => write!(f, "    "),
            MineBoxes::Num(n) => write!(f, "{:4}", n),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
enum MineBoxesState {
    Know(MineBoxes),
    Unknow,
    Flag,
}

impl fmt::Display for MineBoxesState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MineBoxesState::Flag => write!(f, "🚩"),
            MineBoxesState::Unknow => write!(f, "⬜"),
            MineBoxesState::Know(item) => write!(f, "{}", item),
        }
    }
}

enum MineSweeperGameState {
    Win,
    Lose,
    OnGoing,
}

#[derive(Serialize, Deserialize)]
struct MineSweeper {
    id: i64,
    row: usize,
    col: usize,
    data: Vec<MineBoxes>,
    mask: Vec<MineBoxesState>,
    explode_user: Option<User>,
}

impl MineSweeper {
    fn new(id: i64, row: usize, col: usize, mines: usize) -> MineSweeper {
        let mut arr = vec![MineBoxes::Num(0); row * col];
        for i in 0..mines {
            arr[i] = MineBoxes::Mine;
        }

        for i in 0..row * col {
            let rng = rand::thread_rng().gen_range(0..row * col);
            let a = arr[i];
            arr[i] = arr[rng];
            arr[rng] = a;
        }

        for r in 0..row {
            for c in 0..col {
                if let MineBoxes::Mine = arr[r * col + c] {
                    for (_r, _c) in ARROUND {
                        if r as i32 + _r >= 0
                            && r as i32 + _r < col as i32
                            && c as i32 + _c >= 0
                            && c as i32 + _c < row as i32
                        {
                            match arr[(r as i32 + _r) as usize * col + (c as i32 + _c) as usize] {
                                MineBoxes::Num(n) => {
                                    arr[(r as i32 + _r) as usize * col + (c as i32 + _c) as usize] =
                                        MineBoxes::Num(n + 1)
                                }
                                _ => (),
                            }
                        }
                    }
                }
            }
        }

        MineSweeper {
            id,
            row,
            col,
            data: arr,
            mask: vec![MineBoxesState::Unknow; row * col],
            explode_user: None,
        }
    }

    fn get_inline_keyboard(&self) -> InlineKeyboardMarkup {
        let mut keyboad: Vec<Vec<InlineKeyboardButton>> = Vec::new();
        for r in 0..self.row {
            let mut keyboad_row: Vec<InlineKeyboardButton> = Vec::new();
            for c in 0..self.col {
                keyboad_row.push(InlineKeyboardButton::new(
                    self.mask[r * self.col + c].to_string(),
                    InlineKeyboardButtonKind::CallbackData(
                        String::from("mine_") + &r.to_string() + "_" + &c.to_string(),
                    ),
                ));
            }
            keyboad.push(keyboad_row);
        }
        InlineKeyboardMarkup::from(keyboad)
    }

    fn set_flag(&mut self, r: usize, c: usize) {
        match self.mask[r * self.col + c] {
            MineBoxesState::Unknow => self.mask[r * self.col + c] = MineBoxesState::Flag,
            _ => (),
        }

        // if self.check() {
        //     self.game_over(true)
        // }
    }

    fn open(&mut self, r: usize, c: usize) {
        let mut queue = Vec::new();
        match self.mask[r * self.col + c] {
            MineBoxesState::Unknow => {
                match self.data[r * self.col + c] {
                    MineBoxes::Mine => {
                        self.data[r * self.col + c] = MineBoxes::Explode;
                        self.mask[r * self.col + c] = MineBoxesState::Know(MineBoxes::Explode);
                    }
                    MineBoxes::Num(_) => {
                        queue.push((r, c));
                        while queue.len() != 0 {
                            let cell = queue.remove(0);
                            match self.data[cell.0 * self.col + cell.1] {
                                MineBoxes::Num(0) => {
                                    self.mask[cell.0 * self.col + cell.1] =
                                        MineBoxesState::Know(self.data[cell.0 * self.col + cell.1]);

                                    for (_r, _c) in ARROUND {
                                        if cell.0 as i32 + _r >= 0
                                            && cell.0 as i32 + _r < self.col as i32
                                            && cell.1 as i32 + _c >= 0
                                            && cell.1 as i32 + _c < self.row as i32
                                        {
                                            if let MineBoxesState::Unknow =
                                                self.mask[(cell.0 as i32 + _r) as usize * self.col
                                                    + (cell.1 as i32 + _c) as usize]
                                            {
                                                queue.push((
                                                    (cell.0 as i32 + _r) as usize,
                                                    (cell.1 as i32 + _c) as usize,
                                                ))
                                            }
                                        }
                                    }
                                }
                                MineBoxes::Num(_) => {
                                    self.mask[cell.0 * self.col + cell.1] =
                                        MineBoxesState::Know(self.data[cell.0 * self.col + cell.1])
                                }
                                _ => (),
                            }
                        }
                        // self.mask[r * self.col + c] =
                        //     MineBoxesState::Know(self.data[r * self.col + c]);
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    }

    fn open_num(&mut self, r: usize, c: usize) {
        match self.mask[r * self.col + c] {
            MineBoxesState::Know(MineBoxes::Num(0)) => (),
            MineBoxesState::Know(MineBoxes::Num(n)) => {
                let mut flags = 0;
                let mut unknows = 0;
                for (_r, _c) in ARROUND {
                    if r as i32 + _r >= 0
                        && r as i32 + _r < self.col as i32
                        && c as i32 + _c >= 0
                        && c as i32 + _c < self.row as i32
                    {
                        if let MineBoxesState::Flag = self.mask
                            [(r as i32 + _r) as usize * self.col + (c as i32 + _c) as usize]
                        {
                            flags += 1;
                        }
                        if let MineBoxesState::Unknow = self.mask
                            [(r as i32 + _r) as usize * self.col + (c as i32 + _c) as usize]
                        {
                            unknows += 1;
                        }
                    }
                }

                if n == flags {
                    // open all
                    for (_r, _c) in ARROUND {
                        if r as i32 + _r >= 0
                            && r as i32 + _r < self.col as i32
                            && c as i32 + _c >= 0
                            && c as i32 + _c < self.row as i32
                        {
                            self.open((r as i32 + _r) as usize, (c as i32 + _c) as usize);
                        }
                    }
                }
                if n == flags + unknows {
                    // println!("n:{} flags:{} unknows:{}", n, flags, unknows);
                    // flag all
                    for (_r, _c) in ARROUND {
                        if r as i32 + _r >= 0
                            && r as i32 + _r < self.col as i32
                            && c as i32 + _c >= 0
                            && c as i32 + _c < self.row as i32
                        {
                            self.set_flag((r as i32 + _r) as usize, (c as i32 + _c) as usize);
                        }
                    }
                }
            }
            _ => (),
        }

        // if self.check() {
        //     self.game_over(true)
        // }
    }

    fn check(&self) -> MineSweeperGameState {
        let mut is_win = true;
        for r in 0..self.row {
            for c in 0..self.col {
                match self.mask[r * self.col + c] {
                    MineBoxesState::Know(MineBoxes::Explode) => return MineSweeperGameState::Lose,

                    MineBoxesState::Unknow => {
                        is_win = false;
                    }

                    MineBoxesState::Flag => {
                        if let MineBoxes::Mine = self.data[r * self.col + c] {
                        } else {
                            is_win = false;
                        }
                    }

                    _ => continue,
                }
            }
        }
        if is_win {
            MineSweeperGameState::Win
        } else {
            MineSweeperGameState::OnGoing
        }
    }
}

trait MineVec {
    fn get_index(&mut self, id: i64) -> usize;
}

impl MineVec for Vec<MineSweeper> {
    fn get_index(&mut self, id: i64) -> usize {
        match self.iter().position(|v| v.id == id) {
            Some(index) => index,
            None => {
                self.push(MineSweeper::new(id, 8, 8, 10));
                self.len() - 1
            }
        }
    }
}

#[handler(command = "/mine")]
pub async fn mine_command_handler(
    context: &Context,
    command: Command,
) -> Result<HandlerResult, ErrorHandler> {
    let message = command.get_message();
    let chat_id = message.get_chat_id();

    if let Some(_) = message.get_user() {
        let method = SendMessage::new(chat_id, "Mine").reply_markup(
            ReplyMarkup::InlineKeyboardMarkup(MineSweeper::new(0, 8, 8, 10).get_inline_keyboard()),
        );
        context.api.execute(method).await?;
    }
    Ok(HandlerResult::Stop)
}

#[handler]
pub async fn mine_inlinekeyboard_handler(
    context: &Context,
    query: CallbackQuery,
) -> Result<HandlerResult, ErrorHandler> {
    let data = query.data;
    if let Some(data) = data {
        let cell: Option<(usize, usize)> = {
            let splits: Vec<&str> = data.split('_').collect();
            if splits[0] == "mine" {
                Some((splits[1].parse().unwrap(), splits[2].parse().unwrap()))
            } else {
                None
            }
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
                let mut mine = session.get("mine").await?.unwrap_or(Vec::new());
                let index = mine.get_index(message_id);
                let edit_message: Option<EditMessageText> = None;
                match mine[index].mask[cell.0 * mine[index].col + cell.1] {
                    MineBoxesState::Know(MineBoxes::Num(0)) => (),
                    MineBoxesState::Know(MineBoxes::Num(_)) => mine[index].open_num(cell.0, cell.1),
                    MineBoxesState::Unknow => mine[index].open(cell.0, cell.1),

                    _ => (),
                }
                match mine[index].check() {
                    MineSweeperGameState::Win => {
                        let edit_reply_markup = EditMessageReplyMarkup::new(chat_id, message_id)
                            .reply_markup({
                                let mut keyboad: Vec<Vec<InlineKeyboardButton>> = Vec::new();
                                for r in 0..mine[index].row {
                                    let mut keyboad_row: Vec<InlineKeyboardButton> = Vec::new();
                                    for c in 0..mine[index].col {
                                        keyboad_row.push(InlineKeyboardButton::new(
                                            mine[index].mask[r * mine[index].col + c].to_string(),
                                            InlineKeyboardButtonKind::CallbackData(String::from(
                                                "none",
                                            )),
                                        ));
                                    }
                                    keyboad.push(keyboad_row);
                                }
                                InlineKeyboardMarkup::from(keyboad)
                            });
                        context.api.execute(edit_reply_markup).await?;
                        let method = SendMessage::new(chat_id, "WIN")
                            .reply_to_message_id(mine[index].id);
                        context.api.execute(method).await?;
                        mine.remove(index);
                        if mine.is_empty() {
                            session.remove("mine").await?;
                        } else {
                            session.set("mine", &mine).await?;
                        }
                    }
                    MineSweeperGameState::Lose => {
                        let edit_reply_markup = EditMessageReplyMarkup::new(chat_id, message_id)
                            .reply_markup({
                                let mut keyboad: Vec<Vec<InlineKeyboardButton>> = Vec::new();
                                for r in 0..mine[index].row {
                                    let mut keyboad_row: Vec<InlineKeyboardButton> = Vec::new();
                                    for c in 0..mine[index].col {
                                        keyboad_row.push(InlineKeyboardButton::new(
                                            mine[index].data[r * mine[index].col + c].to_string(),
                                            InlineKeyboardButtonKind::CallbackData(String::from(
                                                "none",
                                            )),
                                        ));
                                    }
                                    keyboad.push(keyboad_row);
                                }
                                InlineKeyboardMarkup::from(keyboad)
                            });
                        context.api.execute(edit_reply_markup).await?;
                        let method = SendMessage::new(
                            chat_id,
                            format!(
                                "FAIL\n{} click the mine",
                                user.username.unwrap_or(user.first_name)
                            ),
                        )
                        .reply_to_message_id(mine[index].id);
                        context.api.execute(method).await?;
                        mine.remove(index);
                        if mine.is_empty() {
                            session.remove("mine").await?;
                        } else {
                            session.set("mine", &mine).await?;
                        }
                    }
                    MineSweeperGameState::OnGoing => {
                        session.set("mine", &mine).await?;
                        let edit_reply_markup = EditMessageReplyMarkup::new(chat_id, message_id)
                            .reply_markup(mine[index].get_inline_keyboard());
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
                }
            }
        }
    }
    Ok(HandlerResult::Continue)
}
