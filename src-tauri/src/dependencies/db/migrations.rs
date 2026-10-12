use rusqlite_migration::{Migrations, M};

pub fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        // v1.0: languages, decks, flashcards, stacks and reviews
        M::up(
            r#"
            CREATE TABLE languages (
                id                 INTEGER PRIMARY KEY,
                iso_639_1          TEXT CHECK (iso_639_1 IS NULL OR length(iso_639_1) = 2),
                iso_639_3          TEXT NOT NULL UNIQUE CHECK (length(iso_639_3) = 3),
                name               TEXT NOT NULL,
                
                -- Circular with language_variants.language_id: set after the variant exists.
                default_variant_id INTEGER REFERENCES language_variants(id) ON DELETE SET NULL
            );

            CREATE TABLE language_variants (
                id           INTEGER PRIMARY KEY,
                language_id  INTEGER NOT NULL REFERENCES languages(id) ON DELETE CASCADE,
                bcp47_code   TEXT NOT NULL UNIQUE CHECK (length(bcp47_code) <= 10),
                variant_name TEXT NOT NULL,
                country_code TEXT CHECK (country_code IS NULL OR length(country_code) = 2)
            );
            
            CREATE INDEX idx_language_variants_language_id ON language_variants(language_id);

            CREATE TABLE decks (
                id                  INTEGER PRIMARY KEY,
                name                TEXT NOT NULL,
                language_variant_id INTEGER NOT NULL REFERENCES language_variants(id) ON DELETE RESTRICT,
                description         TEXT,
                created_at          TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX idx_decks_language_variant_id ON decks(language_variant_id);

            CREATE TABLE flashcard_types (
                id          INTEGER PRIMARY KEY,
                slug        TEXT NOT NULL UNIQUE,
                name        TEXT NOT NULL,
                description TEXT
            );

            -- What a card type can hold. language_id NULL = applies to every language,
            -- otherwise only to decks of that language (e.g. romaji/hiragana for Japanese).
            CREATE TABLE field_definitions (
                id               INTEGER PRIMARY KEY,
                flashcard_type_id INTEGER NOT NULL REFERENCES flashcard_types(id) ON DELETE RESTRICT,
                language_id      INTEGER REFERENCES languages(id) ON DELETE RESTRICT,
                side             TEXT NOT NULL CHECK (side IN ('FRONT', 'BACK')),
                key              TEXT NOT NULL,
                label            TEXT NOT NULL,
                position         INTEGER NOT NULL DEFAULT 0,
                is_required      INTEGER NOT NULL DEFAULT 0 CHECK (is_required IN (0, 1)),
                media_allowed    INTEGER NOT NULL DEFAULT 0 CHECK (media_allowed IN (0, 1))
            );
            CREATE UNIQUE INDEX idx_field_definitions_unique
                ON field_definitions(flashcard_type_id, COALESCE(language_id, 0), key);

            -- A link between independent flashcards, for features that relate them.
            CREATE TABLE stacks (
                id         INTEGER PRIMARY KEY,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE flashcards (
                id                INTEGER PRIMARY KEY,
                deck_id           INTEGER NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
                flashcard_type_id INTEGER NOT NULL REFERENCES flashcard_types(id) ON DELETE RESTRICT,
                -- A flashcard belongs to at most one stack. Deleting a stack keeps its flashcards.
                stack_id          INTEGER REFERENCES stacks(id) ON DELETE SET NULL,
                created_at        TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at        TEXT
            );
            CREATE INDEX idx_flashcards_deck_id ON flashcards(deck_id);
            CREATE INDEX idx_flashcards_flashcard_type_id ON flashcards(flashcard_type_id);
            CREATE INDEX idx_flashcards_stack_id ON flashcards(stack_id);

            -- Side and media_allowed come from the field definition.
            CREATE TABLE flashcard_fields (
                id                  INTEGER PRIMARY KEY,
                flashcard_id        INTEGER NOT NULL REFERENCES flashcards(id) ON DELETE CASCADE,
                field_definition_id INTEGER NOT NULL REFERENCES field_definitions(id) ON DELETE RESTRICT,
                content             TEXT,
                UNIQUE (flashcard_id, field_definition_id)
            );
            CREATE INDEX idx_flashcard_fields_field_definition_id ON flashcard_fields(field_definition_id);

            -- All timestamps are UTC text in the 'YYYY-MM-DD HH:MM:SS' format of datetime('now').

            -- SM-2 scheduling state. A flashcard with no row has never been reviewed.
            CREATE TABLE sm2_states (
                flashcard_id     INTEGER PRIMARY KEY REFERENCES flashcards(id) ON DELETE CASCADE,
                due_at           TEXT NOT NULL,
                interval_days    REAL NOT NULL,
                ease_factor      REAL NOT NULL,
                repetitions      INTEGER NOT NULL,
                last_reviewed_at TEXT
            );
            CREATE INDEX idx_sm2_states_due_at ON sm2_states(due_at);

            -- Append-only review history.
            CREATE TABLE review_logs (
                id             INTEGER PRIMARY KEY,
                flashcard_id   INTEGER NOT NULL REFERENCES flashcards(id) ON DELETE CASCADE,
                reviewed_at    TEXT NOT NULL,
                rating         INTEGER NOT NULL CHECK (rating BETWEEN 1 AND 4),
                elapsed_days   REAL,
                scheduled_days REAL NOT NULL,
                duration_ms    INTEGER
            );
            CREATE INDEX idx_review_logs_flashcard_reviewed_at ON review_logs(flashcard_id, reviewed_at);

            -- Per-flashcard counters, independent of the scheduling algorithm.
            CREATE TABLE card_stats (
                flashcard_id      INTEGER PRIMARY KEY REFERENCES flashcards(id) ON DELETE CASCADE,
                review_count      INTEGER NOT NULL DEFAULT 0,
                correct_count     INTEGER NOT NULL DEFAULT 0,
                lapses            INTEGER NOT NULL DEFAULT 0,
                total_duration_ms INTEGER NOT NULL DEFAULT 0,
                last_rating       INTEGER CHECK (last_rating IS NULL OR last_rating BETWEEN 1 AND 4)
            );
            "#,
        ),
    ])
}
