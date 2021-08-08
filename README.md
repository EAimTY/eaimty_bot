# eaimty_bot

个人用 Telegram Bot

## 功能

- [x] 没有，没有，没有，通过！ - 文字内容包含 `有没有` - 连续发送 3 次“没有”和 1 次“好，没有，通过！”
- [x] 掷飞标 - `/dart` 或文字内容包含 `飞标` - 掷一枚飞标
- [x] 掷骰子 - `/dice` 或文字内容包含 `骰子` - 掷一枚骰子
- [x] OCR - `/ocr` - 识别图片中文字（基于 Tesseract）
- [x] 黑白棋 - `/othello` - 玩黑白棋
- [x] 老虎机 - `/slot` - 转一次老虎机
- [x] Tic-Tac-Toe - `/tictactoe` - 玩 Tic-Tac-Toe
- [ ] 扫雷 - `/minesweeper` - 玩扫雷

...

## 依赖

Leptonica、Tesseract、Tesseract 语言包（eng、jpn、chi_sim、chi_tra）

## 使用

    USAGE:
        eaimty_bot [OPTIONS] --token <TOKEN>

    FLAGS:
        -h, --help       打印帮助信息
        -V, --version    打印版本信息

    OPTIONS:
        -p, --proxy <PROXY>    设置代理（支持：http、https、socks5）
        -t, --token <TOKEN>    设置 Telegram Bot HTTP API Token
        -w, --webhook <PORT>   以 webhook 模式运行，后接监听端口号

本 bot 支持 longpoll 与 webhook 两种运行方式，默认使用 longpoll

以 webhook 模式运行时，由于 Telegram 限制，webhook 地址必须为 HTTPS 协议，所以需要使用任意 web server 作为中继，以 Nginx 为例：

    server {
        listen 443 ssl http2;
        listen [::]:443 ssl http2;

        # webhook callback URL 域名
        server_name DOMAIN;

        # SSL 证书设置
        ssl_certificate      /PATH/TO/cert.pem;
        ssl_certificate_key  /PATH/TO/key.pem;
        ssl_protocols        TLSv1.1 TLSv1.2 TLSv1.3;

        # webhook callback URL 路径
        location /PATH {
            proxy_redirect off;

            # bot 监听地址与端口
            proxy_pass http://127.0.0.1:PORT;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
            proxy_set_header Host $http_host;
        }
    }

## 编译

    git clone https://github.com/EAimTY/eaimty_bot.git
    cd eaimty_bot

切换至 Rust Nightly 工具链

    rustup default nightly

编译

    cargo build --release

## 开源许可

The GNU General Public License v3.0