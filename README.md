# AppleSkin for Horizon

Porte nativo do AppleSkin para **Horizon / Minecraft 26.3**, com previsão de fome e saturação a partir da comida que o personagem está segurando.

> **Formato:** este comportamento precisa de código nativo. O artefato é **`.hnmod`**, não um `.hmod` somente de dados/assets. O launcher compila o código Rust dentro do Horizon ao instalar/ativar o mod.

## O que faz

- Lê os valores nutricionais do item na mão ativa ou no slot selecionado do hotbar.
- Mostra a previsão de fome e saturação sobre a barra, atualizando ao trocar de item.
- Pode usar a comida da mão secundária quando a principal não tiver comida consumível.
- Mostra saturação atual e uma faixa de exaustão.
- Mantém animações de fade/pulse e configurações em **Options > Mods > AppleSkin**.
- Usa os hooks nomeados `gui_layer_post`, `mods`, `settings` e `settings_click` da API nativa do Horizon.

A previsão usa os dados de comida que o cliente conhece. Em servidores, valores não sincronizados ou alterados por plugins podem não ser representados exatamente; a exaustão remota não é inventada.

## Baixar e instalar

1. Abra a execução mais recente de **Build AppleSkin native** na aba **Actions** deste repositório.
2. Baixe o artefato `appleskin-native` e extraia o `.zip`.
3. Coloque `appleskin-0.2.0.hnmod` em `horizon-native-mods/`, na pasta do jogo que contém `options.txt`.
4. No launcher, ative o mod e mande **compilar/build**. O launcher usa Rust para aplicar o mod ao Horizon.
5. Entre no mundo, segure uma comida e observe a barra de fome. Troque, por exemplo, de pão para cenoura para ver a previsão mudar.

Não habilite outra versão do AppleSkin com o mesmo id ao mesmo tempo. Código nativo roda com as permissões do usuário; revise o fonte antes de instalar.

## Compilação pelo GitHub

O workflow `.github/workflows/build.yml` roda em push, pull request e manualmente:

- `cargo fmt --check`, `cargo clippy --locked -D warnings` e `cargo test --locked`;
- testes Python de empacotamento, pinos e hooks;
- geração determinística de `target/appleskin-0.2.0.hnmod` e SHA-256;
- download do fonte do Horizon no commit fixado, aplicação dos mixins e `cargo check` do `horizon-client` com o mod aplicado.

O job **Tests and package** publica o artefato instalável. O job **Compile into Horizon** verifica a compilação integrada e publica `compile-report.json`. A execução manual aceita outro commit/tag do Horizon ou `latest`; o padrão é o pin testado em `compatibility.json`.

### Compilar localmente

Requisitos: Python 3.11+, Rust 1.99.1 e dependências nativas do Horizon (`libasound2-dev`, `libudev-dev`, `pkg-config` em Ubuntu).

```sh
cargo test --locked
python3 -m unittest discover -s tests -v
python3 tools/native.py package
python3 tools/native.py check
```

Para compilar o cliente integrado, use `python3 tools/native.py build`. O jogo precisa dos recursos preparados do Minecraft separadamente; este projeto não redistribui assets nem binários do jogo.

## Pinos e créditos

- Horizon: `02c8607ee8ebd216f2f535c1eb7b62b572830df2`
- horizon-mixins: `cb1ad23022392d88f006cf94cbf6552a73f5df12`
- API mínima de hooks: `139`

Adaptação de [AppleSkin-Horizon](https://github.com/soufied/AppleSkin-Horizon), por @soufied, commit `20180ee8c7606d4cdc139211a02ca587ae2710fd`, sob MIT. Baseado no [AppleSkin](https://github.com/squeek502/AppleSkin), por squeek502, dedicado ao domínio público via Unlicense. Veja `LICENSE`, `LICENSE-APPLESKIN` e `NOTICE`.

Projeto independente, não é uma release oficial do AppleSkin nem do Horizon.
