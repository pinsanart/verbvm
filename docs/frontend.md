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
| `/create/:deckName` | `CreateFlashcards`: the editor of a deck, with its flashcards of any type in one list and the type buttons at the bottom |
| `/`, `/review`, `/dashboard`, `/settings` | Placeholders |

## The create flow

1. **Deck list** (`Create`, `DeckCard`, `NewDeckDialog`, `useDecksViewModel`): each deck shows its flag, name, language and accent, description and card counts by type. The dialog creates a deck from a name, a language and an accent (preselected with the language's default) and a description.
2. **Editor** (`CreateFlashcards`, `FlashcardInput`, `TypePicker`, `useFlashcardEditorViewModel`): clicking a deck opens **one screen** with the deck's flashcards in a single list that mixes types. The **type buttons are at the bottom** of the screen: pressing an enabled one adds a card of that type to the end of the list. Stack, Phonetic and Orthographic show a "Soon" badge, and only vocabulary is enabled (`ENABLED_TYPE_SLUGS` in `data/FlashcardTypeStyles.ts`). The control bar of each card moves it up or down, duplicates it or deletes it, whatever its type. "Save all" publishes every card. Drafts are stored by the backend, so closing the app loses nothing. A deck with no cards shows an empty state.

The editor is **driven by the field definitions** the backend sends: for each draft, `FlashcardInput` renders the fields of the draft's type, grouped under front and back, and `FieldInput` picks the rich editor (`isRich`) or plain text. A new type or a language-specific field (for example a gender field for French) needs no component change. Required fields are marked with an asterisk, and the errors that publishing returns are shown under the field.

### Confirmations and controls

- **No system dialogs for confirmations.** `ConfirmDeleteButton` replaces the trash button with **Keep** and **Delete** buttons inside the view. Keep takes focus first, and Keep, Escape, clicking elsewhere and a 5 second timeout go back to the trash button. It is used for decks and flashcards. `window.confirm` does nothing in the Tauri webview, and the file picker is the only native dialog.
- **`Select`** is a dark dropdown (ARIA select-only combobox, with keyboard support) used instead of the native `<select>`, whose list the system draws and cannot be themed. `index.css` also sets `color-scheme: dark`.
- **`Button`** (primary, outline, ghost and danger), **`Notice`** (animated success and error messages) and the shared control classes in `formStyles.ts` keep controls consistent. Every interactive element has a visible focus ring.

### Typography

- **Inter** (variable, `@fontsource-variable/inter`) is bundled with the app, so the text looks the same on every machine and offline. CJK system fonts are fallbacks for the languages Inter does not cover.
- The base styles are in `index.css`: antialiasing, a 1.5 line height, headings with a tight letter spacing and balanced wrapping, and paragraphs that wrap prettily.
- Secondary text uses the `text-muted` and `text-subtle` colors (both at least 4.5:1 contrast on the panel color), not ad hoc `white/NN` opacities. Uppercase labels use `tracking-wider`, badges are 11px at the smallest, the text people write in the editors is 15px with a 1.5rem line height, and counts use tabular numbers.

### Motion

Animations use `motion/react`. `Root` wraps the app in `MotionConfig reducedMotion="user"`, so they turn off for people who ask the system for less motion, and fades between screens. Lists use `AnimatePresence` with `layout`: cards slide in, fade out and glide when one is moved or removed. Dialogs, notices, the dropdown and media previews animate in and out. The "saved" notice dismisses itself after about 4 seconds.

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
