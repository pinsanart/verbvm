# Arquitetura

O sistema é um **monolito modular**. Cada módulo se comporta como um pequeno serviço que os outros só alcançam pela API pública dele, mas tudo roda no mesmo processo e no mesmo arquivo SQLite.

## Camadas

```
model <- repositories <- services <- (orchestrators) <- viewmodel <- view
 Rust       Rust          Rust          Rust             TS         TS
```

| Camada | Onde | Responsabilidade | Não faz |
|---|---|---|---|
| model | `src-tauri/src/modules/<módulo>/model.rs` | Entidades com campos privados, getters e construtores que validam. As invariantes e as regras que dependem só da entidade moram aqui | Não conhece banco nem serde |
| repositories | `.../repository.rs` | Só CRUD. Um trait `Send + Sync` com uma implementação `Sqlite*`, que mapeia linhas em entidades | Nenhuma regra de negócio |
| services | `.../service.rs` | Regras de negócio. Implementam o trait público `*Api` do módulo | Não conhecem Tauri nem DTOs |
| orchestrators | `src-tauri/src/orchestrators/` (ainda não existe) | Casos de uso que coordenam 2 ou mais services | Não acessam repositórios |
| commands | `src-tauri/src/commands/` | Fronteira com o Tauri. Adaptadores finos sobre um service ou orchestrator, com DTOs próprios (camelCase) e `CommandError { kind, message }` | Não têm regra de negócio |
| viewmodel | `src/viewmodels/` | A "API" da view, em TypeScript: estado, ações e tratamento de erro. Chama os wrappers de `src/api/` | Não renderiza |
| view | `src/routes/`, `src/components/` | Renderiza o viewmodel e guarda só estado de UI, como os campos de um formulário | Não chama `invoke` |

Os DTOs ficam na camada de commands para que o formato que a view precisa nunca vaze para as entidades.

## Regras dos módulos

- As tabelas de um módulo só são tocadas pelos repositórios dele. Não há JOIN entre módulos.
- Um módulo só é alcançado pelo trait `*Api` dele, nunca pelo struct do service nem pelos repositórios.
- As foreign keys entre módulos continuam no banco, por integridade. Por isso um `ON DELETE CASCADE` pode atravessar a fronteira de um módulo (apagar um flashcard apaga o estado de revisão dele, por exemplo).
- Todo horário é UTC, e os services recebem um `Clock` em vez de chamar `Utc::now()` (veja [backend.md](backend.md)).

## Módulos

| Módulo | Tabelas | Situação |
|---|---|---|
| `languages` | `languages`, `language_variants` | Implementado, com commands e tela de teste |
| `reviews` | `sm2_states`, `review_logs`, `card_stats` | Implementado (model, repository, service). Sem commands ainda |
| `decks` | `decks` | Planejado, é o próximo |
| `card_types` | `flashcard_types`, `field_definitions` | Planejado |
| `flashcards` | `flashcards`, `flashcard_fields`, `stacks` | Planejado |

Dependências previstas, sempre em um sentido só e sem ciclos:

```
languages <- decks <- flashcards -> card_types -> languages
                          ^
                       reviews (liga a flashcards só por id, no orchestrator)
```

## Convenções ainda não confirmadas

Estas duas foram assumidas e valem até o dono do projeto decidir diferente:

1. **Services não chamam services de outros módulos.** Uma checagem que atravessa módulos (por exemplo, "o variant do deck existe") vai para um orchestrator, que consulta um service e depois chama o outro. O banco continua garantindo a integridade pelas FKs.
2. **Casos de uso com um único service pulam o orchestrator.** O command chama o service direto. O orchestrator só aparece quando coordena 2 ou mais.
