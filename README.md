# histwrapped

> **Spotify Wrapped for your terminal.** See your command-line year in one screenshot.

`histwrapped` reads your shell history and turns it into stats you'll actually
want to share: your most-used commands, when you're most active, your longest
streak, and a screenshot-ready "wrapped" card.

<p align="center">
  <img src="assets/wrapped.png" alt="A histwrapped card showing command counts, active days, streak, peak hour, and a personality badge" width="560">
</p>

## Install

```sh
# from source (crates.io release coming soon)
git clone https://github.com/swemendez/histwrapped
cd histwrapped
cargo install --path .
```

## Usage

```sh
histwrapped stats                 # quick text summary
histwrapped tui                   # interactive dashboard (q to quit)
histwrapped wrapped               # print the shareable card
histwrapped wrapped --png card.png   # save the card as an image
histwrapped wrapped --svg card.svg   # save the card as SVG
histwrapped export                # dump stats as JSON
```

Options (work on any subcommand):

```sh
histwrapped --top 20 stats                  # longer top-N lists
histwrapped --file ~/.zsh_history stats     # point at a specific file
histwrapped --shell bash stats              # force a parser
```

## What you get

- Top programs, top subcommands (`git status`, `cargo build`), and top full commands
- Total and unique command counts
- Active days, longest streak, and busiest hour (when your history has timestamps)
- A playful "terminal personality" archetype
- A shareable card as text, SVG, or PNG

## Supported shells

| Shell | Status      | Notes                                             |
|-------|-------------|---------------------------------------------------|
| zsh   | supported   | extended + plain history, multi-line commands     |
| bash  | supported   | with and without `HISTTIMEFORMAT` timestamps      |
| fish  | supported   | `fish_history` format                             |

> **Tip:** zsh only records timestamps with extended history on. Add
> `setopt EXTENDED_HISTORY` to your `~/.zshrc` to unlock the streak and
> time-of-day stats.

## Roadmap

- [x] zsh, bash, and fish parsing
- [x] core stats + JSON export
- [x] interactive TUI (top lists, hour bar chart)
- [x] `wrapped` card with SVG/PNG export
- [x] "terminal personality" archetypes
- [ ] publish to crates.io
- [ ] day-of-week heatmap

## License

MIT, see [LICENSE](LICENSE).
