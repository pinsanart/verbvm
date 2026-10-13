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
│   ├── http.rs, http/      # HttpClient (reqwest wrapper), Auth, HttpError
│   └── speech.rs, speech/  # SpeechSynthesizer (text to speech) and GoogleTranslateSynthesizer
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

- **`MediaStorage`** (`LocalMediaStore`): `import(path)` validates a file **by its content**, not its extension (JPEG, PNG, GIF, WebP, MP3, OGG, WAV, FLAC, M4A, AAC, MP4, WebM and MOV, at most 50 MB, or 100 MB for video, and no SVG because it can carry scripts) and copies it to `media/<sha256>.<ext>`. Importing the same content again returns the same entry. `import_bytes(bytes)` does the same for content already in memory (generated audio), and `import` reads the file and delegates to it. `path_of(file_name)` gives the absolute path and refuses anything that is not a plain file name.
- **`HtmlSanitizer`** (`AmmoniaSanitizer`): `sanitize_html` keeps a small allow-list of formatting tags (b, i, u, br, p, ul, li and a few more) and removes scripts, event handlers and links. `strip_tags` reduces HTML to text, which the editor uses to tell whether a rich field is empty.

## Pronunciation and stacks

- **`PronunciationSource`** (`dependencies/pronunciation.rs`): `lookup(language, word)` returns the first IPA pronunciation of a word, with its slashes, for a BCP 47 code, or `None`; it fails with `NotInstalled` (no `ipa.db`) or `Unsupported(code)` (nothing for that language). `IpaDictionary` implements it over `ipa.db`, opened read-only on first use, and also has `status()` (installed languages and word counts) and `import(folder)`, which finds the `ipa-dict` `.txt` files in the folder, in `folder/data` or in `folder/*/data`, writes `ipa.db.part` (the first pronunciation of each word, without the invisible zero-width joiners the British English list has) and renames it over `ipa.db` only when it all worked (so a failed import keeps the old file). The files map to accents: `en_US` en-US, `en_UK` en-GB, `fr_FR` fr-FR, `es_ES` es-ES, `es_MX` es-MX, `de` de-DE, `pt_BR` pt-BR.
- **`pronunciation_text::transcribe(phrase, language, source)`** (`orchestrators/`, pure) splits a phrase into words (letters and digits, with `'`, `’` or `-` inside a word), looks each up once, and builds `<b>word</b>: /ipa/` lines joined with `<br>`. A word that is not listed but has an apostrophe or hyphen is tried in parts; words that are still unknown are returned in `missing`.
- **`generate_pronunciation` / `generate_saved_pronunciation`** (editor orchestrator) fill an `IPA` field from the plain text of its `source_key` field, sanitize the result and store it (a saved card keeps its other fields). Warnings name the words with no pronunciation. A missing dictionary or an unsupported language is `Unavailable`; a phrase where no word is found is `Invalid`. Generating audio now replaces the audio the field had, so the audio of an older phrase does not stay.
- **A stack** is drafts (or saved cards) that share a uuid; `flashcard_drafts.stack_uuid` for drafts and `stacks.uuid` through `flashcards.stack_id` for cards. The flashcards service makes `move_draft` and `duplicate_draft` act on the whole stack (`list_drafts` returns a stack's drafts together, at the place of the first one, and a duplicate is a new stack), while `delete_draft` and `unstack_draft` act on one draft and dissolve a stack left with fewer than two. Publishing creates the `stacks` row inside the same transaction and links the cards; `delete_flashcard` and `unstack_flashcard` dissolve a small stack the same way.
- **Orchestrator operations:** `add_stack` (vocabulary, phonetic and orthographic drafts), `add_stack_member` (a missing type, with the phrase copied as plain text), `unstack`, `delete_stack`, and async `generate_stack`, which for every card of the stack generates its audio and IPA fields from that card's own phrase, applies what works and returns what failed as warnings. For saved cards: `update_saved_many` (every card is validated with the rules of `update_saved` before any is written), `unstack_saved`, `delete_saved_stack` and `generate_saved_stack`. The view edits the text of a stack with the ordinary per-card operations, so each card can be changed on its own.

## Deck settings and daily limits

`DeckSettings` (`modules/decks/model.rs`) groups the options of a deck: `new_cards_per_day`, `max_reviews_per_day` (both 0 to 9999, validated in the constructor) and `autoplay_audio`. `Deck::new` and `revise` take it as a whole, so adding an option changes one struct and not every signature. `ReviewSessionOrchestrator::queues` applies the limits: `ReviewApi::activity_since(ids, start of the UTC day)` counts the reviews of the deck's cards by `review_logs.phase`, the new queue is cut to `new_cards_per_day` minus the cards started today, and the review queue to `max_reviews_per_day` minus the `REVIEW` answers given today. Cards being learned are never limited, so a new card already started finishes its first wave. The cards that were cut are counted, and when nothing can be shown `ReviewNext.daily_limit_reached` is true. `ReviewNext.autoplay_audio` carries the deck's audio setting to the view. Days are UTC because the whole app is.

## HTTP and text to speech

- **`HttpClient`** (`dependencies/http/http.rs`) wraps `reqwest` with a 30 s timeout. `get`, `post`, `put` and `delete` exchange JSON, `get_bytes(url, query, auth)` returns the raw body and `with_user_agent` sets the `User-Agent`. TLS is rustls, so the build needs no OpenSSL. It is async and runs on Tauri's tokio runtime.
- **`SpeechSynthesizer`** (`dependencies/speech.rs`) is an async trait: `synthesize(text, bcp47) -> SynthesizedAudio { bytes }`, with `SpeechError` (`EmptyText`, `TextTooLong`, `Unavailable`). `GoogleTranslateSynthesizer` implements it with `HttpClient` against `translate.google.com/translate_tts` (`client=tw-ob`), mapping the BCP 47 code to the `tl` parameter (`pt-BR` stays as it is, `en-US` becomes `en`). Text over 200 characters is refused, not cut. The endpoint is unofficial, see [development.md](development.md). The orchestrator tests use `FakeSpeech` (`orchestrators/fake_speech.rs`), which needs no network.

## Startup (`lib.rs`)

In `setup` the app opens `verbvm.db`, builds the services and the orchestrators and registers them as Tauri state. The `languages` service is registered as `Arc<dyn LanguageApi>` and the orchestrators as `Arc<DeckOverviewOrchestrator>` and `Arc<FlashcardEditorOrchestrator>`. The commands are listed in `generate_handler!`. The dialog plugin (`tauri-plugin-dialog`) is initialized for the file picker.

The `reviews` service is built with the system clock and used by the `ReviewSessionOrchestrator`.

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

An orchestrator holds the `*Api` traits of the modules it needs and nothing else. See [architecture.md](architecture.md) for what each one does. `OrchestratorError` (`NotFound`, `Invalid`, `Conflict`, `Unavailable`, `Internal`) is built from each module's error, from `MediaError` and from `SpeechError` (a network failure is `Unavailable`).

The editor orchestrator works on a deck's single list of drafts, which mixes flashcard types. It enforces the rules that need the card type, the language and the deck together: a draft can only hold the fields of its own type for the deck's language, rich values are sanitized when they are saved, media can only be attached to fields that allow it (at most 10 files per field), and a required field must have text or media for the drafts to be published. Audio fields (`kind` `AUDIO`) can also be **generated**: `generate_media` (drafts) and `generate_saved_media` (saved cards) read the text of the field named by the audio field's `source_key` (HTML is stripped first, and empty text is refused), speak it in the deck variant's BCP 47 code, store the bytes with `import_bytes` and attach them like an uploaded file, with the same limits. All database reads happen before the request and the attach after it, so nothing is held while waiting for the network. The text is the one stored, so the view saves pending edits first. Publishing checks every draft first, each against its own type. If any has a problem, nothing is saved and the problems are returned per draft and field.

## Commands (`src-tauri/src/commands/`)

Thin adapters over an orchestrator (or a service):

- The DTOs have `serde` and camelCase names. Entities have no serde. Nothing carries a database id.
- Errors become `CommandError { kind, message }`, where `kind` is `not_found`, `invalid`, `conflict`, `unavailable` or `internal`.
- Commands: `list_languages`, `list_language_variants`, `list_decks`, `create_deck`, `update_deck` (both carry the deck settings; `create_deck` uses the defaults for what is absent), `delete_deck`, `get_flashcard_form(deckName)` (the form of every type), `list_drafts(deckName)`, `create_draft(deckName, typeSlug)`, `update_draft`, `duplicate_draft`, `move_draft`, `delete_draft`, `attach_media`, `generate_media(draftUuid, fieldKey)`, `generate_pronunciation(draftUuid, fieldKey)`, `add_stack`, `add_stack_member`, `unstack_draft`, `delete_stack`, `generate_stack`, `remove_media`, `publish_drafts(deckName)`, for saved cards `list_saved_cards(deckName)`, `update_saved_card(cardUuid, values)`, `attach_saved_card_media`, `generate_saved_card_media(cardUuid, fieldKey)`, `generate_saved_card_pronunciation`, `update_saved_cards`, `unstack_saved_card`, `delete_saved_stack`, `generate_saved_stack`, `remove_saved_card_media` and `delete_saved_card(cardUuid)` (a saved card has the shape of a draft, keyed by field key, and obeys the same rules: known keys, sanitized rich text, required fields), `pronunciation_status` and `import_pronunciation_dictionary(folder)` (the dictionary shown and filled in Settings), and for the review `list_review_overview`, `next_review_card(deckName, typeSlug)` and `answer_review_card(cardUuid, rating, durationMs)` and `undo_review_answer(deckName, typeSlug, cardUuid)`, which takes back the latest answer given to that card and returns the same card again (each call goes one answer further back).
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
