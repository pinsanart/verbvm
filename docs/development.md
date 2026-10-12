# Development

## Commands

| Command | Purpose |
|---|---|
| `npm run tauri dev` | The full desktop app (starts Vite, then the Rust shell) |
| `npm run dev` | The frontend alone in the browser, at http://localhost:1420 (the port is fixed, `strictPort` fails if it is taken). Tauri does not exist here |
| `npm run build` | `tsc`, then `vite build` into `dist/` |
| `npm run tauri build` | The bundled app |
| `cargo check` / `cargo build` in `src-tauri/` | Rust only |
| `cargo test` in `src-tauri/` | The unit tests. `cargo test <name>` runs one |
| `cargo doc --no-deps --document-private-items --open` in `src-tauri/` | The API documentation of the Rust code |

There is no linter and no frontend test runner.

## Development database

The schema is still being edited directly in v1.0. After changing `migrations.rs`, delete `~/.local/share/verbvm/verbvm.db` and the `verbvm.db-wal` and `verbvm.db-shm` files, and reopen the app. You lose the test data. The `media/` folder next to it is not touched, so delete it too if you want a clean slate. See [backend.md](backend.md).

## Known limitations

- **`http` does not compile.** `src-tauri/src/dependencies/http/` (a `reqwest` wrapper with `Auth::{None, Bearer}`) is switched off: `pub mod http;` is commented out in `dependencies.rs` because `reqwest` is not in `Cargo.toml`. To turn it on, add `reqwest` with the `json` feature and uncomment it.
- **Only vocabulary can be created from the UI.** Phonetic and orthographic have their schema and seed fields, but their tiles are disabled. Stack has no editor yet.
- **No review screen yet.** The `reviews` module is done but is not wired to `lib.rs`, and there is no orchestrator, command or screen for it.
- **Orphan media files.** Detaching media from a draft, or deleting a draft, leaves the file in the `media/` folder. A cleanup that deletes files no row references is not written yet.
- **No S3 sync yet.** `field_media.remote_key` is reserved for it.
- **Japanese, Chinese and Korean** use the generic vocabulary fields until language-specific definitions (for example romaji or pinyin) are added.
- **Dead-code warnings** in `cargo check` appear for methods that nothing calls yet, such as the language and deck update methods, and for the `reviews` module.
- The frontend bundle is large (about 700 KB) because it includes all the flag SVGs.
