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
| `/decks` | `Decks`: the deck list, with a "+" tile at the end to add a deck and an edit button on each deck |
| `/decks/:deckName` | `DeckEditor`: the editor of a deck, with two tabs: New cards (drafts, with the type buttons at the bottom) and Saved cards (list and edit panel) |
| `/review` | `Review`: the decks with the types each has and what is waiting in them, with a Review button per type |
| `/review/:deckName/:typeSlug` | `ReviewSession`: one card at a time, Show response, then Again, Hard, Good and Easy |
| `/`, `/dashboard`, `/settings` | Placeholders with an empty state |

## The decks flow

1. **Deck list** (`Decks`, `DeckCard`, `DeckEditForm`, `NewDeckTile`, `NewDeckForm`, `useDecksViewModel`): each deck shows its flag, name, language and accent, description and card counts by type. A dashed **"+" tile after the last deck** adds a deck: it opens the form in that place (name, language, accent preselected with the language's default, and an optional description). The pencil on a deck turns its card into a form for the name and the description. The language and accent are fixed once a deck exists, because the fields of its cards depend on the language. Creating and editing happen in place, not in a modal.
2. **Editor** (`DeckEditor`, `FlashcardInput`, `TypePicker`, `useFlashcardEditorViewModel`): clicking a deck opens **one screen** with the deck's flashcards in a single list that mixes types. The **type buttons are at the bottom** of the screen: pressing an enabled one adds a card of that type to the end of the list. The bar shows only the types that can be written today (`ENABLED_TYPE_SLUGS` in `data/FlashcardTypeStyles.ts`, currently vocabulary), so every button does something. The control bar of each card moves it up or down, duplicates it or deletes it, whatever its type. "Save all" publishes every card. Drafts are stored by the backend, so closing the app loses nothing. A deck with no cards shows an empty state.

   The screen has two tabs. **New cards** is the editor above. **Saved cards** (`SavedCardList`, `SavedCardPanel`, `useSavedCardsViewModel`) lists what is already saved, newest first, with a search box and a filter by type. Clicking a card opens the **edit panel**: it slides in from the right on a wide screen and fills the screen on a phone, shows the same Front and Back faces as the editor (`FlashcardFaces`), and has Save changes, Close and a delete with Keep and Delete. Text is saved with an explicit Save, because it changes what the learner studies, while media and deletion apply at once. Closing with unsaved text asks first, inside the panel, and Escape closes it. Editing never touches the card's review history.

Each card has two **faces**: the front is a raised surface with a solid light tag ("Shown first, as the prompt"), the back is a sunken surface with an outlined tag ("Revealed after, as the answer"), with a flip mark between them, so the difference does not depend on color. The text people write is 18px.

The editor is **driven by the field definitions** the backend sends: for each draft, `FlashcardInput` renders the fields of the draft's type, grouped under front and back, and `FieldInput` picks the rich editor (`isRich`) or plain text. A new type or a language-specific field (for example a gender field for French) needs no component change. Required fields are marked with an asterisk, and the errors that publishing returns are shown under the field.

## The review flow

1. **Overview** (`Review`, `ReviewDeckRow`, `ReviewCountList`, `useReviewOverviewViewModel`): each deck is a panel with its flag, name and accent. Inside, one column per flashcard type the deck has cards of, so the columns are dynamic. A column has the type name, the New, Learning and Review counts (accent blue, red and green) and a **Review** button, or **Completed** when nothing is waiting. Columns are separated by lines instead of being cards inside a card, and on a phone they stack. A deck with no cards says so with a link to Decks.
2. **Session** (`ReviewSession`, `ReviewCardFace`, `ReviewMedia`, `RatingBar`, `ReviewSummary`, `useReviewSessionViewModel`): the type name is the title. The front of the card is large (`--text-card`, 40 to 64px) and centered, with the deck accent's `lang`, and the back appears under a divider after **Show response** (Space or Enter). The four answers (keys 1 to 4) are quiet until hovered, take their colors from the `rating-*` tokens and show how long the card would wait (the backend computes it). R plays the card's audio again. Rich fields render sanitized HTML (styled by `.rich` in `index.css`), and images and audio show under the field they belong to. The bar at the bottom has the waiting counts, which are neutral so red and green stay with the answers. The card takes the focus when the response appears, so keyboard users keep their place. The shortcuts leave focused buttons, links and inputs alone. At the end, `ReviewSummary` shows what was answered, remembered and missed, when the next card comes back and links to the other types of the deck with cards waiting.

The viewmodel holds only the card on screen, whether its response is shown and how long the answer took. The backend keeps the queue, so after each answer the next card is asked for again.

### Confirmations and controls

- **No system dialogs for confirmations.** `ConfirmDeleteButton` replaces the trash button with **Keep** and **Delete** buttons inside the view. Keep takes focus first, and Keep, Escape, clicking elsewhere and a 5 second timeout go back to the trash button. It is used for decks and flashcards. `window.confirm` does nothing in the Tauri webview, and the file picker is the only native dialog.
- **`Select`** is a dark dropdown (ARIA select-only combobox, with keyboard support) used instead of the native `<select>`, whose list the system draws and cannot be themed. `index.css` also sets `color-scheme: dark`.
- **`Button`** (primary, outline, ghost and danger), **`Notice`** (animated success and error messages) and the shared control classes in `formStyles.ts` keep controls consistent. Every interactive element has a visible focus ring.

### Design system

The tokens live in `index.css` (Tailwind `@theme`). Use them instead of ad hoc values.

- **Surfaces**, from the page outward: `canvas` (the page), `panel` (sidebar and cards) and `raised` (menus and the front face of a card). They lean slightly cool, like the brand blue. Borders are `line` and `line-strong`.
- **Text:** `ink`, `muted` and `subtle`. Each keeps at least 4.5:1 contrast on every surface, including form fields. Placeholders use `subtle`.
- **Brand colors**, one per flashcard type, are for fills, borders and icons. As text they are too dark (the raw blue is 2.5:1 on the panel), so text uses the `*-ink` variants. `danger` is the red behind white text, and `accent` is the color of primary actions and focus. It shares the blue with the vocabulary type but is a separate token.
- **Type scale** (fixed rem, one family, Inter bundled with the app): `label` 12px, `meta` 13px, `ui` 14px, `body` 16px, `lead` 18px (what people write in the editor), `heading` 20px and `title` 24px. Light text on a dark surface gets a little more weight, tracking and leading, so the body is weight 450 with a 1.55 line height.
- **Browser-drawn parts** follow the theme: text selection, the caret, scrollbars, `accent-color`, placeholders and the focus ring (`outline` on `:focus-visible`).

### Page frame and common components

- **`Page`** is the frame of every screen: a slim sticky bar with a breadcrumb (`Decks / Travel phrases`) whose last step is the page's `h1`, set in uppercase, with optional meta (the deck's accent) and page actions at its end. On a phone the title is 20px, uppercase and centered, with a back arrow to the closest parent on one side and the actions on the other. There is no large heading, so the content starts right below. Screens that have nothing yet use **`EmptyState`**, which says what the place is for and what to do next.
- **No modals for tasks that need neither interruption nor trapped focus.** Creating and editing a deck happen in place.
- **No system dialogs for confirmations.** `ConfirmDeleteButton` replaces the trash button with **Keep** and **Delete** buttons inside the view. Keep takes focus first, and Keep, Escape, clicking elsewhere and a 5 second timeout go back to the trash button. It is used for decks and flashcards. `window.confirm` does nothing in the Tauri webview, and the file picker is the only native dialog.
- **`Select`** is a dark dropdown (ARIA select-only combobox, with keyboard support) used instead of the native `<select>`, whose list the system draws and cannot be themed.
- **`Button`** (primary, outline, ghost and danger, with a loading state), **`Notice`**, **`IconButton`** (with a real accessible name and a larger touch target) and the shared control classes in `formStyles.ts` keep controls consistent. Field labels are tied to their controls and errors are announced.
- **The sidebar rail** opens on hover and when keyboard focus is inside it.

### Motion

Motion conveys state: something appeared, moved, was removed or was pressed. It is not decoration, so there are no page transitions, entrance sequences or hover lifts. `lib/motion.ts` has the one easing (exponential ease-out) and the two durations (0.15 s and 0.22 s). Animations use `motion/react`, and `Root` wraps the app in `MotionConfig reducedMotion="user"`, so they turn off for people who ask the system for less motion. The animated things are: draft cards sliding in, fading out and gliding when one is moved or removed, the in-place forms, the confirmation swap, notices, the dropdown, media previews and the button press. The "saved" notice dismisses itself after about 4 seconds.

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
