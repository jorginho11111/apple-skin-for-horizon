# AppleSkin para Horizon Android

Port de AppleSkin como **`.hmod`** para o port Android oficial do Horizon:
[`Vasysik/horizon-android`](https://github.com/Vasysik/horizon-android).

- **Minecraft:** 26.3
- **Horizon Android testado:** `v0.7.7` (`80700a6f26afe53a368032d8b28bd364f7c3e7e4`)
- **Formato:** `.hmod` sandboxed (componente WebAssembly), **não** `.hnmod`
- **API de mods:** `0.1`
- **APKs:** `arm64-v8a` ou `x86_64` (64 bits, Android 9+)

> Este projeto não é oficial da Mojang/Microsoft nem do Horizon. O pacote é
> compilado pelo GitHub Actions a partir deste repositório.

## O que ele faz

Quando você segura uma comida na **mão principal**, o mod desenha uma prévia da
fome projetada após comer:

- uma linha de ícones/meios ícones em posição fixa **logo acima da barra de fome
  vanilla**, com **opacidade pulsante** — só a opacidade aumenta e diminui; os
  ícones **não se movem**;
- uma faixa **amarela** sobre os ícones: o **ganho potencial de saturação** do
  alimento, limitado pela fome projetada;
- um painel de texto opcional, com modo compacto e posição configurável.

### Por que acima da barra, e não exatamente sobre os ícones?

No Horizon Android `v0.7.7`, o jogo desenha os shapes de mods (`render.set_hud`)
**antes** da barra de fome vanilla, apesar de a documentação da API dizer que eles
ficam por cima do HUD. Os ícones vanilla opacos cobrem uma prévia desenhada exatamente
na mesma posição. A v0.4.1 usa uma linha compatível logo acima da barra para ficar
visível no APK atual, sem movimento e sem depender de um APK modificado.

Comandos:

```text
/appleskin              liga/desliga
/appleskin overlay      alterna prévia de ícones / painel de texto
/appleskin pulse        alterna pulsação de opacidade
/appleskin status       mostra a prévia atual no chat
/appleskin compact      alterna modo compacto (texto)
/appleskin position     alterna posição (texto)
/appleskin help         ajuda
```

Também há configurações em **Options > Mods > AppleSkin**.

## Limitações importantes da API Android

A API `0.1` do Horizon Android expõe a fome atual do jogador e o item da mão
principal, mas **não** expõe:

- saturação atual;
- exaustão;
- mão secundária;
- modo de jogo.

Por isso a faixa amarela é o **ganho potencial de saturação do alimento**, não
a reserva real de saturação do jogador. Em veículo, modo criativo ou com outro mod
alterando o HUD, a linha de prévia pode sobrepor outros elementos ou ficar onde a
barra de fome não está visível. Apenas alimentos vanilla de Minecraft 26.3 presentes
em [`food-data/foods-26.3.json`](food-data/foods-26.3.json) são reconhecidos. As
capacidades `input`, `render` e `ui` são experimentais na API `0.1`.

## Download e instalação

Baixe o arquivo `appleskin-*-android.hmod` na
[última Release](https://github.com/jorginho11111/apple-skin-for-horizon/releases/latest).

1. Confira o SHA-256 publicado na Release.
2. No Horizon Android, abra **Mods**.
3. Use **Import file** e selecione o `.hmod`, ou cole a URL HTTPS do arquivo e
   toque em **Import .hmod**.
4. Abra **Installed**, ative **AppleSkin** e inicie o mundo.
5. Segure uma comida na mão principal.

Não renomeie `.jar`, `.zip` ou `.hnmod` para `.hmod`: o formato é diferente.

## Compilar localmente

Requisitos: Rust 1.95.0, alvo `wasm32-unknown-unknown` e a ferramenta oficial
`cargo-horizon-mod` do commit fixado do Horizon Android.

```sh
rustup target add wasm32-unknown-unknown
cargo install --git https://github.com/Vasysik/horizon-android \
  --rev 80700a6f26afe53a368032d8b28bd364f7c3e7e4 cargo-horizon-mod --locked

cargo test --locked
cargo horizon-mod build
cargo horizon-mod check target/horizon-mod/appleskin.hmod --json
cargo horizon-mod test --hmod target/horizon-mod/appleskin.hmod
python3 tools/package.py
```

O SDK oficial está vendored em [`third_party/horizon-mod-sdk`](third_party/horizon-mod-sdk)
para tornar o build reproduzível sem baixar o jogo inteiro.

## CI e Releases

O workflow [`Build AppleSkin Android hmod`](.github/workflows/build.yml) executa:

- testes Python;
- `cargo fmt --check`;
- `cargo clippy --all-targets --locked -- -D warnings`;
- `cargo test --locked`;
- `cargo horizon-mod build`;
- `cargo horizon-mod check` (inspeção oficial + trial no sandbox);
- `cargo horizon-mod test` (scripts de sandbox);
- empacotamento versionado com SHA-256 e relatório.

Ao chegar em `main`, a Release é publicada/atualizada com o `.hmod`, checksum,
`build.json`, `check.json`, licenças e um relatório PT-BR com versões, instruções
de download e limitações/erros conhecidos. Se o CI falhar, a Release não é
publicada automaticamente; o erro fica no GitHub Actions.

## Estrutura

```text
horizon-mod.toml              manifesto .hmod (API 0.1, client)
src/lib.rs                    mod Rust: HUD, comandos, configurações
src/overlay.rs                geometria da prévia compatível e pulsação + testes
src/preview.rs                matemática pura da prévia + testes
src/foods.rs                  tabela gerada de alimentos vanilla 26.3
food-data/foods-26.3.json     dados nutricionais pinados do mcmeta 26.3
third_party/horizon-mod-sdk   SDK/WIT oficial vendored (MIT)
tests/commands.toml           testes de sandbox do cargo-horizon-mod
tests/test_project.py         testes de manifesto/versões/dados
tools/                        geração, empacotamento e notas de Release
```

## Créditos e licenças

- AppleSkin original: [squeek502/AppleSkin](https://github.com/squeek502/AppleSkin), Unlicense (`LICENSE-APPLESKIN`).
- Adaptação Horizon anterior: [soufied/AppleSkin-Horizon](https://github.com/soufied/AppleSkin-Horizon), MIT.
- SDK/WIT: [Vasysik/horizon-android](https://github.com/Vasysik/horizon-android), MIT (`third_party/horizon-mod-sdk/LICENSE`).
- Dados nutricionais: [misode/mcmeta](https://github.com/misode/mcmeta), tag `26.3-summary`.
- Este port: MIT (`LICENSE`). Veja [`NOTICE`](NOTICE).
