#!/usr/bin/env python3
"""Generate PT-BR GitHub Release notes with versions, download and error report."""
from __future__ import annotations
import json, sys
from pathlib import Path

def main() -> None:
    build = json.loads(Path(sys.argv[1]).read_text())
    out = Path(sys.argv[2])
    out.parent.mkdir(parents=True, exist_ok=True)
    v = build["mod_version"]
    artifact = build["artifact"]
    url = f"https://github.com/jorginho11111/apple-skin-for-horizon/releases/download/v{v}/{artifact}"
    lines = [
        f"# AppleSkin {v} para Horizon Android",
        "",
        "Mod `.hmod` sandboxed para o port Android oficial do Horizon (`Vasysik/horizon-android`).",
        "Mostra uma prévia de fome em ícones, com opacidade pulsante, e o ganho potencial de saturação da comida segurada na mão principal.",
        "",
    ]
    if build.get("highlights"):
        lines += ["## Destaques", ""]
        lines += [f"- {h}" for h in build["highlights"]]
        lines += [""]
    lines += [
        "## Versões",
        "",
        f"- AppleSkin: **{v}**",
        f"- Minecraft: **{build['game']}**",
        f"- Horizon Android: **{build['horizon_android_release']}** (commit `{build['horizon_android_revision']}`)",
        f"- API de mods: **{build['mod_api']}** (WIT `horizon:mods@0.1.0`)",
        f"- Rust: **{build['rust']}** / alvo `{build['wasm_target']}`",
        f"- APKs compatíveis: **{', '.join(build['supported_apk_architectures'])}** (64 bits, Android {build.get('minimum_android', '9+')})",
        "",
        "## Download",
        "",
        f"1. Baixe [{artifact}]({url}).",
        f"2. Confira o SHA-256: `{build['sha256']}`.",
        "3. No Horizon Android, abra **Mods** e use **Import file** para selecionar o `.hmod`, ou cole a URL HTTPS acima e toque em **Import .hmod**.",
        "4. Abra **Installed**, ative **AppleSkin** e inicie o mundo.",
        "5. Segure uma comida na mão principal. A prévia aparece em uma linha fixa logo acima da barra de fome por padrão.",
        "",
        "Comandos: `/appleskin`, `/appleskin overlay`, `/appleskin pulse`, `/appleskin status`, `/appleskin compact`, `/appleskin position`, `/appleskin help`.",
        "Configurações: **Options > Mods > AppleSkin**.",
        "",
        "## O que a prévia mostra",
        "",
        "- Linha de ícones/meios ícones com a fome projetada após comer, em posição fixa logo acima da barra vanilla, com **opacidade pulsante** (sem movimento).",
        "- Faixa **amarela** sobre os ícones: ganho potencial de saturação do alimento, limitado pela fome projetada.",
        "- No Horizon v0.7.7 a prévia fica acima da barra porque o jogo desenha os ícones vanilla depois dos shapes de mods; uma sobreposição exata seria coberta.",
        "- Painel de texto opcional (`/appleskin overlay`), modo compacto e posição configurável.",
        "",
        "## Erros e limitações conhecidos",
        "",
    ]
    for lim in build["limitations"]:
        lines.append(f"- {lim}")
    lines += [
        "",
        "## Relatório de build",
        "",
        f"- Commit: `{build['ci']['commit']}`",
        f"- CI: [execução {build['ci']['run_id']}]({build['ci']['run_url']})",
        f"- Artefato: `{artifact}` ({build['size_bytes']} bytes)",
        f"- SHA-256: `{build['sha256']}`",
        "- Verificações: testes Python, `cargo fmt`, `cargo clippy`, `cargo test`, `cargo horizon-mod build`, `cargo horizon-mod check` e scripts de sandbox `cargo horizon-mod test`.",
        "- O arquivo `build.json` anexado contém versões, checksum e limitações; `check.json` contém a inspeção oficial do pacote.",
        "- Se algum desses passos falhar, a Release não é publicada automaticamente; o erro fica no GitHub Actions e deve ser corrigido antes do download.",
        "",
        "## Licenças",
        "",
        "- Este port: MIT (`LICENSE`).",
        "- AppleSkin original: Unlicense (`LICENSE-APPLESKIN`).",
        "- SDK Horizon vendored: MIT (`third_party/horizon-mod-sdk/LICENSE`).",
        "- Créditos e fontes: `NOTICE`.",
        "",
        "Projeto não oficial, sem afiliação com Mojang/Microsoft ou com o Horizon.",
    ]
    out.write_text("\n".join(lines) + "\n")
    print(f"wrote {out}")

if __name__ == "__main__":
    main()
