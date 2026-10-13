# Flashcard review

The `reviews` module (`src-tauri/src/modules/reviews/`) schedules reviews with the **SM-2** algorithm and keeps the history and the statistics. Machine learning (FSRS or an LSTM) is a future step.

## Tables

All hang off `flashcards` with `ON DELETE CASCADE`, with times in UTC.

| Table | Contents |
|---|---|
| `sm2_states` | Current state per card: `due_at`, `interval_days`, `ease_factor`, `repetitions`, `last_reviewed_at`, `learning_step` (reviews passed in the first wave, NULL once the card has graduated). The row is created on the first review. A card with no row has never been reviewed, and creating a card does not depend on this module |
| `review_logs` | One row per review, append-only: `reviewed_at`, `rating` (1 to 4), `elapsed_days` (null on the first), `scheduled_days`, `duration_ms` |
| `card_stats` | Per-card counters, independent of the algorithm: `review_count`, `correct_count`, `lapses`, `total_duration_ms`, `last_rating` |

A future `fsrs_states` table would sit next to `sm2_states`.

## Ratings and SM-2 rules

The ratings are `Again` (1), `Hard` (2), `Good` (3) and `Easy` (4). The ease factor starts at 2.5 and never drops below 1.3. The interval uses the ease factor from before the rating is applied.

| Rating | Repetitions | Ease factor | Next interval |
|---|---|---|---|
| Again | reset to 0 | -0.20 | 10 minutes |
| Hard | +1 | -0.15 | previous x 1.2, at least 1 day |
| Good | +1 | unchanged | 1 day, then 6 days, then previous x ease |
| Easy | +1 | +0.15 | the Good interval x 1.3 |

The rule is `Sm2State::review` in `model.rs`, and it is tested case by case.

### The first wave of a new card

A new card must be reviewed **twice** before it leaves the learning queue, unless it is answered Easy. The table above applies from the second review on:

| Answer | On a new card (first review) | On its second review |
|---|---|---|
| Again | Back to the start, 10 minutes | Back to the start, 10 minutes |
| Hard, Good | Comes back in 10 minutes, no repetition counted (Hard still lowers the ease) | Graduates by the table: 1 day |
| Easy | Graduates at once, 1.3 days | Graduates at once, 1.3 days |

So Good on a new card is not enough: it comes back and is only scheduled for a day after a second pass. The state is `learning_step` (0 on a new card, 1 after the first pass, NULL once graduated). A card that lapses later (Again after it graduated) goes back to 10 minutes and then recovers by the normal rule, without a new first wave. A card answered Again therefore keeps coming back, 10 minutes later, until it is recalled.

### Statistics

- `correct_count` counts the reviews that were not `Again`.
- A `lapse` is an `Again` on a card that had been recalled at least once. An `Again` on the first attempt is not a lapse.

## The `record_review` flow

`ReviewApi::record_review(flashcard_id, rating, duration_ms)`:

1. Reads the time from the `Clock` and loads the card's state and statistics (or creates the initial ones).
2. Computes `elapsed_days` since `last_reviewed_at`.
3. Applies the rating to the state (`Sm2State::review`) and to the statistics (`CardStats::record`).
4. Builds the `ReviewLog` with the scheduled interval.
5. `save_review` writes state, statistics and log in **one transaction**: either all three are stored or none.

Other operations: `due(limit)` (due cards, oldest first), `states_for(ids)`, `stats(flashcard_id)` and `history(flashcard_id)`. Reviewing a card that does not exist is a `Conflict`, because of the foreign key.

The three accesses of `record_review` (read state, read statistics, write) are not a single transaction. With one user that is not a problem, but two simultaneous reviews of the same card could overwrite each other.

## Undoing an answer

`ReviewApi::undo_last_review(flashcard_id)` takes back the card's latest review in one transaction: it restores the `sm2_states` and `card_stats` rows from the `previous` snapshot saved in the `review_logs` row (deleting a row that did not exist before, so a card whose first review is undone is new again) and deletes that log row. Because the daily limits are counted from the log, undoing also gives the limit back. Each call goes one review further back for that card. `ReviewSessionOrchestrator::undo_answer` calls it and returns that same card, ready to be answered again, with the counts as they are now.

## The review screens

`/review` is the overview and `/review/:deckName/:typeSlug` reviews one type of one deck, card by card. Both talk to the `ReviewSessionOrchestrator`, which joins decks, flashcards, card types and the `reviews` module (the `reviews` module itself does not know about decks).

### Orthographic cards (dictation)

An orthographic card has no back: its audio field (generated or uploaded) and its phrase are both on the front, which is how it is written in the editor. The review shows only the audio and keeps the phrase hidden. The backend sends each review field with its `kind` and `source_key`, so `ReviewSession` takes the audio fields to show and the field named by their `source_key` as the expected answer. It branches on the type slug and renders `DictationReview`: the audio (the Replay button, or **Alt+R**, since R types a letter in the box), a text box that takes the focus and submits with Enter, **Alt+Space** or the **Show response** button, and then a sentence that says how the answer compares ("Exact match", "1 letter missing", "1 extra and 2 missing letters", "Nothing typed") above what was typed and what was expected, on two lines that share a label column, letter by letter (`lib/textDiff.ts`, a longest common subsequence, which also counts the extra and the missing letters). Correct letters are green, letters that were typed but are wrong are red and struck through, and expected letters that were missing are gray and underlined, so color is not the only signal. The wrong, missing and right backgrounds all keep at least 4.5:1 against the text. The comparison is strict (case, accents and punctuation count) after trimming. The learner still picks the rating, so the review log is unchanged and the typed text is not stored. The viewmodel keeps `typed` and clears it with each new card.

