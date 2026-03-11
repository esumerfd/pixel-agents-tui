# Pixel Agents TUI

A terminal UI that visualizes Claude Code agent activity as animated pixel-art characters in a virtual office.

![Pixel Agents TUI](docs/pixel-agents-tui.png)

## What is it?

Pixel Agents TUI watches your active Claude Code sessions and represents each agent as an animated character in a pixel-art office. Characters walk around, sit at desks when working, wander to the lounge when idle, and reflect real-time tool usage in the activity panel.

- Agents typing, reading, walking, and sitting — all animated
- Subagents (team members) appear as smaller characters with matching shirt colors
- Activity panel shows live tool status per agent (e.g. "Reading main.rs", "Running: cargo build")
- Diverse character palettes with varied skin tones
- 3-room office: Work Room (dynamic desks), Snack Bar, and Lounge
- Idle agents wander the office and sit on lounge couches

## How it works

Claude Code writes JSONL transcripts to `~/.claude/projects/`. Pixel Agents TUI polls these files, parses tool invocations, and maps agent states to character animations. No API access or configuration required — just run it alongside your Claude Code sessions.

## Install

```bash
brew tap esumerfd/pixel-agents-tui https://github.com/esumerfd/pixel-agents-tui
brew install pixel-agents-tui
```

## Usage

```bash
pixel-agents-tui
```

### Controls

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |
| `r` | Refresh agent list |
| `?` | Toggle help panel |

## Requirements

- Rust 2024 edition
- A terminal with 256-color support

## Inspiration

Inspired by [pixel-agents](https://github.com/pablodelucca/pixel-agents), a VS Code extension by Pablo De Lucca that visualizes AI agents as pixel-art characters in an animated office scene.

## License

MIT
