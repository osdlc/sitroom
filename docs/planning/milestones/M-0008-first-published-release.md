---
type: milestone
schema_version: '2'
id: M-0008
state: open/draft
title: First published release
created: 2026-11-14
tags:
- release
- stability
description: The first release with published surfaces and a deprecation policy.
version: 1.0.0
---

# First published release

## Goal

The first release with published surfaces and a deprecation policy.

## Success criteria

- 1.0.0 is tagged, the binaries and the formula install, and the SDKs are on npm and PyPI.

## Deliverables

- Published: levels 1 and 2, the `View` JSON, the plugin package format and the core of the
  config file. Level 3 and the network protocol are published only if
  [M-0007](M-0007-refine.md) found no changes left to make to them.
- The deprecation policy in the [roadmap](../../roadmap.md#stability-tiers) is in effect.
- Release binaries for macOS and Linux (Windows once its CI job passes), a Homebrew formula in
  `osdlc/homebrew-tap`, and the SDKs on npm and PyPI.
