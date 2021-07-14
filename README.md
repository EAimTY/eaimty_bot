# eaimty_bot

个人用 Telegram Bot，使用 [Rust](https://www.rust-lang.org/) 编写，基于 [carapax](https://github.com/tg-rs/carapax)

## 使用

    USAGE:
        eaimty_bot [OPTIONS] --token <TOKEN>

    FLAGS:
        -h, --help       打印帮助信息
        -V, --version    打印版本信息

    OPTIONS:
        -p, --proxy <PROXY>    设置代理（支持：http、https、socks5）
        -t, --token <TOKEN>    设置 Telegram Bot Token

## 功能

- [x] 掷骰子（`/dice` 或文字内容包含 `骰子`）
- [x] 掷飞标（`/dart` 或文字内容包含 `飞标`）
- [x] 没有，没有，没有，通过！（文字内容包含 `有没有`）
- [ ] 扫雷（`/minesweeper`）
- [ ] OCR（`/ocr`）
...

## 编译

    git clone https://github.com/EAimTY/eaimty_bot.git
    cd eaimty_bot

切换至 Rust Nightly 工具链

    rustup default nightly

编译

    cargo build --release

## 开源许可

The GNU General Public License v3.0