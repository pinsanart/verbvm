# verbvm

A desktop flashcard app for learning languages. It works **offline**: all data lives in a local SQLite file, with no server.

> Work in progress. Today you can create decks in a language and accent, and write vocabulary flashcards with rich text and image or audio previews, with drafts that survive restarts. The SM-2 review engine exists in the backend, but there is no review screen yet.

## Main ideas

- **Offline first.** A Tauri 2 (Rust) shell around a React 19 + TypeScript frontend, with SQLite embedded.
- **Rust owns everything.** Data, rules and scheduling live in the backend. The frontend only presents.
- **Flexible card layouts.** Each flashcard type (vocabulary, phonetic, orthographic) has its fields defined as data, front and back, and a language can add fields of its own (romaji and hiragana for Japanese, for example) without changing the schema.
- **Drafts.** Cards are written as drafts that are stored by the backend, so nothing is lost when the app closes, and are published together.
- **Media.** Images and audio are copied into a local folder and previewed in the editor. They are meant to be mirrored in S3 by a future sync.
- **Spaced repetition.** SM-2 scheduling today, with the path open for FSRS or an LSTM model later.

## Technology

Tauri 2, Rust (`rusqlite`, `rusqlite_migration`, `chrono`, `ammonia`), React 19, TypeScript, Vite, Tailwind CSS v4.

## Running

Prerequisites: [Node.js](https://nodejs.org), [Rust](https://rustup.rs) and the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your system.

```bash
npm install
npm run tauri dev      # the full desktop app
```

| Command | Purpose |
|---|---|
| `npm run tauri dev` | The full app (frontend and backend) |
| `npm run dev` | The frontend alone, at http://localhost:1420 (no backend) |
| `npm run build` | Type-check and build the frontend |
| `npm run tauri build` | The bundled app |
| `cargo test` (in `src-tauri/`) | The backend tests |

On Linux the database is at `~/.local/share/verbvm/verbvm.db`. While the schema is still changing, delete that file (and the `-wal` and `-shm` files next to it) after editing the migrations.

## Structure

```
src/          # frontend: routes, components, viewmodels, api, types
src-tauri/    # Rust backend: modules, orchestrators, commands, dependencies (database, clock, media)
docs/         # documentation
```

The layered architecture, `model <- repositories <- services <- (orchestrators) <- viewmodel <- view`, is explained in [docs/architecture.md](docs/architecture.md).

## Documentation

See the [index in docs/](docs/README.md): architecture, data model, backend, frontend, the review system and the development guide.

## Recommended development environment

[VS Code](https://code.visualstudio.com/) with the [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) and [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) extensions.
