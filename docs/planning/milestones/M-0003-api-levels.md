---
type: milestone
schema_version: '2'
id: M-0003
state: open/draft
title: API levels written down
created: 2026-11-14
related:
- '[[M-0002-core-and-plugins]]'
tags:
- api
- docs
description: One place that says what each API level is and what it promises.
version: 0.11.0
---

# API levels written down

## Goal

One place that says what each API level is and what it promises. Issue #18. The levels are listed
in the [roadmap](../../roadmap.md#api-levels).

## Success criteria

- `docs/api-levels.md` covers every level, and the contract and plugin docs link to it.
- The example widget renders the same view at every level, checked by a test.
- `plugin-check` names the level of each plugin in `plugins/ext`.

## Deliverables

- `docs/api-levels.md`: for each level, what the host guarantees (process lifecycle, timeouts,
  history, error codes), what the plugin must do, what it costs (quoted from the
  [M-0002](M-0002-core-and-plugins.md) run), and what would make it published.
- The same widget written at each level and in several languages, tested to render the same view
  (#17).
- `sitroom plugin-check` reports which level a plugin uses.
- The lists of what is not built yet point here instead, so there is one list: "Not in v0.1"
  in [contract.md](../../contract.md), and "Not yet" in [plugins.md](../../plugins.md)
  and [marketplace.md](../../marketplace.md).

## Depends on

- [M-0002](M-0002-core-and-plugins.md), for the cost of each level.
