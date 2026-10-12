# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

verbvm is an **offline-first** language-learning flashcard desktop app: all data lives locally in SQLite (no server). It is a Tauri 2 shell (Rust) around a React 19 + TypeScript frontend built with Vite and styled with Tailwind CSS v4. There is no test runner or linter configured.

## Commands

- `npm run tauri dev`: run the full desktop app (starts Vite via `beforeDevCommand`, then the Rust shell)
- `npm run dev`: frontend only in the browser at http://localhost:1420 (the port is fixed; `strictPort` fails if it is taken). Tauri APIs are unavailable here.
- `npm run build`: `tsc` type-check, then `vite build` into `dist/`
- `npm run tauri build`: bundled desktop app
- `cargo check` / `cargo build` in `src-tauri/`: Rust only

## Architecture

A modular monolith. Each module behaves like a small service that others reach only through its public API, but everything runs in one process and one SQLite file. Layers, from data to screen:

```
model <- repositories <- services <- (orchestrators) <- viewmodel <- view
 Rust       Rust          Rust          Rust             TS         TS
```

- **model**: entities with private fields, getters and validating constructors. `new` builds an unsaved entity (no id), and `restore` (module-private) rebuilds one from storage. Invariants live on the entity.
- **repositories**: CRUD only, no business rules. Each is a `Send + Sync` trait with a `Sqlite*` implementation. Rows are mapped into entities.
- **services**: business rules. Each module exposes a public `*Api` trait (for example `LanguageApi`) implemented by its service. Other code depends on the trait, never on the struct or the repositories.
- **orchestrators** (optional, `src-tauri/src/orchestrators/`, not created yet): use cases that coordinate two or more services. A use case that needs only one service skips this layer.
- **commands** (`src-tauri/src/commands/`): the Tauri boundary. Thin adapters over a service or orchestrator, with their own camelCase DTOs and a serializable `CommandError { kind, message }`. DTOs belong here, so the entities carry no serde.
- **viewmodel** (`src/viewmodels/`): the view's API, written in TypeScript. A hook that holds state and actions and calls the typed `invoke` wrappers in `src/api/`. The Rust backend owns all logic and data, and the frontend only presents it.
- **view** (`src/routes/`, `src/components/`): renders the viewmodel and keeps only UI state such as form inputs.

Module rules:
- A module's tables are touched only by its own repositories. There are no SQL joins across modules.
- Foreign keys between modules stay in the database for integrity, which also means `ON DELETE CASCADE` can cross a module boundary.
- The intended modules are `languages`, `reviews` (`sm2_states`, `review_logs`, `card_stats`), `decks`, `card_types` (`flashcard_types`, `field_definitions`) and `flashcards` (`flashcards`, `flashcard_fields`, `stacks`). `languages` and `reviews` exist so far, and only `languages` has commands and UI.
- Convention not confirmed with the owner yet: services do not call each other's services. Cross-module checks go through an orchestrator.

