# Pulsar Reference Architecture

This document explains how technical explanations, glossary entries, search terms, and future translations are centralized.

## Goal

Keep one reference model for:

- TUI search and index
- API exposure
- future CLI explain/help commands
- future translations

The point is to avoid duplicating metric explanations in several UIs with diverging wording.

## Current Design

The shared model lives in:

```text
src/reference.rs
```

Each entry carries:

- stable `id`
- related `panel`
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
- `Esc`: close search or index

Behavior:

- the right-side reference pane shows glossary/index content
- matching monitoring panels are visually highlighted
- the same search model is reused for beginner and expert explanations

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
- consistent wording across OS and host views
- easier onboarding for beginners
- still useful detail for expert operators
- future i18n without redesigning the feature later

## What Is Still Missing

- CLI command using the shared reference catalog
- deeper per-metric inline highlighting inside tables
- more entries for Windows and macOS specifics
- more complete locale coverage beyond `fr` and `en`
