use rusqlite_migration::{Migrations, M};

pub fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        // v1.0: languages, decks, flashcards, drafts, media, stacks, reviews and seed data
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
                name                TEXT NOT NULL UNIQUE,
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

            -- The study options of a deck, one row per flashcard type of the deck (like subdecks). A type with no row uses the defaults. Daily limits are counted per UTC day: a new
            -- card is one seen for the first time, a review is an answer to a card that had been
            -- recalled before. `autoplay_audio` is whether the audio plays by itself in a review. The time
            -- the study day starts at is added by v1.1.
            CREATE TABLE deck_type_settings (
                deck_id             INTEGER NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
                flashcard_type_id   INTEGER NOT NULL REFERENCES flashcard_types(id) ON DELETE RESTRICT,
                new_cards_per_day   INTEGER NOT NULL DEFAULT 20  CHECK (new_cards_per_day BETWEEN 0 AND 9999),
                max_reviews_per_day INTEGER NOT NULL DEFAULT 200 CHECK (max_reviews_per_day BETWEEN 0 AND 9999),
                autoplay_audio      INTEGER NOT NULL DEFAULT 1 CHECK (autoplay_audio IN (0, 1)),
                PRIMARY KEY (deck_id, flashcard_type_id)
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
                media_allowed    INTEGER NOT NULL DEFAULT 0 CHECK (media_allowed IN (0, 1)),
                -- PLAIN fields hold text, RICH fields hold HTML (rich editor), AUDIO fields hold only
                -- audio, uploaded or spoken from the text of the field named by `source_key`, and IPA
                -- fields hold HTML with the pronunciation of the words of the `source_key` field.
                kind             TEXT NOT NULL DEFAULT 'PLAIN' CHECK (kind IN ('PLAIN', 'RICH', 'AUDIO', 'IPA')),
                source_key       TEXT,
                CHECK ((kind IN ('AUDIO', 'IPA')) = (source_key IS NOT NULL)),
                CHECK (kind <> 'AUDIO' OR media_allowed = 1)
            );
            CREATE UNIQUE INDEX idx_field_definitions_unique
                ON field_definitions(flashcard_type_id, COALESCE(language_id, 0), key);

            -- A link between independent flashcards, for features that relate them.
            CREATE TABLE stacks (
                id         INTEGER PRIMARY KEY,
                -- Opaque public identifier. Database ids never leave the backend.
                uuid       TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE flashcards (
                id                INTEGER PRIMARY KEY,
                -- Opaque public identifier. Database ids never leave the backend.
                uuid              TEXT NOT NULL UNIQUE,
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

            -- Unsaved flashcards being edited. A deck has one list of drafts, in `position` order,
            -- mixing every flashcard type. `values_json` maps a field key to its content and is never
            -- queried by field.
            CREATE TABLE flashcard_drafts (
                id                INTEGER PRIMARY KEY,
                uuid              TEXT NOT NULL UNIQUE,
                deck_id           INTEGER NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
                flashcard_type_id INTEGER NOT NULL REFERENCES flashcard_types(id) ON DELETE RESTRICT,
                position          INTEGER NOT NULL,
                values_json       TEXT NOT NULL DEFAULT '{}',
                -- Drafts that share a value are one stack in the editor and are linked by a stack
                -- when published.
                stack_uuid        TEXT,
                updated_at        TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX idx_flashcard_drafts_deck_position
                ON flashcard_drafts(deck_id, position);
            CREATE INDEX idx_flashcard_drafts_stack_uuid ON flashcard_drafts(stack_uuid);

            -- Media files are kept in S3 and cached in a local folder. `file_name` is
            -- `<sha256>.<ext>` inside that folder. `remote_key` is the S3 object key, NULL until the
            -- file is uploaded. A sync compares the rows with the files on disk.
            CREATE TABLE field_media (
                id                 INTEGER PRIMARY KEY,
                flashcard_field_id INTEGER NOT NULL REFERENCES flashcard_fields(id) ON DELETE CASCADE,
                kind               TEXT NOT NULL CHECK (kind IN ('IMAGE', 'AUDIO', 'VIDEO')),
                mime_type          TEXT NOT NULL,
                sha256             TEXT NOT NULL,
                file_name          TEXT NOT NULL,
                remote_key         TEXT,
                size_bytes         INTEGER NOT NULL,
                position           INTEGER NOT NULL DEFAULT 0,
                created_at         TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX idx_field_media_flashcard_field_id ON field_media(flashcard_field_id);
            CREATE INDEX idx_field_media_sha256 ON field_media(sha256);

            -- Media attached to a draft, by the key of the field it is for. Publishing copies the
            -- rows into field_media. Attaching the same file to a field twice has no effect.
            CREATE TABLE flashcard_draft_media (
                id         INTEGER PRIMARY KEY,
                draft_id   INTEGER NOT NULL REFERENCES flashcard_drafts(id) ON DELETE CASCADE,
                field_key  TEXT NOT NULL,
                kind       TEXT NOT NULL CHECK (kind IN ('IMAGE', 'AUDIO', 'VIDEO')),
                mime_type  TEXT NOT NULL,
                sha256     TEXT NOT NULL,
                file_name  TEXT NOT NULL,
                size_bytes INTEGER NOT NULL,
                position   INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE (draft_id, field_key, sha256)
            );

            -- All timestamps are UTC text in the 'YYYY-MM-DD HH:MM:SS' format of datetime('now').

            -- SM-2 scheduling state. A flashcard with no row has never been reviewed.
            CREATE TABLE sm2_states (
                flashcard_id     INTEGER PRIMARY KEY REFERENCES flashcards(id) ON DELETE CASCADE,
                due_at           TEXT NOT NULL,
                interval_days    REAL NOT NULL,
                ease_factor      REAL NOT NULL,
                repetitions      INTEGER NOT NULL,
                last_reviewed_at TEXT,
                -- Reviews passed in the first wave of a new card (it needs two unless it is answered
                -- Easy). NULL once the card has graduated.
                learning_step    INTEGER CHECK (learning_step IS NULL OR learning_step >= 0)
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
                duration_ms    INTEGER,
                -- What the card was when it was answered, for the deck's daily limits: NEW is the
                -- first review ever, LEARNING a card not yet recalled, REVIEW a recalled card.
                phase          TEXT NOT NULL CHECK (phase IN ('NEW', 'LEARNING', 'REVIEW')),
                -- The card's sm2_states and card_stats rows as they were before this review, as
                -- JSON (null for a row that did not exist). Undoing the review puts them back.
                previous       TEXT NOT NULL DEFAULT '{"state":null,"stats":null}'
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

            -- Seed data: the languages and accents the app starts with. The first variant listed for
            -- a language is its default.
            INSERT INTO languages (iso_639_1, iso_639_3, name) VALUES
                ('en', 'eng', 'English'),
                ('fr', 'fra', 'French'),
                ('pt', 'por', 'Portuguese'),
                ('es', 'spa', 'Spanish'),
                ('de', 'deu', 'German'),
                ('it', 'ita', 'Italian');

            INSERT INTO language_variants (language_id, bcp47_code, variant_name, country_code)
            SELECT l.id, v.bcp47_code, v.variant_name, v.country_code
            FROM (
                SELECT 'eng' AS iso, 'en-US' AS bcp47_code, 'English (United States)' AS variant_name, 'US' AS country_code
                UNION ALL SELECT 'eng', 'en-GB', 'English (United Kingdom)', 'GB'
                UNION ALL SELECT 'fra', 'fr-FR', 'French (France)', 'FR'
                UNION ALL SELECT 'por', 'pt-BR', 'Portuguese (Brazil)', 'BR'
                UNION ALL SELECT 'spa', 'es-ES', 'Spanish (Spain)', 'ES'
                UNION ALL SELECT 'spa', 'es-MX', 'Spanish (Mexico)', 'MX'
                UNION ALL SELECT 'deu', 'de-DE', 'German (Germany)', 'DE'
                UNION ALL SELECT 'ita', 'it-IT', 'Italian (Italy)', 'IT'
            ) AS v
            JOIN languages l ON l.iso_639_3 = v.iso;

            UPDATE languages SET default_variant_id = (
                SELECT lv.id FROM language_variants lv
                WHERE lv.language_id = languages.id ORDER BY lv.id LIMIT 1
            );

            -- Seed data: flashcard types and their language-agnostic fields.
            INSERT INTO flashcard_types (slug, name, description) VALUES
                ('vocabulary',   'Vocabulary',   'A phrase on the front, the word or expression and its meaning on the back'),
                ('phonetic',     'Phonetic',     'A phrase on the front, its IPA phonetics on the back'),
                ('orthographic', 'Orthographic', 'Dictation: the audio of a phrase and the phrase itself, both on the front, with no back');

            INSERT INTO field_definitions
                (flashcard_type_id, language_id, side, key, label, position, is_required, media_allowed, kind, source_key)
            SELECT t.id, NULL, d.side, d.key, d.label, d.position, d.is_required, d.media_allowed, d.kind, d.source_key
            FROM (
                SELECT 'vocabulary' AS slug, 'FRONT' AS side, 'phrase' AS key, 'Phrase' AS label, 0 AS position, 1 AS is_required, 1 AS media_allowed, 'RICH' AS kind, NULL AS source_key
                UNION ALL SELECT 'vocabulary', 'BACK', 'word_or_expression', 'Word or expression', 0, 1, 0, 'PLAIN', NULL
                UNION ALL SELECT 'vocabulary', 'BACK', 'meaning', 'Meaning', 1, 1, 1, 'PLAIN', NULL
                UNION ALL SELECT 'phonetic', 'FRONT', 'phrase', 'Phrase', 0, 1, 0, 'PLAIN', NULL
                UNION ALL SELECT 'phonetic', 'BACK', 'ipa_phonetics', 'IPA phonetics', 0, 1, 0, 'IPA', 'phrase'
                UNION ALL SELECT 'orthographic', 'FRONT', 'audio', 'Audio', 0, 1, 1, 'AUDIO', 'phrase'
                UNION ALL SELECT 'orthographic', 'FRONT', 'phrase', 'Phrase', 1, 1, 0, 'PLAIN', NULL
            ) AS d
            JOIN flashcard_types t ON t.slug = d.slug;
            "#,
        ),
        // v1.1: the time the study day starts, per deck and flashcard type. After it, the review
        // cards due that day are all offered, whatever hour they were scheduled for. Minutes after
        // UTC midnight: 360 is 06:00 UTC, which is 03:00 in São Paulo.
        M::up(
            "ALTER TABLE deck_type_settings ADD COLUMN day_starts_at_minute INTEGER NOT NULL DEFAULT 360
                 CHECK (day_starts_at_minute BETWEEN 0 AND 1439);",
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn v1_1_adds_the_rollover_to_existing_settings_and_keeps_the_data() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", true).unwrap();
        migrations().to_version(&mut conn, 1).unwrap();
        conn.execute_batch(
            "INSERT INTO decks (name, language_variant_id) VALUES ('Old', 1);
             INSERT INTO deck_type_settings (deck_id, flashcard_type_id, new_cards_per_day, max_reviews_per_day, autoplay_audio)
             VALUES (1, 1, 5, 50, 0);",
        )
        .unwrap();

        migrations().to_latest(&mut conn).unwrap();

        let row: (i64, i64, i64, i64) = conn
            .query_row(
                "SELECT new_cards_per_day, max_reviews_per_day, autoplay_audio, day_starts_at_minute FROM deck_type_settings",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap();
        assert_eq!(row, (5, 50, 0, 360), "the old row keeps its options and starts the day at 06:00 UTC");
        assert!(conn
            .execute("UPDATE deck_type_settings SET day_starts_at_minute = 1440", [])
            .is_err(), "the minute is checked by the database too");
    }
}
