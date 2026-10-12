# Backend

The backend is the `verbvm_lib` crate in `src-tauri/` (`lib.rs`, with `main.rs` only calling `run()`).

## Structure

```
src-tauri/src/
├── lib.rs                  # Tauri setup: opens the database, builds services and orchestrators, registers commands
├── dependencies/           # infrastructure
│   ├── clock.rs            # Clock (UTC) and the time format
│   ├── db/                 # SqliteHandle, DbError, migrations, sql_time (UTC column readers)
│   ├── media_store.rs      # MediaStorage: copies picked files into the local media folder
│   ├── sanitizer.rs        # HtmlSanitizer (ammonia)
│   └── http/               # HttpClient (NOT compiled, see development.md)
├── modules/                # one directory per module
│   ├── languages/  decks/  card_types/  flashcards/  reviews/    # model, repository, service, error
├── orchestrators/          # use cases across modules
└── commands/               # the Tauri boundary: DTOs, CommandError, commands
```

## Database and migrations

`SqliteHandle` (`dependencies/db/connection.rs`) wraps a `Mutex<rusqlite::Connection>`. On open it creates the parent directory, turns on WAL and foreign keys, sets `busy_timeout` and runs the migrations (`rusqlite_migration`). Every access goes through `with_connection(|conn| ...)`. Writes that span several tables use `conn.unchecked_transaction()` inside that call, which is the only way to share a transaction across tables.

The database file is `verbvm.db` in the app data folder. On Linux that is `~/.local/share/verbvm/verbvm.db` (the app identifier is `verbvm`). The media folder is `media/` next to it.

### Evolving the schema

The database stores in the file itself which migration version has been applied. This has two consequences:

- **In production**, never edit a migration that already ran. Append a new `M::up(...)` to the end of the list.
- **During development**, while the schema is still changing, the owner edits v1.0 directly. A development database that already applied v1.0 will not apply it again, so delete `verbvm.db` and the `verbvm.db-wal` and `verbvm.db-shm` files next to it and reopen the app.

## Time (UTC)

`dependencies/clock.rs` defines the `Clock` trait, `SystemClock` (the current UTC time, truncated to seconds) and, in tests only, `FixedClock`, which moves only when told to. It also defines `format_utc` and `parse_utc`, the only place that knows the `YYYY-MM-DD HH:MM:SS` format. `dependencies/db/sql_time.rs` reads those columns from rows. Services never call `Utc::now()`. They receive an `Arc<dyn Clock>`.

## Media store and sanitizer

- **`MediaStorage`** (`LocalMediaStore`): `import(path)` validates a file **by its content**, not its extension (JPEG, PNG, GIF, WebP, MP3, OGG, WAV, FLAC, M4A and AAC, at most 50 MB, and no SVG because it can carry scripts) and copies it to `media/<sha256>.<ext>`. Importing the same content again returns the same entry. `path_of(file_name)` gives the absolute path and refuses anything that is not a plain file name.
- **`HtmlSanitizer`** (`AmmoniaSanitizer`): `sanitize_html` keeps a small allow-list of formatting tags (b, i, u, br, p, ul, li and a few more) and removes scripts, event handlers and links. `strip_tags` reduces HTML to text, which the editor uses to tell whether a rich field is empty.

## Startup (`lib.rs`)

In `setup` the app opens `verbvm.db`, builds the services and the orchestrators and registers them as Tauri state. The `languages` service is registered as `Arc<dyn LanguageApi>` and the orchestrators as `Arc<DeckOverviewOrchestrator>` and `Arc<FlashcardEditorOrchestrator>`. The commands are listed in `generate_handler!`. The dialog plugin (`tauri-plugin-dialog`) is initialized for the file picker.

The `reviews` module is not wired into `lib.rs` yet, because its commands depend on the review screen.

## One module step by step: `languages`

`src-tauri/src/modules/languages/` is the reference.

