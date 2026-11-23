---
type: milestone
schema_version: '2'
id: M-0007
state: open/draft
title: Refine
created: 2026-11-14
related:
- '[[M-0002-core-and-plugins]]'
- '[[M-0004-interactive]]'
- '[[M-0006-network]]'
tags:
- api
- stability
description: Use the API levels for real and change them before anything is published.
version: 0.15.0
---

# Refine

## Goal

Use the API levels for real and change them before anything is published.

## Success criteria

- Every plugin in `plugins/ext` uses a documented level and passes `plugin-check`.
- The list of what 1.0 publishes is in the roadmap and the CHANGELOG.

## Deliverables

- Move each plugin in `plugins/ext` to the lowest documented level that does its job (some are
  a URL and a mapping, level 1), run them for a while, and collect what should change in the API.
- Land the module moves decided in [M-0002](M-0002-core-and-plugins.md) that have not landed yet.
- Collect what the network protocol ([M-0006](M-0006-network.md)) and actions
  ([M-0004](M-0004-interactive.md)) taught about the `View` JSON and level 3.
- Pick what 1.0 publishes. Deprecate what is going away.

## Depends on

- [M-0002](M-0002-core-and-plugins.md): the module moves it lands.
- [M-0004](M-0004-interactive.md) and [M-0006](M-0006-network.md): what they taught.
