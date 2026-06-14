# histwrapped

> **Spotify Wrapped for your terminal** — see your command-line year in one screenshot.

`histwrapped` reads your shell history and turns it into stats you'll actually
want to share: your most-used commands, when you're most active, and a
screenshot-ready "wrapped" card.

> 🚧 **Early days.** zsh parsing, stats, and JSON export work today. The
> interactive TUI and the shareable card are on the way — see the
> [roadmap](#roadmap).

## Demo

<!-- TODO: drop a rendered `wrapped` card image here — it's the whole pitch. -->

```text
📊 histwrapped

  total commands : 421
  unique commands: 78

  Top programs:
     1. git                  163
     2. clear                159
     3. claude               26
     4. npm                  24
     5. cd                   21
```

## Install

```sh
# from source (crates.io release coming soon)
git clone https://github.com/swemendez/histwrapped
cd histwrapped
cargo install --path .
```

## Usage

```sh
histwrapped stats              # quick text summary of your history
histwrapped export             # dump stats as JSON
histwrapped wrapped            # shareable card (coming soon)

# options
histwrapped --top 20 stats     # show longer top-N lists
histwrapped --file ~/.zsh_history stats   # point at a specific file
histwrapped --shell bash stats            # force a parser
```

## Supported shells

| Shell | Status        | Notes                                              |
|-------|---------------|----------------------------------------------------|
| zsh   | ✅ supported  | extended + plain history; timestamps when present  |
| bash  | 🚧 planned    | with and without `HISTTIMEFORMAT`                  |
| fish  | 🚧 planned    | `fish_history` YAML-ish format                      |

> **Tip:** zsh only records timestamps when extended history is on. Add
> `setopt EXTENDED_HISTORY` to your `~/.zshrc` to unlock the time-of-day charts.

## Roadmap

- [x] zsh history parsing (extended + plain)
- [x] core stats + JSON export
- [ ] bash & fish parsers
- [ ] interactive TUI (heatmaps, scrollable lists)
- [ ] `wrapped` shareable card (PNG/SVG export)
- [ ] "terminal personality" archetypes
- [ ] publish to crates.io

## License

MIT — see [LICENSE](LICENSE).
