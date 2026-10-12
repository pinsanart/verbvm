# Backend

O backend é o crate `verbvm_lib` em `src-tauri/` (`lib.rs`, com `main.rs` só chamando `run()`).

## Estrutura

```
src-tauri/src/
├── lib.rs                  # setup do Tauri: abre o banco, monta os services, registra os commands
├── dependencies/           # infraestrutura
│   ├── clock.rs            # Clock (UTC) e o formato dos horários
│   ├── db/                 # SqliteHandle, DbError, migrations
│   └── http/               # HttpClient (NÃO compilado, veja desenvolvimento.md)
├── modules/                # um diretório por módulo
│   ├── languages/          # model, repository, service, error
│   └── reviews/            # model, repository, service, error
└── commands/               # fronteira com o Tauri: DTOs, CommandError, commands
```

## Banco e migrations

`SqliteHandle` (`dependencies/db/connection.rs`) embrulha um `Mutex<rusqlite::Connection>`. Ao abrir, ele cria o diretório pai, liga WAL e foreign keys, define `busy_timeout` e roda as migrations (`rusqlite_migration`). Todo acesso passa por `with_connection(|conn| ...)`. Escritas em várias tabelas usam `conn.unchecked_transaction()` dentro dessa chamada, que é o único jeito de compartilhar uma transação entre tabelas.

O arquivo do banco é `verbvm.db` no diretório de dados do app. No Linux, `~/.local/share/verbvm/verbvm.db` (o identificador do app é `verbvm`).

### Evoluindo o schema

O banco guarda no próprio arquivo qual versão de migration já foi aplicada. Isso tem duas consequências:

- **Em produção**, nunca edite uma migration que já rodou. Adicione uma nova `M::up(...)` ao fim da lista.
- **Durante o desenvolvimento**, enquanto o schema ainda muda, o dono do projeto edita a v1.0 mesmo. O banco de desenvolvimento que já aplicou a v1.0 não vai reaplicá-la, então é preciso apagar `verbvm.db` e os arquivos `verbvm.db-wal` e `verbvm.db-shm` ao lado dele e reabrir o app.

## Horário (UTC)

`dependencies/clock.rs` define o `trait Clock`, o `SystemClock` (hora atual em UTC, truncada para segundos) e, só em testes, o `FixedClock`, que só anda quando mandam. Também define `format_utc` e `parse_utc`, o único lugar que conhece o formato `YYYY-MM-DD HH:MM:SS`. Services nunca chamam `Utc::now()`. Eles recebem um `Arc<dyn Clock>`.

## Startup (`lib.rs`)

Em `setup`, o app abre `verbvm.db`, monta os services e os registra como estado do Tauri, por exemplo `Arc<dyn LanguageApi>`. Os commands são listados em `generate_handler!`. Commands próprios não precisam de entrada em `capabilities/default.json`.

O módulo `reviews` ainda não está ligado ao `lib.rs`, porque seus commands dependem do módulo de flashcards.

## Um módulo passo a passo: `languages`

`src-tauri/src/modules/languages/` é a referência.

- **`model.rs`**: `Language` e `LanguageVariant`. Campos privados com getters. `new` cria uma entidade ainda não salva (sem id) e valida tudo. `restore` (visível só no módulo) reconstrói a partir do banco. Regras como `revise(...)` (troca os atributos editáveis, tudo ou nada) e `assign_default_variant(&variant)` (o variant precisa estar salvo e ser do idioma) moram na entidade.
- **`repository.rs`**: os traits `LanguageRepository` e `LanguageVariantRepository` (`insert`, `find`, `find_all`, `update`, `delete`) e as implementações `Sqlite*`. `insert` usa `INSERT ... RETURNING` para devolver a entidade com o id.
- **`service.rs`**: o trait `LanguageApi` (a API pública do módulo) e o `LanguageService`, que recebe `Arc<dyn ...Repository>`. `LanguageService::sqlite(db)` monta a versão com SQLite. As regras: criar o primeiro variant o torna o default, e o default precisa pertencer ao idioma.
- **`error.rs`**: `LanguagesError` (`NotFound`, `Invalid`, `Conflict`, `Db`). Violações de constraint do SQLite, como `iso_639_3` duplicado ou apagar um variant usado por um deck, viram `Conflict`.
- **`languages.rs`**: documentação do módulo e os `pub use` que formam a superfície pública.

### Criando um módulo novo

1. Crie `modules/<nome>.rs` (docs `//!` e `pub use`) e `modules/<nome>/{model,repository,service,error}.rs`. Declare `pub mod <nome>;` em `modules.rs`.
2. Adicione as tabelas do módulo à migration v1.0 (veja "Evoluindo o schema").
3. No model, entidades com campos privados, `new` validando e `restore` com visibilidade `pub(super)`.
4. No repositório, um trait `Send + Sync` e a implementação `Sqlite*`. Só CRUD.
5. No service, o trait `*Api` (a única coisa que outros módulos podem usar) e a implementação com as regras. Horário vem de um `Clock`.
6. Testes em `model.rs` e `service.rs` (veja abaixo).
7. Quando precisar expor ao frontend, crie os commands (próxima seção) e registre o service em `lib.rs`.

## Commands (`src-tauri/src/commands/`)

Adaptadores finos sobre um service (ou orchestrator):

- Os DTOs (`LanguageDto`, `LanguageVariantDto`, os `New*Input`) têm `serde` e nomes em camelCase. As entidades não têm serde.
- Erros viram `CommandError { kind, message }`, onde `kind` é `not_found`, `invalid`, `conflict` ou `internal`. O frontend decide o que mostrar a partir do `kind`.
- Commands de `languages`: `list_languages`, `create_language`, `delete_language`, `list_language_variants`, `create_language_variant`, `set_default_language_variant`, `delete_language_variant`.
- Os commands são `async` e recebem o service como `State<'_, Arc<dyn LanguageApi>>`.

## Testes

Não há linter configurado. Os testes são unitários, dentro dos módulos, e rodam com `cargo test` em `src-tauri/`. Para um só: `cargo test <nome>`.

- Regras das entidades (`model.rs`) testam sem banco.
- Tests de service rodam contra um arquivo SQLite temporário, com as migrations reais, e removem o arquivo no fim. Tabelas de outros módulos, como `flashcards` nos testes de `reviews`, são preenchidas com SQL puro, porque o módulo testado não é dono delas.
- Tempo é controlado com `FixedClock`.