### Daily limits and audio

Each deck has `new_cards_per_day` (20 by default) and `max_reviews_per_day` (200), set when the deck is created and editable later, and `autoplay_audio`. A **new** card counts against the first limit when it is reviewed for the first time. A **review** counts against the second when a card that had been recalled before is answered. Both are counted per UTC day from `review_logs.phase`, and they cut the New and Review counts and queues: when a limit is reached the counts drop to what is still allowed and the session ends with a note that the rest comes back tomorrow. Cards being learned (Learning) are never limited, so a card already started always finishes its first wave. A limit of 0 offers none of that kind. The audio of a card plays by itself **every time the card is shown** (the first audio of the front, and of the back when the response appears), including a card that comes back at once after Again or its first pass: each showing is a new presentation (`presentation` in the viewmodel, part of the card element's key) and `MediaPlayer` starts playback itself once its file is loaded. It does not play when the deck turns it off with **Don't play audio automatically**; R still replays it.

### Counts and the queue

For one type in a deck:

| Count | Which flashcards |
|---|---|
| **New** | Never reviewed (no `sm2_states` row). Every one is counted, since there is no daily limit yet |
| **Learning** | Due now (`due_at <= now`) and not recalled yet (`repetitions == 0`): the last answer was Again, or it is a new card waiting for its second review |
| **Review** | Due now and recalled before (`repetitions >= 1`) |

A flashcard that is not due yet is in none of them. A type is **Completed** when all three are 0. The overview only lists the types a deck has flashcards of, so its columns change with the deck.

A session is stateless. After every answer the next card is asked for again: **learning** first (oldest due first), then **review** (oldest due first), then **new** (in creation order). That is why a card answered Again comes back by itself, with no queue kept in the view.

**Learn ahead.** A card still being learned (Again, or the first pass of a new card) is only due in 10 minutes, which would end the session with the card still unlearned. So when nothing else is due, cards in learning that come back within 20 minutes (`LEARN_AHEAD`) are brought forward and shown right away, and a card keeps coming back until it is recalled. Cards that are due, then new ones, always go first, so a missed card is not repeated while others are waiting. The overview counts still include only what is due, and the Learning count of a session includes the brought-forward cards while they are the only ones left.

### The screen

The type name is the title. The front of the card is set large (40 to 64px) and centered, with the `lang` of the deck's accent, so scripts such as Japanese get the right spacing. The waiting counts (New, Learning, Review) are in the top bar, with the count of the card on screen highlighted and the meaning of the three shown on hover and on keyboard focus. **Show response** (Space or Enter) reveals the back under a divider, and the four answers appear as solid buttons in their colors: **Again, Hard, Good, Easy** (keys 1 to 4). Each shows how long the card would wait for it (for example 10 min, 1 day, 6 days), computed by the backend with the SM-2 rule without recording anything. Each is recorded through `ReviewApi::record_review` with how long it took. Rich fields are sanitized HTML, and the media of a field (images and audio) shows under it. **R** plays the card's audio again, and each audio field has a visible **Replay** button that does the same for its own files. **Ctrl+Z** (also from the dictation box, as long as there is an answer to take back), **Z** or the Undo button in the top bar takes back the last answer and shows that card again as it was, as many times as needed; **Escape** leaves the session (answers are saved as they are given). If an answer was saved but the next card could not be loaded, the bar offers **Try again** instead of the answers, so the same card cannot be answered twice. When the queue is empty, a summary shows how many different cards were reviewed, how many answers there were, and how many of them were remembered and missed (a missed card comes back until it is recalled, so a card can be missed more than once and **Missed** counts the answers, not the cards), when the next card comes back, and links to the other types of the deck that still have cards waiting. Editing or deleting a saved flashcard from a review is not built yet, so there is no Actions menu.

## Path to ML

The design already keeps what ML will need:

- `review_logs` holds each card's sequence of reviews, which is the training dataset. The complete history for training is meant to be kept on a server in the future. Locally it disappears with the card.
- Switching algorithms means a second state table next to `sm2_states` and, at that point, a `Scheduler` trait with both implementations. There is no trait yet because there is only one algorithm.

Implementation options (from memory, confirm versions and APIs before adopting):

- **FSRS** (the `fsrs` crate from the open-spaced-repetition project): few parameters, works from the first reviews and tends to be competitive with recurrent networks in public benchmarks.
- **LSTM**: the network reads a card's sequence of reviews (rating, log of the interval since the previous one and, optionally, type and language) and predicts the memory **stability** `s`. Retention `R(t) = 0.9^(t/s)` is then monotonic and the ideal interval comes out directly. Training minimizes binary cross-entropy (recalled or not). In Rust, Burn has `nn::Lstm`, autodiff and a CPU backend, and can train and infer inside the app. `ort` (ONNX Runtime) only infers, so training would stay in Python. Training should run on a background thread, never in the commands.
- **Little data**: a per-user LSTM needs thousands of reviews to beat a simple model. The options are to pretrain on a public dataset and fine-tune locally, or to start from FSRS.
- **Evaluation**: compare the algorithms by log-loss on a held-out set split by date.
