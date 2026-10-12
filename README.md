# verbvm

App desktop de flashcards para aprender idiomas. Funciona **offline**: todos os dados ficam em um arquivo SQLite local, sem servidor.

> Em desenvolvimento. Hoje existem o cadastro de idiomas e variantes, o motor de revisão SM-2 e a base do banco. Decks e flashcards são os próximos módulos.

## Ideias principais

- **Offline primeiro.** Tauri 2 (Rust) em volta de um frontend React 19 + TypeScript, com SQLite embutido.
- **O Rust cuida de tudo.** Dados, regras e agendamento ficam no backend. O frontend só apresenta.
- **Layout de cartão flexível.** Cada tipo de flashcard (vocabulário, ortografia, fonética) tem os campos definidos por dados, frente e verso, e um idioma pode acrescentar campos próprios (romaji e hiragana para japonês, por exemplo) sem mudar o schema.
- **Revisão espaçada.** Agendamento SM-2 hoje, com o caminho aberto para FSRS ou um modelo LSTM depois.

## Tecnologias

Tauri 2, Rust (`rusqlite`, `rusqlite_migration`, `chrono`), React 19, TypeScript, Vite, Tailwind CSS v4.

## Como rodar

Pré-requisitos: [Node.js](https://nodejs.org), [Rust](https://rustup.rs) e as [dependências do Tauri](https://v2.tauri.app/start/prerequisites/) para o seu sistema.

```bash
npm install
npm run tauri dev      # o app desktop completo
```

| Comando | Para quê |
|---|---|
| `npm run tauri dev` | App completo (frontend e backend) |
| `npm run dev` | Só o frontend, em http://localhost:1420 (sem o backend) |
| `npm run build` | Checagem de tipos e build do frontend |
| `npm run tauri build` | App empacotado |
| `cargo test` (em `src-tauri/`) | Testes do backend |

O banco fica em `~/.local/share/verbvm/verbvm.db` no Linux. Enquanto o schema ainda muda, apague esse arquivo (e os `-wal` e `-shm` ao lado) depois de editar as migrations.

## Estrutura

```
src/          # frontend: routes, components, viewmodels, api, types
src-tauri/    # backend Rust: modules, commands, dependencies (banco, relógio)
docs/         # documentação
```

A arquitetura em camadas, `model <- repositories <- services <- (orchestrators) <- viewmodel <- view`, está explicada em [docs/arquitetura.md](docs/arquitetura.md).

## Documentação

Veja o [índice em docs/](docs/README.md): arquitetura, modelo de dados, backend, frontend, o sistema de revisão e o guia de desenvolvimento.

## Ambiente de desenvolvimento recomendado

[VS Code](https://code.visualstudio.com/) com as extensões [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) e [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).
