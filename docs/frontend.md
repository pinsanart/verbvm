# Frontend

React 19 + TypeScript, build com Vite e estilo com Tailwind CSS v4 (plugin `@tailwindcss/vite`, entrada em `src/index.css`). O frontend só apresenta: dados e regras ficam no Rust.

## Estrutura

```
src/
├── main.tsx              # rotas (createHashRouter)
├── routes/               # telas (as views)
├── components/           # componentes reutilizáveis
├── viewmodels/           # a "API" das views
├── api/                  # wrappers tipados do invoke do Tauri
├── types/                # tipos compartilhados
└── data/                 # dados estáticos, como os botões da navegação
```

## Rotas

`main.tsx` usa `createHashRouter`, o roteamento por hash que combina com o frontend servido como arquivo pelo Tauri. `routes/Root.tsx` é o layout, com a barra lateral responsiva (`AppNavigationBar`, alimentada por `data/NavigationButton.ts`).

Rotas: `Home`, `review`, `create`, `dashboard`, `settings` e `language_tests`, uma tela manual para criar idiomas e variants.

## View, viewmodel e api

O exemplo de referência é `routes/LanguageTests.tsx` com `viewmodels/useLanguagesViewModel.ts` e `api/languages.ts`.

- **`api/languages.ts`**: funções finas, uma por command (`listLanguages`, `createLanguage`...), com tipos de entrada e saída. Também tem `errorMessage(err)`, que transforma o que o `invoke` rejeitou em texto (lê o `message` do `CommandError`).
- **`viewmodels/useLanguagesViewModel.ts`**: um hook do React. Guarda o estado (idiomas, variants, idioma selecionado, erro) e expõe ações (`addLanguage`, `removeLanguage`, `addVariant`, `makeDefaultVariant`, `removeVariant`, `selectLanguage`). As ações devolvem `true` quando dão certo, para a view limpar os formulários.
- **`routes/LanguageTests.tsx`**: só renderiza e guarda o estado dos inputs. Não importa nada de `api/`.
- **`types/Language.ts`**: os tipos que espelham os DTOs do Rust (`Language`, `LanguageVariant`, `NewLanguage`, `CommandError`...). Os nomes dos campos vêm do `serde(rename_all = "camelCase")` e dos `rename` explícitos (`iso6391`, `iso6393`).

### Criando uma tela

1. Escreva os commands no Rust e liste-os em `generate_handler!`.
2. Crie os tipos em `types/` e os wrappers em `api/`.
3. Crie o viewmodel em `viewmodels/use<Nome>ViewModel.ts`. Ele concentra estado, ações e erros.
4. Crie a view em `routes/` consumindo o viewmodel, e registre a rota em `main.tsx`. Se precisar de acesso pela barra lateral, adicione um botão em `data/NavigationButton.ts`.

## Rodando

- `npm run tauri dev`: o app completo.
- `npm run dev`: só o frontend, em http://localhost:1420. O `invoke` do Tauri não existe no navegador, então as telas que chamam o backend mostram um erro. Use o app completo para elas.
- `npm run build`: `tsc` e depois `vite build`.

## Pendências

`types/Flashcard.ts` e os componentes de `Create` (`FlashcardInput` e campos) ainda usam ids string e uma união fixa de tipos (`vocabulary`, `phonetic`, `orthographic`). Eles não refletem o schema do banco e serão reconciliados quando os commands de flashcards existirem.
