---
type: milestone
schema_version: '2'
id: M-0001
state: closed/done
title: Foundation
created: 2026-11-14
tags:
- ci
- stability
description: Make changes safe to land, and mark what is experimental.
version: 0.9.0
completion_note: CI gates every pull request (#20); #6 to #16 are settled (#12 to #16 merged, #6 merged as an experiment with #30); workspace lints and crate metadata (#31); every surface shows its stability tier in its doc, and catalog entries carry a stability field shown by sitroom widgets, the web view and the native app.
---

# Foundation

## Goal

Make changes safe to land, and mark what is experimental, before the larger work starts.

## Success criteria

- CI gates every pull request on Linux and macOS.
- Every surface in the roadmap's table shows its tier where it is documented.
- No pull request from #6 to #16 is open without a decision.

## Deliverables

- Rust CI: format, clippy and tests on Linux and macOS, each bundle on its own, the web build and
  the generated types, all on every pull request. Windows runs without gating. Done in #20.
- The stability tiers ([roadmap](../../roadmap.md#stability-tiers)) shown where people meet
  each surface: a note at the top of each doc, a `stability` field on catalog entries, shown by
  `sitroom widgets` and in the web catalog. Done: a note at the top of each surface's doc, and
  `stability` on catalog entries, shown by `sitroom widgets`, the web view's add dialog and the
  native app's.
- Settle the open pull requests: merge each one, or move it to an experiment. Done: #12 to #16
  are merged, and #6 (image widget and screensaver module) is merged as an experiment, with the
  fixes from its review in #30.
- The code-health part of osdlc's Rust package standard
  ([`S-package-shape-rs`](https://github.com/osdlc/osdlc/blob/main/docs/planning/standards/S-package-shape-rs.md)): workspace lints,
  a `description` and `repository` on every crate. In #31.
