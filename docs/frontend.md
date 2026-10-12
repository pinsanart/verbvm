# Frontend

React 19 + TypeScript, built with Vite and styled with Tailwind CSS v4 (the `@tailwindcss/vite` plugin, entry in `src/index.css`). The frontend only presents: data and rules live in Rust.

## Structure

```
src/
├── main.tsx              # routes (createHashRouter)
├── routes/               # screens (the views)
├── components/           # reusable, presentational components
├── viewmodels/           # the views' API
├── api/                  # typed wrappers around Tauri's invoke
├── types/                # shared types
└── data/                 # static data: navigation buttons, flashcard type colors and icons
```

## Routes

`main.tsx` uses `createHashRouter`, hash routing that suits a frontend served as files by Tauri. `routes/Root.tsx` is the layout, with the responsive sidebar (`AppNavigationBar`, fed by `data/NavigationButton.ts`).

| Route | Screen |
|---|---|
| `/create` | `Create`: the deck list, with a dialog to create a deck |
| `/create/:deckName` | `CreateTypes`: tiles for Stack, Vocabulary, Phonetic and Orthographic in their brand colors (only vocabulary is enabled) |
| `/create/:deckName/:typeSlug` | `CreateFlashcards`: the editor of the drafts of a type in a deck |
| `/`, `/review`, `/dashboard`, `/settings` | Placeholders |

## The create flow

1. **Deck list** (`Create`, `DeckCard`, `NewDeckDialog`, `useDecksViewModel`): each deck shows its flag, name, language and accent, description and card counts by type. The dialog creates a deck from a name, a language and an accent (preselected with the language's default) and a description.
2. **Type selection** (`CreateTypes`): dotted tiles with the brand colors of `data/FlashcardTypeStyles.ts` (the theme colors in `index.css`). Stack is not a flashcard type but a way to link cards, and the other types are not built yet, so they show a "Soon" badge.
3. **Editor** (`CreateFlashcards`, `FlashcardInput`, `useFlashcardEditorViewModel`): a list of drafts. The control bar of each card moves it up or down, duplicates it or deletes it. "Add card" appends a draft and "Save all" publishes them. Drafts are stored by the backend, so closing the app loses nothing.

The editor is **driven by the field definitions** the backend sends: `FlashcardInput` renders the fields it receives, grouped under front and back, and `FieldInput` picks the rich editor (`isRich`) or plain text. A new type or a language-specific field (for example romaji for Japanese) needs no component change. Required fields are marked with an asterisk, and the errors that publishing returns are shown under the field.

### Flags

`CountryFlag` shows an SVG flag from the accent's country code, using the offline `country-flag-icons` package, or a globe when there is no country. Flag emoji are not used because Windows does not render them.

## Media preview

Fields that allow media (the phrase of a vocabulary card, the phrase of an orthographic card) have an attach button.

1. `pickMediaFiles` (`api/files.ts`) opens the system file picker for images and audio.
2. `attachMedia` sends the path to the backend, which validates the file, copies it into its media folder and attaches it to the draft.
3. The backend returns the draft with the absolute path of each file. `api/flashcards.ts` turns it into an asset protocol URL with `convertFileSrc`, so components only receive ready-to-use `Media` (`sha256`, `kind`, `mimeType`, `url`).
4. `MediaPreview` shows images as thumbnails and audio with a player, each with a remove button. A file that is missing from disk shows "File not found".

The asset protocol only serves the app's media folder (see [backend.md](backend.md)). Typing is never overwritten when media changes: the viewmodel takes only the `media` of the returned draft.

## View, viewmodel and api

- **`api/`**: one function per command, and `errorMessage(err)`, which turns what `invoke` rejected with into text (it reads the `message` of the `CommandError`).
- **`viewmodels/`**: one hook per screen. `useAsyncRunner` runs actions, captures the error and tracks `busy`. `useDecksViewModel` and `useFlashcardEditorViewModel` are the two screen hooks. The editor one keeps Rust as the source of truth, applies typing locally and saves it after a 500 ms pause, and flushes pending saves before any other action and when the screen closes.
- **Views** only render and keep UI state, such as an open dialog.
- **`types/`** mirror the DTOs of the commands. Field names come from `serde(rename_all = "camelCase")` and the explicit renames (`iso6393`).

### Adding a screen

1. Write the commands in Rust and list them in `generate_handler!`.
2. Create the types in `types/` and the wrappers in `api/`.
3. Create the viewmodel in `viewmodels/use<Name>ViewModel.ts`. It concentrates state, actions and errors.
4. Create the view in `routes/` consuming the viewmodel, and register the route in `main.tsx`. For sidebar access, add a button in `data/NavigationButton.ts`.

## Running

- `npm run tauri dev`: the full app.
- `npm run dev`: the frontend alone, at http://localhost:1420. Tauri's `invoke` does not exist in the browser, so screens that call the backend show an error. Use the full app for them.
- `npm run build`: `tsc`, then `vite build`.
