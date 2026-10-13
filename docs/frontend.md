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

1. **Deck list** (`Decks`, `DeckCard`, `DeckEditForm`, `NewDeckTile`, `NewDeckForm`, `useDecksViewModel`): each deck shows its flag, name, language and accent, description and card counts by type. A dashed **"+" tile after the last deck** adds a deck: it opens the form in that place (name, language, accent preselected with the language's default, and an optional description). The pencil on a deck turns its card into a form for the name, the description and the **Study options per card type**: one `DeckSettingsFields` group for each flashcard type (vocabulary, phonetic, orthographic) with new cards per day, maximum reviews per day and **Don't play audio automatically** (defaults 20, 200 and off). The create form has no study options, since a new deck uses the defaults for every type. The language and accent are fixed once a deck exists, because the fields of its cards depend on the language. Creating and editing happen in place, not in a modal.
2. **Editor** (`DeckEditor`, `FlashcardInput`, `EditorDock`, `TypePicker`, `useFlashcardEditorViewModel`): clicking a deck opens **one screen** with the deck's flashcards in a single list that mixes types. The controls are a **floating dock** at the bottom of the screen (`EditorDock`, a rounded pill over the content, not a full-width bar): pressing an enabled one adds a card of that type to the end of the list. On a phone the type buttons become a round "+" at the end of the bar (the right corner) that opens the list of types above it, where a thumb reaches, and on a wide screen each type is its own button. The bar shows only the types that can be written today (`ENABLED_TYPE_SLUGS` in `data/FlashcardTypeStyles.ts`, currently vocabulary), so every button does something. The control bar of each card moves it up or down, duplicates it or deletes it, whatever its type. The dock's save button says what it will do ("Save 3 cards") and confirms in place after saving ("3 cards saved", in green, for a few seconds). Drafts are stored by the backend, so closing the app loses nothing. A deck with no cards shows an empty state.

   The screen has two tabs. **New cards** is the editor above. **Saved cards** (`SavedCardList`, `SavedCardPanel`, `useSavedCardsViewModel`) lists what is already saved, newest first, with a search box and a filter by type. Clicking a card opens the **edit panel**: it slides in from the right on a wide screen and fills the screen on a phone, shows the same Front and Back faces as the editor (`FlashcardFaces`), and has Save changes, Close and a delete with Keep and Delete. Text is saved with an explicit Save, because it changes what the learner studies, while media and deletion apply at once. Closing with unsaved text asks first, inside the panel, and Escape closes it. Editing never touches the card's review history.

Each card has two **faces** on one shared surface, told apart by structure and not by color: the front has a solid top rule and an open eye ("Shown first, as the prompt"), the back a dashed top rule and a closed eye ("Revealed after, as the answer"). The text people write is 18px. The files of a field (`MediaPreview`) are grouped by kind under their own headings, Images as thumbnails and Audio as players, and a group appears only when the field has files of that kind. Plain and rich fields that allow media both have the paperclip at the end of their label. The labels sit close to their controls (a label row is only taller when it has buttons), and rich text, pronunciation and audio fields take the whole row on the back face instead of half of it. Rich fields are one box holding a formatting bar (`RichToolbar`), the text and, when opened, the HTML source it produces, joined by thin dividers so they read as one control. The bar offers bold, italic, underline, strikethrough, bulleted and numbered lists and clear formatting, which are the tags the backend keeps. The buttons do not take focus from the text, show the formats at the caret, and move with the arrow keys. Ctrl+B, Ctrl+I and Ctrl+U work as well.

The editor is **driven by the field definitions** the backend sends: for each draft, `FlashcardInput` renders the fields of the draft's type, grouped under front and back, and `FieldInput` picks the rich editor (`kind` `RICH`), `AudioInputField` (`kind` `AUDIO`) or plain text. A new type or a language-specific field (for example a gender field for French) needs no component change. Required fields are marked with an asterisk, and the errors that publishing returns are shown under the field.

## The review flow

1. **Overview** (`Review`, `ReviewDeckRow`, `ReviewCountList`, `useReviewOverviewViewModel`, `useCollapsedDecks`): each deck is a panel with its flag, name and accent. The header is a button that collapses and expands the deck's columns (with a height animation), and a collapsed deck shows how many cards are waiting in all (or Completed). **Collapse all** and **Expand all** appear when there is more than one deck. Which decks are collapsed is kept in `localStorage` by deck name, since it is only how the screen looks. Inside, one column per flashcard type the deck has cards of, so the columns are dynamic. A column has the type name, the New, Learning and Review counts (accent blue, red and green) and a **Review** button, or **Completed** when nothing is waiting. Columns are separated by lines instead of being cards inside a card, and on a phone they stack. A deck with no cards says so with a link to Decks.
2. **Session** (`ReviewSession`, `ReviewCardFace`, `ReviewMedia`, `RatingBar`, `ReviewSummary`, `useReviewSessionViewModel`): the type name is the title. The front of the card is large (`--text-card`, 40 to 64px) and centered, with the deck accent's `lang`, and the back appears under a divider after **Show response** (Space or Enter). The four answers (keys 1 to 4) are solid buttons with white text, filled from the `rating-*` tokens, and show how long the card would wait (the backend computes it). R plays the card's audio again. Rich fields render sanitized HTML (styled by `.rich` in `index.css`), and images and audio show under the field they belong to. The waiting counts (`ReviewCountsInline`) are in the top bar of the page on a wide screen and in a row under it on a phone, where the bar keeps only the Undo button, and they are neutral so red and green stay with the answers. The card takes the focus when the response appears, so keyboard users keep their place. The shortcuts leave focused buttons, links and inputs alone. At the end, `ReviewSummary` shows how many different cards were reviewed and how many answers were given, remembered and missed (Missed counts Again answers, so a card that came back can be counted more than once), when the next card comes back and links to the other types of the deck with cards waiting.

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

## Stacks, pronunciation and the dictionary

A **stack** is shown as one card (`StackInput`, with `StackBody` and one `StackLane` per card) in the editor, and as one panel (`SavedStackPanel`) in Saved flashcards. The type bar has a **Stack** tile that adds a vocabulary, a phonetic and an orthographic card at once.

- **The phrase band** holds the one phrase the linked cards share and the **Generate audio and pronunciation** button, which calls `generateStack` (the saved phrase of each card is used, so typed text is saved first). A status line under it lists what did not work or was left out (words with no pronunciation, offline) with a link to Settings. The new audio plays at once.
- **Each lane** is an ordinary card of its type under its own heading, with its own fields, so it can be edited alone. Its header has three buttons: link the phrase to the stack or give the card its own phrase (`Link2` / `Unlink2`, `aria-pressed`; unlinked, the lane shows its own phrase field and a "Use the stack's phrase" action), take the card out of the stack, and delete only that card (embedded confirmation). A stack with fewer than two cards stops being a stack. In the editor a missing type can be added back with "Add Orthographic".
- **Link state** is the view's: a card is unlinked when the user unlinks it, or when it comes back from the backend with a phrase different from the stack's (`lib/stackFields.ts`: `initialUnlinked`, `sharedPhrase`, `orderedMembers`, `canGenerate`). Typing in the stack's phrase updates every linked card with the ordinary per-card update.
- **An IPA field** is the rich editor with a **Generate pronunciation** sparkles button next to its label (off while the phrase is empty). The generated text replaces the field, so the card is remounted with a revision in its `key` (the rich editor does not follow `values`). The IPA text uses the `.ipa` class (font stack with fallbacks that cover the IPA Extensions block, since a browser falls back one glyph at a time), in the editor and in the review of a phonetic card. Words with no pronunciation come back as an info notice (`Notice` tone `info`).
- **Saved flashcards:** the cards of a stack are one row (the Layers badge, the phrase and the types' icons). The row opens `SavedStackPanel`, where the cards are edited together with one **Save changes** (`update_saved_cards`, all or nothing), one generate button and the same lane controls. A chevron beside the row (`aria-expanded`) shows the cards under it, and each opens alone in the ordinary card panel, where editing does not touch the others. The type filter matches a stack when any card in it does, and the search reads the phrase and the other fields. Both panels share `SidePanel` (focus, Escape, unsaved-changes prompt).
- **Settings → Import from Anki** (`AnkiImportSettings`, `useAnkiImportViewModel`): what the import does and does not bring, the **Import from Anki...** button (file picker for `collection.anki2`), a bar while it runs, then a list of what each language got (stacks and cards, images, or why it was skipped), the warnings, and a note when Anki had the file open.
- **Settings → Pronunciation dictionary** (`DictionarySettings`, `useDictionaryViewModel`): a list of the supported accents with their word counts or "Not installed", and **Import from folder...** (folder picker, an indeterminate bar and a note while it runs, then the result or the error).

## Keyboard shortcuts

Global shortcuts use the observer pattern (`lib/keyboard.ts`). A single `KeyboardSubject` listens to `keydown` on the document and tells each `KeyObserver` only about the keys it declared (`KeyboardEvent.key`, letters in either case). A key nobody declared is dropped there, without running a handler or calling `preventDefault`, and the document listener exists only while someone is subscribed. Components subscribe with `useKeys(keys, handler, enabled)` (`lib/useKeys.ts`), which always calls the latest handler, or with `keyboard().subscribe(...)` inside an effect that has other cleanup. `hasModifier` and `isTypingOrPressing` are the shared guards. Do not add `document.addEventListener("keydown")` for global keys.

The keys in use: the review listens to `r`, Space, Enter, `1` to `4`, `z` (undo) and Escape (leave the session), plus **Alt+Space** (show the response) and **Alt+R** (replay the audio), and **Ctrl+Z** (Cmd+Z), which do the same as the Undo button and, like the Alt chords, work from anywhere, including the dictation text box where the plain keys write. Ctrl+Z only takes over when there is an answer to take back, so otherwise the box undoes its own typing. On macOS those chords type " " and "®", so those are accepted too. Only Space and Enter step aside when a button or link has the focus, since those press it; the others keep working, and all of them step aside in a text field (`isTyping`). Escape also closes the type picker, the saved card panel and a pending delete confirmation. Handlers on one element (`Select`, `RichToolbar`, the deck forms, the dictation box) stay as `onKeyDown` on that element.

## Media preview

Fields that allow media (the phrase of a vocabulary card, the audio of an orthographic card) have an attach button. An audio field (`AudioInputField`) has no text box: it has the attach button, a **Generate audio** button and the files already attached. Generate is enabled when the field it is spoken from (`sourceKey`) has text, shows a spinner while the request runs, and calls `generateAudio` in the viewmodel, which saves pending edits first (drafts through `flush`, saved cards through `updateSavedCard`) because the backend speaks the stored text. The audio that was just generated plays at once (`playAudio` in `api/media.ts`; `generatedAudio` in `lib/media.ts` picks the new file), whatever the deck's autoplay setting says. Errors, such as being offline, appear in the editor's notice.

1. `pickMediaFiles` (`api/files.ts`) opens the system file picker for images, audio and video.
2. `attachMedia` sends the path to the backend, which validates the file, copies it into its media folder and attaches it to the draft.
3. The backend returns the draft with the absolute path of each file. `api/flashcards.ts` turns it into an asset protocol URL with `convertFileSrc`, so components only receive ready-to-use `Media` (`sha256`, `kind`, `mimeType`, `url`).
4. `MediaPreview` shows images as thumbnails, audio and video with a player (read through the backend, because the Linux webview cannot stream them from the asset protocol), each with a remove button. A file that is missing from disk shows "File not found".

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
