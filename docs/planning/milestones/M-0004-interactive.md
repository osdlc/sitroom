---
type: milestone
schema_version: '2'
id: M-0004
state: open/draft
title: Interactive level and actions
created: 2026-11-14
related:
- '[[M-0003-api-levels]]'
tags:
- api
- actions
description: Build API level 3, and use it to act on items from their tiles.
version: 0.12.0
---

# Interactive level and actions

## Goal

Build API level 3, as experimental, and use it so a user can act on an item from its tile.

## Success criteria

- One built-in and one plugin each take an action from both views, after approval. The plugin
  is one that has been run against its real service.
- An action that fails shows its error on the tile, and the tile keeps its last good view.

## Deliverables

### Level 3

- Push mode for applets: `view/update`, which the host refuses today.
- Events sent back to the applet: a picked item, an action on it.
- Declared commands that run only after the user approves them, and the `run_command` action.

### Acting on items

- Acknowledge an incident (PagerDuty, Opsgenie), re-run a failed CI job (GitHub, Buildkite,
  CircleCI), close a todo (Todoist, todo.txt, Taskwarrior), from the TUI and the web view. The
  list mixes built-ins and plugins, so both paths are used. Several of these plugins have not
  been run against the real service yet; [M-0002](M-0002-core-and-plugins.md) may move or drop
  them first.
- Every action that changes something outside sitroom asks first, shows what it will run or
  send, and is written to the log.
- Ambient mode shows no actions (nor the screensaver module).

## Depends on

- [M-0003](M-0003-api-levels.md): level 3 is written down before it is built.
