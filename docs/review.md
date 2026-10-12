# Flashcard review

The `reviews` module (`src-tauri/src/modules/reviews/`) schedules reviews with the **SM-2** algorithm and keeps the history and the statistics. Machine learning (FSRS or an LSTM) is a future step.

## Tables

All hang off `flashcards` with `ON DELETE CASCADE`, with times in UTC.

| Table | Contents |
|---|---|
| `sm2_states` | Current state per card: `due_at`, `interval_days`, `ease_factor`, `repetitions`, `last_reviewed_at`. The row is created on the first review. A card with no row has never been reviewed, and creating a card does not depend on this module |
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

## Not implemented yet

- **`ReviewSessionOrchestrator`**: asks the flashcards service for a deck's cards and the reviews service for `states_for(ids)`, and joins the due cards with the new ones, with no join across modules.
- Commands, `api/reviews.ts` and a review viewmodel and screen.

## Path to ML

The design already keeps what ML will need:

- `review_logs` holds each card's sequence of reviews, which is the training dataset. The complete history for training is meant to be kept on a server in the future. Locally it disappears with the card.
- Switching algorithms means a second state table next to `sm2_states` and, at that point, a `Scheduler` trait with both implementations. There is no trait yet because there is only one algorithm.

Implementation options (from memory, confirm versions and APIs before adopting):

- **FSRS** (the `fsrs` crate from the open-spaced-repetition project): few parameters, works from the first reviews and tends to be competitive with recurrent networks in public benchmarks.
- **LSTM**: the network reads a card's sequence of reviews (rating, log of the interval since the previous one and, optionally, type and language) and predicts the memory **stability** `s`. Retention `R(t) = 0.9^(t/s)` is then monotonic and the ideal interval comes out directly. Training minimizes binary cross-entropy (recalled or not). In Rust, Burn has `nn::Lstm`, autodiff and a CPU backend, and can train and infer inside the app. `ort` (ONNX Runtime) only infers, so training would stay in Python. Training should run on a background thread, never in the commands.
- **Little data**: a per-user LSTM needs thousands of reviews to beat a simple model. The options are to pretrain on a public dataset and fine-tune locally, or to start from FSRS.
- **Evaluation**: compare the algorithms by log-loss on a held-out set split by date.
