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

- **Text to speech uses an unofficial endpoint.** `GoogleTranslateSynthesizer` calls the Google Translate TTS URL, which has no key, is undocumented and can rate limit or change without notice. It needs network, takes at most about 200 characters (longer text is refused) and sends a browser `User-Agent`. Generating audio fails with an `unavailable` error when offline, while reviewing never needs the network because the audio is stored locally. To use another provider, implement `SpeechSynthesizer` (`dependencies/speech.rs`) and build it in `lib.rs`.
- **The pronunciation dictionary is not in the repository.** The phonetic card and the stack get their IPA from `ipa.db`, built from the [`ipa-dict`](https://github.com/open-dict-data/ipa-dict) word lists (`en_US`, `en_UK`, `fr_FR`, `es_ES`, `es_MX`, `de` and `pt_BR`; about 60 MB of text, 2.5 million words). In the app, open Settings, press **Import from folder...** and pick the folder that holds the lists (the unpacked project, its `data` folder, or the folder that contains it); it takes about 15 seconds and `ipa.db` takes about 110 MB in the app data folder. Until it is imported, the pronunciation buttons say the dictionary is not installed. Only the first pronunciation of a word is kept, a word is looked up as written and then in lower case, and a word with an apostrophe or a hyphen that is not listed is looked up by its parts (`l'homme` is `l'` and `homme`). Italian is not supported yet: the lists have none. The lists have different licenses (the British English list derives from GPL-3 data, the German one is CC BY-SA, the rest are MIT or generated): read the credits in the project's README before distributing the app.
- **`ENABLED_TYPE_SLUGS` (`data/FlashcardTypeStyles.ts`) lists what the editor offers.** It now has the stack and the three card types, and a type added to the database stays hidden until it is listed there: `ENABLED_TYPE_SLUGS` (`data/FlashcardTypeStyles.ts`) lists only the types that can be written, and the bar shows only those. Stack has no editor yet.
- **Only a few accents are seeded** (en-US, en-GB, fr-FR, pt-BR, es-ES, es-MX, de-DE), and there is no screen to add more.
- **A review cannot edit or delete the card.** Saved cards are edited and deleted from the deck's Saved cards tab, but the review still has no Actions menu to jump there.
- **Daily limits count per UTC day.** A deck's new-card and review limits reset at midnight UTC, which is the evening in the Americas, because the whole app works in UTC. A local day would need the time zone to be stored.
- **`Couldn't get key from code: ...` in the terminal.** In `npm run tauri dev`, pressing a key `tao` cannot map (the volume keys, for example) prints this line from the native layer. It is an `eprintln!` in `tao` for debug builds only, emitted when the window gets the key and before the webview sees it, so the view cannot silence it and a release build never prints it. The view itself ignores such keys: see the keyboard section in [frontend.md](frontend.md).
- **Orphan media files.** Detaching media from a draft, or deleting a draft, leaves the file in the `media/` folder. A cleanup that deletes files no row references is not written yet.
- **No S3 sync yet.** `field_media.remote_key` is reserved for it.
- **Japanese, Chinese and Korean** use the generic vocabulary fields until language-specific definitions (for example romaji or pinyin) are added.
- **Dead-code warnings** in `cargo check` appear for methods that nothing calls yet, such as the language and deck update methods, and for the `reviews` module.
- The frontend bundle is large (about 700 KB) because it includes all the flag SVGs.
