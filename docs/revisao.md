# Revisão de flashcards

O módulo `reviews` (`src-tauri/src/modules/reviews/`) agenda as revisões com o algoritmo **SM-2** e guarda o histórico e as estatísticas. Machine learning (FSRS ou um LSTM) é um passo futuro.

## Tabelas

Todas ligadas a `flashcards` com `ON DELETE CASCADE`, com horários em UTC.

| Tabela | Conteúdo |
|---|---|
| `sm2_states` | Estado atual por card: `due_at`, `interval_days`, `ease_factor`, `repetitions`, `last_reviewed_at`. A linha é criada na primeira revisão. Um card sem linha nunca foi revisado, e criar um card não depende deste módulo |
| `review_logs` | Uma linha por revisão, só de inserção: `reviewed_at`, `rating` (1 a 4), `elapsed_days` (nulo na primeira), `scheduled_days`, `duration_ms` |
| `card_stats` | Contadores por card, independentes do algoritmo: `review_count`, `correct_count`, `lapses`, `total_duration_ms`, `last_rating` |

Um futuro `fsrs_states` ficaria ao lado de `sm2_states`.

## Notas e regras do SM-2

As notas são `Again` (1), `Hard` (2), `Good` (3) e `Easy` (4). O fator de facilidade começa em 2,5 e nunca cai abaixo de 1,3. O intervalo usa o fator de antes da nota ser aplicada.

| Nota | Repetições | Fator de facilidade | Próximo intervalo |
|---|---|---|---|
| Again | volta a 0 | -0,20 | 10 minutos |
| Hard | +1 | -0,15 | anterior x 1,2, no mínimo 1 dia |
| Good | +1 | não muda | 1 dia, depois 6 dias, depois anterior x fator |
| Easy | +1 | +0,15 | o intervalo do Good x 1,3 |

A regra está em `Sm2State::review`, no `model.rs`, e é testada tabela a tabela.

### Estatísticas

- `correct_count` conta as revisões que não foram `Again`.
- Um `lapse` é um `Again` num card que já tinha sido lembrado ao menos uma vez. O `Again` na primeira tentativa não conta como lapso.

## Fluxo de `record_review`

`ReviewApi::record_review(flashcard_id, rating, duration_ms)`:

1. Lê a hora do `Clock` e carrega o estado e as estatísticas do card (ou cria os iniciais).
2. Calcula `elapsed_days` desde `last_reviewed_at`.
3. Aplica a nota ao estado (`Sm2State::review`) e às estatísticas (`CardStats::record`).
4. Monta o `ReviewLog` com o intervalo agendado.
5. `save_review` grava estado, estatísticas e log em **uma transação**: ou os três entram, ou nenhum.

Outras operações: `due(limit)` (cards vencidos, os mais antigos primeiro), `states_for(ids)`, `stats(flashcard_id)` e `history(flashcard_id)`. Revisar um card que não existe dá `Conflict`, pela FK.

Os três acessos de `record_review` (ler estado, ler estatísticas, gravar) não formam uma única transação. Com um usuário só, isso não é problema, mas duas revisões simultâneas do mesmo card poderiam se sobrescrever.

## Ainda não implementado

- **Orchestrator `ReviewSessionOrchestrator`**: pede ao service de flashcards os cards de um deck e ao de reviews `states_for(ids)`, e junta os vencidos com os novos. Sem JOIN entre módulos. Depende do módulo de flashcards.
- Commands, `api/reviews.ts` e `useReviewViewModel`.

## Caminho para ML

O desenho já separa o que o ML vai precisar:

- `review_logs` guarda a sequência de revisões de cada card, que é o dataset de treino. O histórico completo para treino deve ficar em um servidor no futuro. Localmente ele some junto com o card.
- Trocar de algoritmo significa uma segunda tabela de estado ao lado de `sm2_states` e, nesse ponto, um `trait Scheduler` com as duas implementações. Ainda não existe trait, porque há um algoritmo só.

Opções de implementação (de memória, confirme as versões e APIs antes de adotar):

- **FSRS** (crate `fsrs`, do projeto open-spaced-repetition): poucos parâmetros, funciona desde as primeiras revisões e costuma ficar competitivo com redes recorrentes em benchmarks públicos.
- **LSTM**: a rede recebe a sequência de revisões de um card (nota, log do intervalo desde a anterior e, se quiser, tipo e idioma) e prevê a **estabilidade** `s` da memória. Assim a retenção `R(t) = 0,9^(t/s)` é monotônica e o intervalo ideal sai direto. O treino minimiza a entropia cruzada binária (lembrou ou não). Em Rust, o Burn tem `nn::Lstm`, autodiff e backend em CPU, e treina e infere dentro do app. O `ort` (ONNX Runtime) só infere, então o treino ficaria em Python. O treino deve rodar em uma thread de fundo, nunca nos commands.
- **Pouco dado**: um LSTM por usuário precisa de milhares de revisões para superar um modelo simples. As opções são pré-treinar em um dataset público e ajustar localmente, ou começar de FSRS.
- **Avaliação**: comparar os algoritmos por log-loss em um conjunto de teste separado por data.