- **`model.rs`**: `Language` and `LanguageVariant`. Private fields with getters. `new` creates an entity that is not saved yet (no id) and validates everything. `restore` (visible only inside the module) rebuilds one from the database. Rules such as `revise(...)` (replaces the editable attributes, all or nothing) and `assign_default_variant(&variant)` (the variant must be saved and belong to the language) live on the entity.
- **`repository.rs`**: the `LanguageRepository` and `LanguageVariantRepository` traits (`insert`, `find`, `find_by_iso_639_3` or `find_by_bcp47`, `find_all`, `update`, `delete`) and the `Sqlite*` implementations. `insert` uses `INSERT ... RETURNING` to return the entity with its id.
- **`service.rs`**: the `LanguageApi` trait (the module's public API) and `LanguageService`, which takes `Arc<dyn ...Repository>`. `LanguageService::sqlite(db)` builds the SQLite version. The API identifies languages by ISO 639-3 code and variants by BCP 47 code. The id-based getters exist for modules that hold a foreign key. The rules: the first variant created becomes the default, and the default must belong to the language.
- **`error.rs`**: `LanguagesError` (`NotFound`, `Invalid`, `Conflict`, `Db`). SQLite constraint violations, such as a duplicate `iso_639_3`, become `Conflict`.
- **`languages.rs`**: the module docs and the `pub use` items that make up its public surface.

The other modules follow the same layout. `flashcards` has one repository trait for all of its tables because publishing writes flashcards, fields, media rows and drafts in one transaction, and so does `reviews`.

### Adding a module

1. Create `modules/<name>.rs` (`//!` docs and `pub use`) and `modules/<name>/{model,repository,service,error}.rs`. Declare `pub mod <name>;` in `modules.rs`.
2. Add the module's tables to the v1.0 migration (see "Evolving the schema").
3. In the model, entities with private fields, a validating `new` and a `pub(super)` `restore`.
4. In the repository, a `Send + Sync` trait and the `Sqlite*` implementation. CRUD only.
5. In the service, the `*Api` trait (the only thing other modules may use) and the implementation with the rules. Time comes from a `Clock`.
6. Tests in `model.rs` and `service.rs` (see below).
7. When it needs to reach the view, write an orchestrator if the use case spans modules, then the commands, and register everything in `lib.rs`.

## Orchestrators (`src-tauri/src/orchestrators/`)

An orchestrator holds the `*Api` traits of the modules it needs and nothing else. See [architecture.md](architecture.md) for what each one does. `OrchestratorError` (`NotFound`, `Invalid`, `Conflict`, `Internal`) is built from each module's error and from `MediaError`.

The editor orchestrator works on a deck's single list of drafts, which mixes flashcard types. It enforces the rules that need the card type, the language and the deck together: a draft can only hold the fields of its own type for the deck's language, rich values are sanitized when they are saved, media can only be attached to fields that allow it (at most 10 files per field), and a required field must have text or media for the drafts to be published. Publishing checks every draft first, each against its own type. If any has a problem, nothing is saved and the problems are returned per draft and field.

## Commands (`src-tauri/src/commands/`)

Thin adapters over an orchestrator (or a service):

- The DTOs have `serde` and camelCase names. Entities have no serde. Nothing carries a database id.
- Errors become `CommandError { kind, message }`, where `kind` is `not_found`, `invalid`, `conflict` or `internal`.
- Commands: `list_languages`, `list_language_variants`, `list_decks`, `create_deck`, `update_deck`, `delete_deck`, `get_flashcard_form(deckName)` (the form of every type), `list_drafts(deckName)`, `create_draft(deckName, typeSlug)`, `update_draft`, `duplicate_draft`, `move_draft`, `delete_draft`, `attach_media`, `remove_media` and `publish_drafts(deckName)`.
- Commands are `async` and receive their state as `State<'_, Arc<...>>`.
- A draft's DTO includes, for each attached file, its absolute `path`, which the view previews through the asset protocol.
- Each file has tests that pin the JSON shape the TypeScript types rely on (`iso6393`, `bcp47Code`, `fieldKey`...).

### Media preview configuration

`tauri.conf.json` enables the asset protocol with a scope limited to `$APPDATA/media/**`, and the `protocol-asset` feature is on for `tauri` in `Cargo.toml`. The file picker needs `dialog:default` in `capabilities/default.json`. Custom commands need no capability entry.

## Tests

There is no linter. The tests are unit tests inside the modules and run with `cargo test` in `src-tauri/`. For a single one: `cargo test <name>`.

- Entity rules (`model.rs`) are tested without a database.
- Service and orchestrator tests run against a temporary SQLite file with the real migrations (so the seed data is there) and remove it at the end. Tables owned by other modules, such as `flashcards` in the `reviews` tests, are filled with plain SQL.
- Time is controlled with `FixedClock`.
- Media tests use a temporary folder and a few bytes with a PNG or MP3 signature.
