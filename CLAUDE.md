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

## Language

Everything in the repository is written in English: identifiers, comments, strings, UI text, docs and commit messages. The owner chats in Portuguese, but nothing in Portuguese goes into a file.

## Architecture

A modular monolith. Each module behaves like a small service that others reach only through its public API, but everything runs in one process and one SQLite file. Layers, from data to screen:

```
model <- repositories <- services <- (orchestrators) <- viewmodel <- view
 Rust       Rust          Rust          Rust             TS         TS
```

- **model**: entities with private fields, getters and validating constructors. `new` builds an unsaved entity (no id), and `restore` (module-private) rebuilds one from storage. Invariants live on the entity.
- **repositories**: CRUD only, no business rules. Each is a `Send + Sync` trait with a `Sqlite*` implementation. Rows are mapped into entities.
- **services**: business rules. Each module exposes a public `*Api` trait (for example `LanguageApi`) implemented by its service. Other code depends on the trait, never on the struct or the repositories.
- **orchestrators** (`src-tauri/src/orchestrators/`): use cases that coordinate two or more modules (`DeckOverviewOrchestrator`, `FlashcardEditorOrchestrator`). They resolve natural keys into ids. A use case that needs only one service skips this layer.
- **commands** (`src-tauri/src/commands/`): the Tauri boundary. Thin adapters over a service or orchestrator, with their own camelCase DTOs and a serializable `CommandError { kind, message }`. DTOs belong here, so the entities carry no serde. Tests pin the JSON shape the TS types rely on.
- **viewmodel** (`src/viewmodels/`): the view's API, written in TypeScript. A hook that holds state and actions and calls the typed `invoke` wrappers in `src/api/`. The Rust backend owns all logic and data, and the frontend only presents it.
- **view** (`src/routes/`, `src/components/`): renders the viewmodel and keeps only UI state such as form inputs. Components never import a viewmodel or `api`.

Rules:
- **No database ids cross the view boundary.** Commands and DTOs use natural keys: ISO 639-3 code for a language, BCP 47 code for a variant, the unique name for a deck, `slug` for a flashcard type, `key` for a field. Flashcards and drafts have no natural key, so they get an opaque `uuid` generated in Rust. Integer ids stay inside Rust.
- A module's tables are touched only by its own repositories. There are no SQL joins across modules.
- Foreign keys between modules stay in the database, so `ON DELETE CASCADE` can cross a module boundary.
- Services do not call each other's services. Cross-module checks go through an orchestrator.
- Modules: `languages`, `decks`, `card_types` (`flashcard_types`, `field_definitions`), `flashcards` (flashcards, fields, drafts, media rows) and `reviews` (`sm2_states`, `review_logs`, `card_stats`). `reviews` is used through `ReviewSessionOrchestrator` (overview with New, Learning and Review counts per type, the next card in queue order, recording an answer).

**Backend (`src-tauri/`)**
- The crate is `verbvm_lib` (`lib.rs`, with `main.rs` as a thin entry). `lib.rs` opens `verbvm.db` in the app data dir during `setup`, builds the services and orchestrators, registers them as Tauri state and lists the commands in `generate_handler!`. Custom commands need no capability entry.
- `src/dependencies/clock.rs`: everything is UTC. Timestamps are stored as `TEXT` `YYYY-MM-DD HH:MM:SS` (the format of `datetime('now')`), and services take an `Arc<dyn Clock>` instead of calling `Utc::now()`. Tests use `FixedClock`.
- `src/dependencies/db/`: `SqliteHandle` wraps a `Mutex<rusqlite::Connection>`. It creates the parent dir, enables WAL and foreign keys, and runs `rusqlite_migration` migrations on open. Access goes through `with_connection(|conn| ...)`, and multi-statement writes can use `conn.unchecked_transaction()`.
- `src/dependencies/media_store.rs` copies picked images, audio and video into `media/` in the app data dir as `<sha256>.<ext>`, validating by content (no SVG). `src/dependencies/sanitizer.rs` sanitizes rich HTML with ammonia. The Tauri asset protocol is enabled for `$APPDATA/media/**` only, and `dialog:default` is granted for the file picker.
- Migrations are `M::up(...)` entries in `db/migrations.rs`. The database tracks the applied version, so edit an existing entry only while no database has applied it. Otherwise add a new one. While the schema is still in flux, the owner keeps editing v1.0 and deletes the dev database (`~/.local/share/verbvm/verbvm.db` plus its `-wal` and `-shm` files) to re-run it. v1.0 also seeds four accents (en-US, en-GB, fr-FR, pt-BR), the three flashcard types and their fields.
- `src/dependencies/http.rs` declares `auth`, `error` and `http`; `http/http.rs` has `HttpClient`, a `reqwest` wrapper (JSON verbs plus `get_bytes` for raw bodies, rustls, no OpenSSL). `src/dependencies/speech/` has the `SpeechSynthesizer` trait (async, via `async_trait`; `synthesize(text, bcp47)` returns audio bytes) and `GoogleTranslateSynthesizer`, which uses `HttpClient` against the unofficial Google Translate TTS endpoint (about 200 characters per request, needs network). `lib.rs` builds it as `Arc<dyn SpeechSynthesizer>`, so another provider only implements the trait. `MediaStorage::import_bytes` stores generated audio the same way as a picked file.
- `src/modules/languages/` is the reference module: `model.rs`, `repository.rs`, `service.rs` and `error.rs`, plus unit tests that run against a temp SQLite file with the real migrations. Run them with `cargo test` in `src-tauri/`, or `cargo test <name>` for one.

