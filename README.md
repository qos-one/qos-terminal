# QOS Terminal

The trading desk of QOS, the Quant Operating System: watchlist, charts, heatmaps, agents,
positions, orders, risk controls and the built-in Store — connected to your own Interactive
Brokers account.

It runs in your browser, served by the QOS engine on your own server
([qos](https://github.com/qos-one/qos)). Your broker login, orders and P&L stay on that
server.

> QOS is trading software, not investment advice. Trading involves substantial risk of loss.
> Start with an IBKR **paper** account.

## Use it

Each [release](https://github.com/qos-one/qos-terminal/releases) contains the web build
(`qos-terminal-web.tgz`: WASM client and Linux server, with a SHA-256 checksum). The engine's
installer downloads and sets it up for you — see
[Running QOS on your own server](https://github.com/qos-one/qos/blob/main/docs/SELF_HOSTING.md).
Then open `https://terminal.<your-domain>` in any browser and sign in with your operator PIN.

## Build from source

Requirements: Rust (stable) and the Dioxus CLI 0.7.

```bash
cargo install dioxus-cli --version 0.7.10 --locked
dx bundle --platform web --release        # the web build the releases contain
dx serve --platform desktop               # or a desktop window, for development
```

The Terminal connects to the engine at `http://127.0.0.1:8000` by default (for a remote engine,
through an SSH tunnel). Set `BOT_API` to use another address.

On a personal machine with no operator, the Terminal sends the engine's local admin token
(`X-QOS-Local-Token`). It reads `QOS_LOCAL_TOKEN_FILE`, otherwise `QOS_DATA_DIR/local_token`,
`QOS_HOME/data/local_token`, or `~/trading/data/local_token`. The engine creates that file
(mode 0600) at install and on first start. For an engine on another computer, copy the file
and set `QOS_LOCAL_TOKEN_FILE` to the copy.

The web server listens on `127.0.0.1` and `PORT` (8080 if unset). A non-loopback `IP` is ignored;
set `QOS_TERMINAL_BIND` to listen on another address. The `Host` must be a loopback name, or a host
listed in `QOS_TRUSTED_ORIGINS` or `QOS_TERMINAL_HOST`. For the HTTPS terminal, put the site origin
in `QOS_TRUSTED_ORIGINS` (the service reads `/srv/qos/terminal.env`). `POST`, `PUT`, `PATCH`, and
`DELETE` must include the `qos_csrf` cookie set by the page and come from that same origin
(`http` on loopback, `https` on a configured public host). The desktop app sends no `Origin`;
a loopback request with the cookie and no `Origin`, `Referer`, or `Sec-Fetch-Site` is accepted.

## Layout

| Path | What it is |
|---|---|
| `src/app.rs` | App shell, routing and top bar |
| `src/ui.rs` | Screens and components |
| `src/api.rs` | Server functions that call the QOS engine API |
| `src/guard.rs` | Loopback bind and checks on inbound requests |
| `assets/` | Styles, chart code and icons |
| `demo/` | Browser build of the Terminal on simulated data (the live demo) — see `demo/README.md` |

## Third-party software

Charts use TradingView Lightweight Charts™ (Apache-2.0). See [NOTICE](NOTICE).

## Contributing and trademarks

- [CONTRIBUTING.md](CONTRIBUTING.md) — commits need a DCO sign-off
- [SECURITY.md](SECURITY.md) — report security issues privately
- [TRADEMARKS.md](TRADEMARKS.md) — "QOS", "qos.one" and the QOS logo are trademarks. Forks must
  use their own name and replace the icons in `assets/icon/` and `assets/qos-logo.png`.

## License

AGPL-3.0-only. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
