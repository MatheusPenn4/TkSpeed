# Contribuindo com o TkSpeed

Convenções de engenharia do repositório. Operacional — não é documento de produto.

## Branches

- `main` — sempre verde (CI passa). Nada é commitado direto aqui exceto o commit inicial.
- `feat/<slice>` — uma branch por slice de execução. Ex.: `feat/a3-rollback-center`, `feat/a2-tests`.
- `fix/<assunto>` — correções pontuais.
- Fluxo: branch a partir de `main` → abre PR → CI verde → merge em `main`.

Sem GitFlow. Sem branches de release de longa duração nesta fase.

## Versionamento (SemVer)

Produto em `0.x` (pré-1.0): a API/superfície pode mudar.

- Formato Alpha: `0.1.0-alpha.N` (N incrementa a cada build de Alpha distribuído).
- `0.MINOR.PATCH` segue SemVer normal após o Alpha.
- A versão vive em `Cargo.toml` (`workspace.package.version`) e `apps/desktop/package.json`.

## Tags

- Uma tag anotada por release: `git tag -a v0.1.0-alpha.1 -m "Alpha 1"`.
- Tags só em commits de `main` que passaram no CI.
- `release/CHANGELOG.md` é a fonte de verdade do que entrou em cada versão (formato Keep a Changelog).

## Convenção de commits (Conventional Commits)

`<tipo>(<escopo opcional>): <resumo no imperativo>`

Tipos: `feat`, `fix`, `test`, `refactor`, `chore`, `docs`, `ci`, `perf`, `build`.

Exemplos:
- `feat(rollback): adiciona Rollback Center (A3)`
- `test(rollback): cobre os 4 ReversibleAction (A2.1)`
- `ci: pipeline build+test+clippy+frontend (A1.3)`

Regras:
- Resumo ≤ 72 caracteres, imperativo, sem ponto final.
- Corpo opcional explicando o "porquê".
- Um commit por unidade lógica; evite commits "WIP" em `main`.

## Antes de abrir PR (espelha o CI)

```
cargo build --workspace --exclude tkspeed
cargo test  --workspace --exclude tkspeed
cargo clippy --workspace --exclude tkspeed --all-targets -- -D warnings
cd apps/desktop && npm ci && npm run build
```

> `tkspeed` (binário Tauri em `src-tauri`) é excluído de build/test/clippy do CI: exige o
> contexto/dist do frontend e o WebView no tempo de compilação, não tem testes próprios e é
> validado pelo `tauri build` no empacotamento (A6). A lógica testável vive nos crates de biblioteca.
