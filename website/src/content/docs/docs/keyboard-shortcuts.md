---
title: Keyboard Shortcuts
description: Complete list of jolt keyboard shortcuts
---

jolt is fully keyboard-driven. Here's a complete reference of all shortcuts.

## Navigation

| Key               | Action              |
| ----------------- | ------------------- |
| `↑` / `k`         | Move selection up   |
| `↓` / `j`         | Move selection down |
| `PgUp` / `Ctrl+u` | Page up             |
| `PgDn` / `Ctrl+d` | Page down           |
| `Home` / `g g`    | Jump to top         |
| `End` / `G`       | Jump to bottom      |

## Process List

| Key               | Action                                      |
| ----------------- | ------------------------------------------- |
| `Enter` / `Space` | Expand/collapse process group               |
| `K`               | Kill selected process (with confirmation)   |
| `m`               | Toggle merge mode (group similar processes) |
| `o`               | Cycle sort column (PID/Name/CPU/Mem/Energy) |
| `O`               | Toggle sort direction (asc/desc)            |

## Views & Panels

| Key | Action                                                |
| --- | ----------------------------------------------------- |
| `g` | Cycle graph metric (Battery → Power → Split → Merged) |

## Appearance

| Key       | Action                                      |
| --------- | ------------------------------------------- |
| `t`       | Open theme picker                           |
| `a`       | Cycle appearance mode (Auto → Dark → Light) |
| `+` / `=` | Increase refresh rate (faster updates)      |
| `-`       | Decrease refresh rate (slower updates)      |

## Dialogs

| Key       | Action            |
| --------- | ----------------- |
| `?` / `/` | Show help dialog  |
| `s`       | Open settings     |
| `h`       | View history      |
| `b`       | Battery details   |
| `A`       | Show about dialog |

## General

| Key         | Action                 |
| ----------- | ---------------------- |
| `q` / `Esc` | Quit (or close dialog) |
| `Ctrl+c`    | Force quit             |

## Theme Picker

When the theme picker is open (press `t`):

| Key               | Action                                 |
| ----------------- | -------------------------------------- |
| `↑` / `k`         | Previous theme                         |
| `↓` / `j`         | Next theme                             |
| `Enter` / `Space` | Apply selected theme                   |
| `a` / `←` / `→`   | Toggle preview appearance (dark/light) |
| `i`               | Open theme importer                    |
| `Esc` / `t` / `q` | Close theme picker                     |

## Theme Importer

When the theme importer is open (press `i` in theme picker):

| Key             | Action                     |
| --------------- | -------------------------- |
| `↑` / `k`       | Previous theme             |
| `↓` / `j`       | Next theme                 |
| `Space`         | Toggle theme selection     |
| `Enter` / `p`   | Preview selected theme     |
| `i`             | Import selected themes     |
| `r`             | Refresh from remote        |
| `/` / `s`       | Toggle search mode         |
| `Backspace`     | Clear filter / delete char |
| `PgUp` / `PgDn` | Page navigation            |
| `Esc`           | Close importer             |

## History

When history view is open (press `h`):

| Key             | Action                                 |
| --------------- | -------------------------------------- |
| `←` / `[`       | Previous period (Today/Week/Month/All) |
| `→` / `]` / Tab | Next period                            |
| `s`             | Open settings                          |
| `Esc` / `q`     | Close history                          |

## Settings

When settings is open (press `s`):

| Key               | Action                         |
| ----------------- | ------------------------------ |
| `↑` / `k`         | Navigate settings up           |
| `↓` / `j`         | Navigate settings down         |
| `Enter` / `Space` | Toggle boolean / open selector |
| `→` / `l` / `=`   | Increment numeric values       |
| `←` / `-`         | Decrement numeric values       |
| `Esc` / `s` / `q` | Close settings                 |

## Battery Details

When battery details is open (press `b`):

| Key               | Action                |
| ----------------- | --------------------- |
| `Esc` / `b` / `q` | Close battery details |

## Kill Confirmation

When kill confirmation is shown (press `K` on a process):

| Key                 | Action                              |
| ------------------- | ----------------------------------- |
| `y` / `Y` / `Enter` | Confirm kill                        |
| `n` / `N` / `Esc`   | Cancel kill                         |
| `Tab` / `←` / `→`   | Toggle kill signal (graceful/force) |

## Vim-Style Alternatives

jolt supports vim-style navigation throughout:

| Vim Key  | Standard Key |
| -------- | ------------ |
| `j`      | `↓`          |
| `k`      | `↑`          |
| `g g`    | `Home`       |
| `G`      | `End`        |
| `Ctrl+u` | `PgUp`       |
| `Ctrl+d` | `PgDn`       |
