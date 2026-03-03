# Pulsar Reference Architecture

This document explains how technical explanations, glossary entries, search terms, and future translations are centralized.

## Goal

Keep one reference model for:

- TUI search and index
- API exposure
- future CLI explain/help commands
- future translations

The point is to avoid duplicating metric explanations in several UIs with diverging wording.
The catalog must be wider than the current TUI and include implemented, partial, and planned metrics.

## Current Design

The shared model lives in:

```text
src/reference.rs
```

Each entry carries:

- stable `id`
- `category`
- related `panel`
- delivery `status`
- UI presence: `visible` or `indexed_only`
- `aliases`
- `tags`
- audience level: `beginner` or `expert`
- localized text blocks

Today the structure is wired for:

- `fr`
- `en`

That is enough to prove the design. Additional locales can be added without changing TUI or API behavior.

## TUI Usage

Current shortcuts:

- `/`: open search input
- `?`: toggle reference index
- `i`: switch the TUI locale (`fr` / `en`)
- `Esc`: close search or index

Behavior:

- the right-side reference pane shows glossary/index content
- the index language follows the active TUI locale
- matching monitoring panels are visually highlighted
- the same search model is reused for beginner and expert explanations
- entries not yet rendered in the current TUI can still exist as `indexed_only`

## API Usage

Current endpoint:

```text
/reference
```

Examples:

```text
/reference
/reference?lang=fr
/reference?q=latency&lang=en
```

This keeps the knowledge base accessible outside the TUI.

## Scaling To More Languages

The intended path is:

1. keep stable entry IDs
2. add localized text blocks per entry
3. keep aliases/tags per language where needed
4. reuse the same reference catalog from every UI surface

Recommended next locales only if quality is maintainable:

- French
- English
- Spanish
- German
- Italian or Portuguese after review

## What This Solves

- one source of truth for metric explanations
- one source of truth for the metric inventory itself
- consistent wording across OS and host views
- easier onboarding for beginners
- still useful detail for expert operators
- future i18n without redesigning the feature later

## What Is Still Missing

- deeper per-metric inline highlighting inside tables
- more entries for Windows and macOS specifics
- stricter generation or validation against `docs/metrics-checklist.md`
- more complete locale coverage beyond `fr` and `en`
