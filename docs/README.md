# verbvm documentation

verbvm is a desktop flashcard app for learning languages. It works **offline**: all data lives in a local SQLite file, with no server. It is a Tauri 2 shell (Rust) around a React 19 + TypeScript frontend.

Rust owns all the logic and all the data. The frontend only presents it.

## Index

| Document | Contents |
|---|---|
| [architecture.md](architecture.md) | The layers, the module rules and the conventions |
| [data-model.md](data-model.md) | Every table, the ER diagram and the schema decisions |
| [backend.md](backend.md) | Database, migrations, modules, orchestrators, Tauri commands, media and tests. Includes a step-by-step guide to adding a module |
| [frontend.md](frontend.md) | Routes, viewmodels, `invoke` wrappers and the media preview. Includes a step-by-step guide to adding a screen |
| [review.md](review.md) | The review system: SM-2, tables, flow and the plan for ML |
| [development.md](development.md) | Everyday commands and known limitations |

The API documentation of the Rust code is generated with `cargo doc --no-deps --document-private-items --open` inside `src-tauri/`.
