# Architecture

The system is a **modular monolith**. Each module behaves like a small service that others reach only through its public API, but everything runs in one process and one SQLite file.

## Layers

```
model <- repositories <- services <- (orchestrators) <- viewmodel <- view
 Rust       Rust          Rust          Rust             TS         TS
```

| Layer | Where | Responsibility | Does not |
|---|---|---|---|
| model | `src-tauri/src/modules/<module>/model.rs` | Entities with private fields, getters and validating constructors. Invariants and rules that depend only on the entity live here | Know about the database or serde |
| repositories | `.../repository.rs` | CRUD only. A `Send + Sync` trait with a `Sqlite*` implementation that maps rows into entities | Hold business rules |
| services | `.../service.rs` | Business rules. They implement the module's public `*Api` trait | Know about Tauri or DTOs |
| orchestrators | `src-tauri/src/orchestrators/` | Use cases that coordinate two or more modules. They resolve natural keys into ids | Touch repositories |
| commands | `src-tauri/src/commands/` | The Tauri boundary. Thin adapters over a service or an orchestrator, with their own camelCase DTOs and `CommandError { kind, message }` | Hold business rules |
| viewmodel | `src/viewmodels/` | The view's API, in TypeScript: state, actions and error handling. It calls the wrappers in `src/api/` | Render |
| view | `src/routes/`, `src/components/` | Renders the viewmodel and keeps only UI state, such as the text of a form | Call `invoke` |

DTOs live in the commands layer so that the shape the view needs never leaks into the entities.

## No database ids across the boundary

The view and the viewmodel talk to the backend like an API, so no database id crosses that boundary. Things are identified by natural keys:

| Thing | Identified by |
|---|---|
| Language | ISO 639-3 code (`iso6393`) |
| Language variant (accent) | BCP 47 code (`bcp47Code`) |
| Deck | its unique `name` |
| Flashcard type | `slug` |
| Field | `key` |
| Flashcard, draft | an opaque `uuid` generated in Rust (they have no natural key, and it is also the future sync key) |

Integer ids stay inside Rust. Orchestrators resolve natural keys into ids, and modules hold each other's ids only as foreign keys.

## Module rules

- A module's tables are touched only by its own repositories. There are no SQL joins across modules.
- A module is reached only through its `*Api` trait, never through the service struct or the repositories.
- Foreign keys between modules stay in the database for integrity, which also means `ON DELETE CASCADE` can cross a module boundary (deleting a deck deletes its flashcards and drafts).
- Services do not call the services of other modules. A check that crosses modules, such as "this accent belongs to that language", goes into an orchestrator, which asks one service and then calls the other.
- A use case that needs only one service skips the orchestrator and the command calls the service directly.
- Every time is UTC, and services take a `Clock` instead of calling `Utc::now()`.

## Modules

| Module | Tables | Status |
|---|---|---|
| `languages` | `languages`, `language_variants` | Implemented. Seeded with four accents (en-US, en-GB, fr-FR, pt-BR). Only the lists are exposed to the view |
| `decks` | `decks` | Implemented |
| `card_types` | `flashcard_types`, `field_definitions` | Implemented. Seeded with vocabulary, phonetic and orthographic |
| `flashcards` | `flashcards`, `flashcard_fields`, `flashcard_drafts`, `flashcard_draft_media`, `field_media` | Implemented: flashcards, drafts, media |
| `reviews` | `sm2_states`, `review_logs`, `card_stats` | Implemented (model, repository, service). No commands yet |

Dependencies, always one way and without cycles:

```
languages <- decks <- flashcards -> card_types -> languages
                          ^
                       reviews (linked to flashcards by id only, in an orchestrator)
```

Orchestrators:

| Orchestrator | Coordinates | Use cases |
|---|---|---|
| `DeckOverviewOrchestrator` | decks, languages, card types, flashcards | List decks with language, accent and card counts by type. Create a deck from a language and an optional accent. Delete a deck |
| `FlashcardEditorOrchestrator` | decks, languages, card types, flashcards, sanitizer, media store | The form of every type in a deck. Add a draft of a type, edit, move, duplicate and delete drafts in the deck's single mixed list. Attach and remove media. Publish all the drafts, each against the fields of its own type |

## How the view logic is organized

Dependencies point one way: `routes (views) -> viewmodels -> api -> Tauri invoke`. Components never import a viewmodel or `api`.

- **api** (`src/api/`): one typed function per command. No state.
- **viewmodel** (`src/viewmodels/`): one hook per screen. It owns the state (a loading status, the data, per-item errors), the async actions and the derived data. It receives route params as arguments and imports no JSX and no router. `useAsyncRunner` wraps every action with error capture and a `busy` flag.
- **views** (`src/routes/`): compose components from the viewmodel's object and keep only UI state.
- **No system dialogs for confirmations.** Deleting asks inside the view (`ConfirmDeleteButton`, with Keep and Delete buttons), never with `window.confirm`, which does nothing in the Tauri webview. The only native dialog is the file picker, which is needed to choose a file.
- **components** (`src/components/`): presentational. Data and callbacks come in through props, so a new flashcard type or field needs no component change.

The editor viewmodel keeps Rust as the source of truth. Add, move, duplicate and delete replace the list with the one the backend returns. Typing is applied locally right away and saved after a short pause, and pending saves are flushed before any other action and when the screen closes. Required fields, key checks and sanitizing are enforced in Rust. The viewmodel only shows what the backend reports.