**Frontend (`src/`)**
- `main.tsx` defines the routes with `createHashRouter` (hash routing, which suits Tauri's file-served frontend). `routes/Root.tsx` is the layout, with a responsive sidebar (`AppNavigationBar`, fed by `data/NavigationButton.ts`). The decks flow is `decks` (deck list) and `decks/:deckName` (one editor per deck, with the type buttons at the bottom adding cards of that type to a single mixed list). `Home`, `Review`, `Dashboard` and `Settings` are placeholders.
- `types/` holds the shared TS types, `api/` the typed `invoke` wrappers (`errorMessage`, the file picker, asset URLs), and `viewmodels/` the viewmodels. `routes/DeckEditor.tsx` with `viewmodels/useFlashcardEditorViewModel.ts` is the reference for the view and viewmodel split.
- Creating and editing happen in place, not in a modal. Confirmations are embedded in the view (`ConfirmDeleteButton`, with Keep and Delete buttons): never `window.confirm`, `alert` or `prompt`, which do not work in the Tauri webview. Global keyboard shortcuts go through `lib/keyboard.ts` (an observer: one subject, observers that declare their keys) with the `useKeys` hook, never `document.addEventListener("keydown")`; element-scoped `onKeyDown` is fine. Selects use the dark `Select` component, not the native `<select>`. Motion conveys state only (no page transitions or entrance sequences), uses `lib/motion.ts` for the easing and durations, and `MotionConfig reducedMotion="user"` in `Root`.
- `FlashcardInput` renders the field definitions the backend sends (front and back; rich, plain or audio-only; with media where allowed), so a new type or a language-specific field needs no component change. Type colors and icons are in `data/FlashcardTypeStyles.ts`, flags come from the offline `country-flag-icons` package.
- Styling is Tailwind v4 through the `@tailwindcss/vite` plugin, with the entry in `src/index.css`. Design tokens (surfaces `canvas`/`panel`/`raised`, text `ink`/`muted`/`subtle`, the `*-ink` brand variants for text, `accent`, `danger`, the type scale `label`/`meta`/`ui`/`body`/`lead`/`heading`/`title`) are in `@theme` there: use them, not ad hoc `white/NN` or `text-sm` values. Inter is bundled (`@fontsource-variable/inter`). Every screen is built on `Page` (a sticky breadcrumb bar, no large heading) and uses `EmptyState` when it has nothing yet.
- Internationalization uses i18next and react-i18next (`src/i18n/`), with bundled locales `en` (default and fallback), `fr` and `pt-BR` in `src/locales/*.json`. The language is chosen in Settings and kept in `localStorage`. Keys are typed from `en.json`, so add a key there first and to the other two files too. Plurals use the `_one`/`_other` (and `_many` for fr and pt) suffixes with `count`. Flashcard type names and field labels are looked up by slug and key (`useTypeLabel`, `useFieldLabel`), falling back to the backend's text for unknown ones. Backend error messages are still English text.
- In the browser (`npm run dev`) Tauri's `invoke` is unavailable, so screens that call the backend show an error. Use `npm run tauri dev`.

## Data model

One migration (v1.0) defines the schema and the seed data. Timestamps are `TEXT` UTC, and booleans are `INTEGER` 0/1.

- `languages` and `language_variants` reference each other: `languages.default_variant_id` is nullable and is set after the variant exists. The first variant created for a language becomes its default.
- `decks` have a unique name and belong to a `language_variant`, and carry study settings (`new_cards_per_day`, `max_reviews_per_day`, `autoplay_audio`; limits per UTC day), grouped as `DeckSettings` in the model so a new option does not change every signature. `review_logs.phase` (`NEW`, `LEARNING`, `REVIEW`) lets the daily limits be counted. There is no `users` table, since the app has a single profile.
- Card layouts are data, not code. `flashcard_types` (vocabulary, phonetic, orthographic) own `field_definitions`: one row per field, with `side` (`FRONT` or `BACK`), `key`, `position`, `is_required`, `media_allowed` and `kind` (`PLAIN`, `RICH` for HTML, or `AUDIO`) and `source_key` (for an `AUDIO` field, the key of the field its audio is spoken from). An audio field holds only media, uploaded or generated with text to speech. A definition with `language_id` NULL applies to every language, otherwise only to that language, and then replaces the agnostic one with the same key.
- `flashcards` hold one `flashcard_fields` row per field with content (`field_definition_id`, `content`). SQLite cannot check that a definition belongs to the card's type and language, so the editor orchestrator does.
- `flashcard_drafts` (unsaved cards, one list per deck mixing types and ordered by `position`, with `values_json`) and `flashcard_draft_media` keep work in progress. Publishing creates the flashcards, copies the media rows into `field_media` and deletes the drafts, all or nothing.
- `field_media` has `sha256`, `file_name` (`<sha256>.<ext>` in the local media folder) and `remote_key` (S3 object key, NULL until uploaded). Media lives in S3 but is read offline from the local folder, and a future sync reconciles the two.
- `stacks` only link independent flashcards for future features. `flashcards.stack_id` is nullable, and deleting a stack keeps its cards. Stack is not a flashcard type.
- Review data (`sm2_states`, `review_logs`, `card_stats`) hangs off `flashcards` with `ON DELETE CASCADE`. A flashcard with no `sm2_states` row has never been reviewed.

## Documentation

Longer documentation lives in `docs/` (start at `docs/README.md`): architecture, data model, backend, frontend, the review system and development notes. Keep it in sync when the architecture or schema changes.

## Notes

- `graphify-out/` is generated knowledge-graph output (see the global graphify skill). It is untracked and not source.