**Backend (`src-tauri/`)**
- The crate is `verbvm_lib` (`lib.rs`, with `main.rs` as a thin entry). `lib.rs` opens `verbvm.db` in the app data dir during `setup`, builds the services, registers them as Tauri state (for example `Arc<dyn LanguageApi>`) and lists the commands in `generate_handler!`. Custom commands need no capability entry.
- `src/dependencies/clock.rs`: everything is UTC. Timestamps are stored as `TEXT` `YYYY-MM-DD HH:MM:SS` (the format of `datetime('now')`), and services take an `Arc<dyn Clock>` instead of calling `Utc::now()`. Tests use `FixedClock`.
- `src/dependencies/db/`: `SqliteHandle` wraps a `Mutex<rusqlite::Connection>`. It creates the parent dir, enables WAL and foreign keys, and runs `rusqlite_migration` migrations on open. Access goes through `with_connection(|conn| ...)`, and multi-statement writes can use `conn.unchecked_transaction()`.
- Migrations are `M::up(...)` entries in `db/migrations.rs`. The database tracks the applied version, so edit an existing entry only while no database has applied it. Otherwise add a new one. While the schema is still in flux, the owner keeps editing v1.0 and deletes the dev database (`~/.local/share/verbvm/verbvm.db` plus its `-wal` and `-shm` files) to re-run it.
- `src/dependencies/http/` (`HttpClient`, a `reqwest` wrapper) is **not compiled**: `pub mod http;` is commented out in `dependencies.rs` because `reqwest` is not in `Cargo.toml`. Add it with the `json` feature before enabling the module.
- `src/modules/reviews/` implements SM-2 review scheduling (model, repository, service). It is not wired into `lib.rs` yet, because its commands wait for the flashcards module. The planned `ReviewSessionOrchestrator` will join it with flashcards.
- `src/modules/languages/` is the reference module: `model.rs`, `repository.rs`, `service.rs` and `error.rs`, plus unit tests in `model.rs` and `service.rs` that run against a temp SQLite file. Run them with `cargo test` in `src-tauri/`, or `cargo test <name>` for one.

**Frontend (`src/`)**
- `main.tsx` defines the routes with `createHashRouter` (hash routing, which suits Tauri's file-served frontend). `routes/Root.tsx` is the layout, with a responsive sidebar (`AppNavigationBar`, fed by `data/NavigationButton.ts`). Child routes are `Home`, `Review`, `Create` (with nested `create/:deckName` and `create/:deckName/:typeSlug`), `Dashboard` and `Settings`.
- `types/` holds the shared TS types, `api/` the typed `invoke` wrappers, and `viewmodels/` the viewmodels. `routes/CreateFlashcards.tsx` with `viewmodels/useFlashcardEditorViewModel.ts` is the reference for the view and viewmodel split.
- `Create` uses `FlashcardInput` with `PlainInputField`, `RichInputField`, `AudioField` and `MediaPreview` for card content. `types/Flashcard.ts` still uses string ids and a hard-coded type union and does not match the schema below.
- Styling is Tailwind v4 through the `@tailwindcss/vite` plugin, with the entry in `src/index.css`.
- In the browser (`npm run dev`) Tauri's `invoke` is unavailable, so the language UI shows an error. Use `npm run tauri dev`.

## Data model

One migration (v1.0) defines the schema. Timestamps are `TEXT` from `datetime('now')`, and booleans are `INTEGER` 0/1.

- `languages` and `language_variants` reference each other: `languages.default_variant_id` is nullable and is set after the variant exists. The first variant created for a language becomes its default.
- `decks` belong to a `language_variant`. There is no `users` table, since the app has a single profile.
- Card layouts are data, not code. `flashcard_types` (vocabulary, orthographic, phonetic) own `field_definitions`: one row per field, with `side` (`FRONT` or `BACK`), `key`, `position`, `is_required` and `media_allowed`. A definition with `language_id` NULL applies to every language, otherwise only to that language (for example romaji and hiragana for Japanese). A deck's fields are the type's definitions where `language_id IS NULL OR language_id = <deck language>`.
- `flashcards` hold one `flashcard_fields` row per submitted field (`field_definition_id`, `content`), unique per card and definition. SQLite cannot check that a definition belongs to the card's type and language, so the service must.
- `stacks` only link independent flashcards for future features. `flashcards.stack_id` is nullable, and deleting a stack keeps its cards.
- Review data (`sm2_states`, `review_logs`, `card_stats`) hangs off `flashcards` with `ON DELETE CASCADE`. A flashcard with no `sm2_states` row has never been reviewed.
- `field_media` is not defined yet and is planned for later.
- The seed data (card types and their default fields) is not in the migrations yet.

## Documentation

Longer documentation, in Portuguese, lives in `docs/` (start at `docs/README.md`): architecture, data model, backend, frontend, the review system and development notes. Keep it in sync when the architecture or schema changes.

## Notes

- `graphify-out/` is generated knowledge-graph output (see the global graphify skill). It is untracked and not source.
