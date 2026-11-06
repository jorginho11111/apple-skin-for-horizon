# AppleSkin para Horizon Android

Port não oficial de AppleSkin como **`.hmod`** para o port Android oficial do Horizon:
[`Vasysik/horizon-android`](https://github.com/Vasysik/horizon-android).

- **Versão deste código:** `0.4.2`
- **Minecraft:** 26.3
- **Horizon Android compatível:** `v0.7.7` (`80700a6f26afe53a368032d8b28bd364f7c3e7e4`)
- **Formato:** `.hmod` sandboxed (componente WebAssembly), **não** `.hnmod`
- **API de mods:** `0.1`
- **APKs:** `arm64-v8a` ou `x86_64` (64 bits, Android 9+)

> Este projeto não é oficial da Mojang/Microsoft nem do Horizon. O pacote é
> compilado pelo GitHub Actions a partir deste repositório.

## O que ele faz

Quando você segura uma comida na **mão principal**, o mod mostra:

- **somente as carnes/meias carnes restauradas pelo alimento**, sem repetir toda
  a linha de fome que você já tem;
- **contornos amarelos de carne do AppleSkin original** para o ganho potencial
  de saturação, limitado pela fome projetada — não uma faixa retangular;
- uma linha fixa **logo acima da barra de fome vanilla**, com **opacidade
  pulsante**: só a opacidade aumenta e diminui; os ícones **não se movem**;
- um painel de texto opcional, com modo compacto e posição configurável.

Isso é uma prévia de **restauração de fome**, não de regeneração de vida/corações.

### Como interpretar os ícones

- Fome **12 + pão (+5) → 17**: duas carnes cheias e uma meia carne de ganho;
  três contornos amarelos cheios para a saturação do pão (+6,0).
- Fome **16 + bacalhau cru (+2) → 18**: uma carne cheia de ganho e um contorno
  amarelo pequeno para a saturação (+0,4).
- Fome **15 + bacalhau cru (+2) → 17**: seguindo o AppleSkin original, o espaço
  da meia carne existente é redesenhado como cheio e o próximo espaço recebe
  uma meia carne. Os outros espaços já preenchidos não são repetidos.
- Com a fome cheia, uma maçã dourada ainda pode mostrar somente o ganho de
  saturação, pois pode ser comida sem restaurar fome. Alimentos comuns não
  mostram prévia quando a fome está cheia.

A seleção de fome e os quatro tamanhos de contorno seguem o
[`HUDOverlayHandler` original](https://github.com/squeek502/AppleSkin/blob/66fcdb8b12820c9b1cf3147abfdcf5530559f62b/java/squeek/appleskin/client/HUDOverlayHandler.java).
Como a saturação atual está oculta nesta API, **o amarelo começa pela direita,
a partir de uma base zero**. Ele representa o potencial do alimento, não a
posição exata em que a saturação será adicionada no jogador.

### Por que acima da barra, e não exatamente sobre os ícones?

No Horizon Android `v0.7.7`, o jogo desenha os shapes de mods (`render.set_hud`)
**antes** da barra de fome vanilla, apesar de a documentação da API dizer que eles
ficam por cima do HUD. Os ícones vanilla opacos cobrem uma prévia desenhada exatamente
na mesma posição. Por isso mantemos a linha compatível, fixa, 10 pixels GUI acima
da barra, sem movimento e sem depender de um APK modificado.

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

Por isso os contornos amarelos são o **ganho potencial de saturação do alimento**,
não a reserva real de saturação do jogador. O ganho é limitado pela fome projetada,
mas a saturação já existente pode deslocar ou reduzir o ganho real. Em veículo,
modo criativo ou com outro mod alterando o HUD, a linha pode sobrepor outros
elementos ou ficar onde a barra de fome não está visível. Apenas alimentos vanilla
de Minecraft 26.3 presentes em [`food-data/foods-26.3.json`](food-data/foods-26.3.json)
são reconhecidos. As capacidades `input`, `render` e `ui` são experimentais na API `0.1`.

A visibilidade da linha foi confirmada em aparelho Android na **v0.4.1**. O novo
desenho da **v0.4.2** ainda precisa de confirmação visual no APK: o sandbox oficial
não permite definir a comida segurada nem verificar os pixels do HUD. Para testar,
entre em sobrevivência, perca um pouco de fome e segure pão ou uma cenoura dourada.

## Download e instalação

Baixe o arquivo `appleskin-0.4.2-android.hmod` na
[Release v0.4.2](https://github.com/jorginho11111/apple-skin-for-horizon/releases/tag/v0.4.2).
O SHA-256, o relatório PT-BR e as informações de build estão anexados à Release.

1. Confira o SHA-256 publicado na Release.
2. No Horizon Android, abra **Mods**.
3. Use **Import file** e selecione o `.hmod`, ou cole a URL HTTPS do arquivo e
   toque em **Import .hmod**.
4. Abra **Installed**, ative **AppleSkin** e inicie o mundo.
5. Segure uma comida na mão principal.

Não renomeie `.jar`, `.zip` ou `.hnmod` para `.hmod`: o formato é diferente.

## Compilar localmente

Requisitos: Python 3.11+, Rust 1.95.0, alvo `wasm32-unknown-unknown` e a ferramenta
oficial `cargo-horizon-mod` do commit fixado do Horizon Android.

```sh
rustup target add wasm32-unknown-unknown
cargo install --git https://github.com/Vasysik/horizon-android \
  --rev 80700a6f26afe53a368032d8b28bd364f7c3e7e4 cargo-horizon-mod --locked

python3 -m unittest discover -s tests -v
cargo test --locked
cargo horizon-mod build
cargo horizon-mod check target/horizon-mod/appleskin.hmod --json
cargo horizon-mod test --hmod target/horizon-mod/appleskin.hmod
python3 tools/package.py
```

O SDK oficial está vendored em [`third_party/horizon-mod-sdk`](third_party/horizon-mod-sdk)
para tornar o build reproduzível sem baixar o jogo inteiro.

Os contornos amarelos são extraídos da textura original vendored, sem dependências
Python externas. Para regenerar o arquivo Rust e conferir a fonte fixada:

```sh
python3 tools/generate_saturation_icons.py
```

A textura fica fora de `assets/`/`data/`; o `.hmod` usa sprites vanilla e pequenos
retângulos de 1 pixel para reproduzir os contornos, sem exigir `registries` nem
um pacote de recursos adicional.

## CI e Releases

O workflow [`Build AppleSkin Android hmod`](.github/workflows/build.yml) executa:

- testes Python, incluindo fontes fixadas e geração determinística;
- `cargo fmt --check`;
- `cargo clippy --all-targets --locked -- -D warnings`;
- `cargo test --locked`, incluindo seleção só do ganho e limites dos contornos;
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
src/overlay.rs                seleção do ganho, geometria e pulsação + testes
src/saturation_icons.rs       contornos originais gerados em pixel runs
src/preview.rs                matemática pura da prévia + testes
src/foods.rs                  tabela gerada de alimentos vanilla 26.3
food-data/foods-26.3.json     dados nutricionais pinados do mcmeta 26.3
third_party/appleskin/       textura original e licença Unlicense
third_party/horizon-mod-sdk   SDK/WIT oficial vendored (MIT)
tests/commands.toml           testes de sandbox do cargo-horizon-mod
tests/test_project.py         testes de manifesto/versões/dados
tools/                        geração, empacotamento e notas de Release
```

## Créditos e licenças

- AppleSkin original: [squeek502/AppleSkin](https://github.com/squeek502/AppleSkin), Unlicense (`LICENSE-APPLESKIN`).
  Textura e comportamento de referência fixados no commit `66fcdb8b12820c9b1cf3147abfdcf5530559f62b`.
- Adaptação Horizon anterior: [soufied/AppleSkin-Horizon](https://github.com/soufied/AppleSkin-Horizon), MIT.
- SDK/WIT: [Vasysik/horizon-android](https://github.com/Vasysik/horizon-android), MIT (`third_party/horizon-mod-sdk/LICENSE`).
- Dados nutricionais: [misode/mcmeta](https://github.com/misode/mcmeta), tag `26.3-summary`.
- Este port: MIT (`LICENSE`). Veja [`NOTICE`](NOTICE) para fontes e alterações.
