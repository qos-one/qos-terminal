# QOS Terminal

The desktop app of QOS, the Quant Operating System: watchlist, charts, agents, positions,
orders, risk controls and the built-in Store — running on your own Mac, connected to your own
Interactive Brokers account.

The Terminal talks to the QOS engine running on the same computer
([qos](https://github.com/qos-one/qos)). Your broker login, orders and P&L stay on your
machine.

> QOS is trading software, not investment advice. Trading involves substantial risk of loss.
> Start with an IBKR **paper** account.

## Build and run

Requirements: Rust (stable), the Dioxus CLI 0.7, and the QOS engine running locally.

```bash
cargo install dioxus-cli --version 0.7.10 --locked
dx serve --platform desktop
```

By default the Terminal connects to the engine at `http://127.0.0.1:8000`. Set `BOT_API` to
use another address.

Build an app bundle:

```bash
dx bundle --platform desktop --release
```

## Layout

| Path | What it is |
|---|---|
| `src/app.rs` | App shell, routing and top bar |
| `src/ui.rs` | Screens and components |
| `src/api.rs` | Server functions that call the local QOS engine |
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
