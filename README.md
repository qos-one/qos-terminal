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

## Layout

| Path | What it is |
|---|---|
| `src/app.rs` | App shell, routing and top bar |
| `src/ui.rs` | Screens and components |
| `src/api.rs` | Server functions that call the QOS engine API |
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
