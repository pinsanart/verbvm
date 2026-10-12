# Documentação do verbvm

verbvm é um app desktop de flashcards para aprender idiomas. Funciona **offline**: todos os dados ficam em um arquivo SQLite local, sem servidor. É um shell Tauri 2 (Rust) em volta de um frontend React 19 + TypeScript.

O Rust cuida de toda a lógica e de todos os dados. O frontend só apresenta.

## Índice

| Documento | Conteúdo |
|---|---|
| [arquitetura.md](arquitetura.md) | As camadas, as regras dos módulos e as convenções em aberto |
| [modelo-de-dados.md](modelo-de-dados.md) | Todas as tabelas, o diagrama ER e as decisões do schema |
| [backend.md](backend.md) | Banco, migrations, módulos, commands do Tauri e testes. Inclui o passo a passo para criar um módulo |
| [frontend.md](frontend.md) | Rotas, viewmodels, wrappers do `invoke`. Inclui o passo a passo para criar uma tela |
| [revisao.md](revisao.md) | O sistema de revisão: SM-2, tabelas, fluxo e o plano para ML |
| [desenvolvimento.md](desenvolvimento.md) | Comandos do dia a dia e limitações conhecidas |

A documentação de API do código Rust é gerada com `cargo doc --no-deps --document-private-items --open`, dentro de `src-tauri/`.
