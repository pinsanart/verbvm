use rusqlite_migration::{Migrations, M};

pub fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        // v1.0

        //Test
        M::up("CREATE TABLE users (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL
        );"),
    ])
}