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
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, fmt};
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
    Fail,
    OnGoing,
}

#[derive(Serialize, Deserialize)]
struct MineSweeper {
    id: i64,
    row: usize,
    col: usize,
    mines: usize,
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
            mines,
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
                        String::from("minesweeper_") + &r.to_string() + "_" + &c.to_string(),
                    ),
                ));
            }
            keyboad.push(keyboad_row);
        }
        InlineKeyboardMarkup::from(keyboad)
    }

    fn get_end_inline_keyboard(&self, state: &MineSweeperGameState) -> InlineKeyboardMarkup {
        let mut keyboad: Vec<Vec<InlineKeyboardButton>> = Vec::new();
        for r in 0..self.row {
            let mut keyboad_row: Vec<InlineKeyboardButton> = Vec::new();
            for c in 0..self.col {
                keyboad_row.push(InlineKeyboardButton::new(
                    {
                        match state {
                            MineSweeperGameState::Win => self.mask[r * self.col + c].to_string(),
                            MineSweeperGameState::Fail => self.data[r * self.col + c].to_string(),
                            MineSweeperGameState::OnGoing => "".to_string(),
                        }
                    },
                    InlineKeyboardButtonKind::CallbackData(
                        String::from("none_") + &r.to_string() + "_" + &c.to_string(),
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
    }

    fn open(&mut self, r: usize, c: usize) {
        let mut queue = VecDeque::new();
        match self.mask[r * self.col + c] {
            MineBoxesState::Unknow => match self.data[r * self.col + c] {
                MineBoxes::Mine => {
                    self.data[r * self.col + c] = MineBoxes::Explode;
                    self.mask[r * self.col + c] = MineBoxesState::Know(MineBoxes::Explode);
                }
                MineBoxes::Num(_) => {
                    queue.push_back((r, c));
                    loop {
                        let cell = match queue.pop_front() {
                            Some(cell) => cell,
                            None => break,
                        };
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
                                            queue.push_back((
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
                }
                _ => (),
            },
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
    }

    fn check(&self) -> MineSweeperGameState {
        let mut is_win = true;
        for r in 0..self.row {
            for c in 0..self.col {
                match self.mask[r * self.col + c] {
                    MineBoxesState::Know(MineBoxes::Explode) => return MineSweeperGameState::Fail,

                    MineBoxesState::Unknow | MineBoxesState::Flag => {
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
    fn get_index(&mut self, id: i64) -> Result<usize, ()>;
}

impl MineVec for Vec<MineSweeper> {
    fn get_index(&mut self, id: i64) -> Result<usize, ()> {
        match self.iter().position(|v| v.id == id) {
            Some(index) => Ok(index),
            None => Err(()),
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
    let args = command.get_args();
    let mut session = context
        .session_manager
        .get_session(SessionId::new(chat_id, 0))?;
    let mut minesweeper = session.get("minesweeper").await?.unwrap_or(Vec::new());
    match args.len() {
        0 => {
            minesweeper.push(MineSweeper::new(message.id, 8, 8, 10));

            let method = SendMessage::new(chat_id, "Mine")
                .reply_markup(ReplyMarkup::InlineKeyboardMarkup(
                    minesweeper[minesweeper.len() - 1].get_inline_keyboard(),
                ))
                .reply_to_message_id(message.id);
            context.api.execute(method).await?;
            session.set("minesweeper", &minesweeper).await?;
        }
        3 => {
            let row: usize = match args[0].parse() {
                Ok(row) => row,
                Err(_) => {
                    context
                        .api
                        .execute(
                            SendMessage::new(chat_id, "Wrong args!")
                                .reply_to_message_id(message.id),
                        )
                        .await?;
                    return Ok(HandlerResult::Stop);
                }
            };
            let col: usize = match args[1].parse() {
                Ok(col) => col,
                Err(_) => {
                    context
                        .api
                        .execute(
                            SendMessage::new(chat_id, "Wrong args!")
                                .reply_to_message_id(message.id),
                        )
                        .await?;
                    return Ok(HandlerResult::Stop);
                }
            };
            let mines: usize = match args[2].parse() {
                Ok(mines) => mines,
                Err(_) => {
                    context
                        .api
                        .execute(
                            SendMessage::new(chat_id, "Wrong args!")
                                .reply_to_message_id(message.id),
                        )
                        .await?;
                    return Ok(HandlerResult::Stop);
                }
            };
            if row > 20 || col > 8 || mines > row * col / 2 || mines < row * col / 10 {
                context
                    .api
                    .execute(
                        SendMessage::new(chat_id, "Args out of range!")
                            .reply_to_message_id(message.id),
                    )
                    .await?;
                return Ok(HandlerResult::Stop);
            } else {
                minesweeper.push(MineSweeper::new(message.id, row, col, mines));
                let method = SendMessage::new(chat_id, "Mine")
                    .reply_markup(ReplyMarkup::InlineKeyboardMarkup(
                        minesweeper[minesweeper.len() - 1].get_inline_keyboard(),
                    ))
                    .reply_to_message_id(message.id);
                context.api.execute(method).await?;
                session.set("minesweeper", &minesweeper).await?;
            }
        }
        _ => {
            context
                .api
                .execute(SendMessage::new(chat_id, "Wrong args!").reply_to_message_id(message.id))
                .await?;
        }
    }

    Ok(HandlerResult::Stop)
}

#[handler]
pub async fn minesweeper_inlinekeyboard_handler(
    context: &Context,
    query: CallbackQuery,
) -> Result<HandlerResult, ErrorHandler> {
    let data = query.data;
    if let Some(data) = data {
        let cell: Option<(usize, usize)> = {
            let splits: Vec<&str> = data.split('_').collect();
            if splits[0] == "minesweeper" {
                if let (Ok(r), Ok(c)) = (splits[1].parse(), splits[2].parse()) {
                    Some((r, c))
                } else {
                    context
                        .api
                        .execute(AnswerCallbackQuery::new(query.id))
                        .await?;
                    None
                }
            } else {
                context
                    .api
                    .execute(AnswerCallbackQuery::new(query.id))
                    .await?;
                None
            }
        };
        if let Some(cell) = cell {
            let message = query.message.unwrap();
            let chat_id = message.get_chat_id();
            let message_id = message.id;
            let user = query.from;

            let mut session = context
                .session_manager
                .get_session(SessionId::new(chat_id, 0))?;
            let mut minesweeper = session.get("minesweeper").await?.unwrap_or(Vec::new());
            let index = match minesweeper.get_index(message.reply_to.unwrap().id) {
                Ok(index) => {
                    minesweeper[index].id = message_id;
                    loop {
                        match minesweeper[index].data[cell.0 * minesweeper[index].col + cell.1] {
                            MineBoxes::Num(0) => break,
                            _ => {
                                minesweeper[index] = MineSweeper::new(
                                    message_id,
                                    minesweeper[index].row,
                                    minesweeper[index].col,
                                    minesweeper[index].mines,
                                );
                            }
                        }
                    }
                    index
                }
                Err(_) => match minesweeper.get_index(message_id) {
                    Ok(index) => index,
                    Err(_) => return Ok(HandlerResult::Stop),
                },
            };

            let edit_message: Option<EditMessageText> = None;
            match minesweeper[index].mask[cell.0 * minesweeper[index].col + cell.1] {
                MineBoxesState::Know(MineBoxes::Num(0)) => (),
                MineBoxesState::Know(MineBoxes::Num(_)) => {
                    minesweeper[index].open_num(cell.0, cell.1)
                }
                MineBoxesState::Unknow => minesweeper[index].open(cell.0, cell.1),
                _ => (),
            }
            match minesweeper[index].check() {
                MineSweeperGameState::OnGoing => {
                    session.set("minesweeper", &minesweeper).await?;
                    let edit_reply_markup = EditMessageReplyMarkup::new(chat_id, message_id)
                        .reply_markup(minesweeper[index].get_inline_keyboard());
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
                state => {
                    let edit_reply_markup = EditMessageReplyMarkup::new(chat_id, message_id)
                        .reply_markup(minesweeper[index].get_end_inline_keyboard(&state));
                    context.api.execute(edit_reply_markup).await?;
                    let method = match state {
                        MineSweeperGameState::Win => SendMessage::new(chat_id, "WIN")
                            .reply_to_message_id(minesweeper[index].id),
                        MineSweeperGameState::Fail => SendMessage::new(
                            chat_id,
                            format!(
                                "FAIL\n{} click the mine",
                                user.username.unwrap_or(user.first_name)
                            ),
                        )
                        .reply_to_message_id(minesweeper[index].id),
                        _ => SendMessage::new(chat_id, "impossable"),
                    };
                    context.api.execute(method).await?;
                    minesweeper.remove(index);
                    if minesweeper.is_empty() {
                        session.remove("minesweeper").await?;
                    } else {
                        session.set("minesweeper", &minesweeper).await?;
                    }
                }
            }
        }
    }
    Ok(HandlerResult::Continue)
}
