# Desenvolvimento

## Comandos

| Comando | Para quê |
|---|---|
| `npm run tauri dev` | O app desktop completo (sobe o Vite e depois o shell Rust) |
| `npm run dev` | Só o frontend no navegador, em http://localhost:1420 (a porta é fixa, `strictPort` falha se estiver ocupada). O Tauri não existe aqui |
| `npm run build` | `tsc` e depois `vite build` para `dist/` |
| `npm run tauri build` | O app empacotado |
| `cargo check` / `cargo build` em `src-tauri/` | Só o Rust |
| `cargo test` em `src-tauri/` | Os testes unitários. `cargo test <nome>` roda um só |
| `cargo doc --no-deps --document-private-items --open` em `src-tauri/` | A documentação de API do código Rust |

Não há linter nem test runner de frontend configurados.

## Banco de desenvolvimento

O schema ainda está sendo editado direto na v1.0. Depois de mudar `migrations.rs`, apague `~/.local/share/verbvm/verbvm.db` e os arquivos `verbvm.db-wal` e `verbvm.db-shm`, e reabra o app. Você perde os dados de teste. Veja [backend.md](backend.md).

## Limitações conhecidas

- **`http` não compila.** `src-tauri/src/dependencies/http/` (um wrapper de `reqwest` com `Auth::{None, Bearer}`) está desligado: `pub mod http;` está comentado em `dependencies.rs`, porque o `reqwest` não está no `Cargo.toml`. Para ligar, adicione `reqwest` com a feature `json` e descomente.
- **Módulos que faltam:** `decks`, `card_types` e `flashcards`. O `reviews` está pronto, mas sem commands.
- **`field_media`** (mídia dos campos) e os **registros iniciais** de `flashcard_types` e `field_definitions` ainda não existem.
- **`types/Flashcard.ts`** e os componentes de `Create` não refletem o schema.
- **Avisos de código morto** no `cargo check`: aparecem para métodos que ainda não têm chamador, como `update_language` e `update_variant`, e para o módulo `reviews`, que não está ligado ao `lib.rs`.
