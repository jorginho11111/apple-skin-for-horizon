//! AppleSkin for Horizon Android (`.hmod`, sandboxed WebAssembly component).
//!
//! Reads the local player's hunger and the main-hand stack through the official
//! Horizon Android mod API, then draws a small preview panel near the hunger
//! bar. The API does not expose current saturation/exhaustion or the off hand,
//! so the panel shows projected hunger and the held food's saturation gain.

mod foods;
mod preview;

use horizon_mod_sdk::{
    Mod, Registration, game, input,
    render::{self, FillShape, ItemShape, Point, Rect, Shape, TextShape},
    storage,
    ui::{self, ButtonWidget, TextWidget, Widget},
};
use preview::Preview;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const BG: u32 = 0xB0_10_10_10;
const HUNGER_COLOR: u32 = 0xFF_C8_5A;
const SAT_COLOR: u32 = 0xFF_E0_8A;
const TEXT_COLOR: u32 = 0xFF_FF_FF;

const TOGGLE_BUTTON: u32 = 1;
const COMPACT_BUTTON: u32 = 2;
const POSITION_BUTTON: u32 = 3;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Position {
    HungerBar,
    TopLeft,
    TopRight,
}

impl Position {
    const ALL: [Position; 3] = [Position::HungerBar, Position::TopLeft, Position::TopRight];

    fn key(self) -> &'static str {
        match self {
            Position::HungerBar => "hunger_bar",
            Position::TopLeft => "top_left",
            Position::TopRight => "top_right",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Position::HungerBar => "Acima da barra de fome",
            Position::TopLeft => "Canto superior esquerdo",
            Position::TopRight => "Canto superior direito",
        }
    }

    fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.key() == key)
    }
}

struct AppleSkin {
    enabled: bool,
    compact: bool,
    position: Position,
    /// Last drawn signature, so an unchanged panel costs no host call.
    last: String,
}

impl AppleSkin {
    fn load() -> Self {
        let bool_key = |key: &str, default: bool| {
            storage::get(key)
                .and_then(|v| String::from_utf8(v).ok())
                .map(|v| v.trim() == "true")
                .unwrap_or(default)
        };
        let position = storage::get("position")
            .and_then(|v| String::from_utf8(v).ok())
            .and_then(|v| Position::from_key(v.trim()))
            .unwrap_or(Position::HungerBar);
        Self {
            enabled: bool_key("enabled", true),
            compact: bool_key("compact", false),
            position,
            last: String::new(),
        }
    }

    fn save(&self) {
        let put = |key: &str, value: &str| {
            if let Err(err) = storage::set(key, value.as_bytes()) {
                horizon_mod_sdk::warn(&format!("não foi possível salvar {key}: {err}"));
            }
        };
        put("enabled", if self.enabled { "true" } else { "false" });
        put("compact", if self.compact { "true" } else { "false" });
        put("position", self.position.key());
    }

    fn page(&self) {
        let on = |b: bool| if b { "Ligado" } else { "Desligado" };
        ui::set_settings(&[
            Widget::Text(TextWidget {
                id: 0,
                text: format!("AppleSkin {VERSION} para Horizon Android 0.7.7 / Minecraft 26.3."),
                color: TEXT_COLOR,
            }),
            Widget::Button(ButtonWidget {
                id: TOGGLE_BUTTON,
                label: format!("Prévia: {}", on(self.enabled)),
            }),
            Widget::Button(ButtonWidget {
                id: COMPACT_BUTTON,
                label: format!("Modo compacto: {}", on(self.compact)),
            }),
            Widget::Button(ButtonWidget {
                id: POSITION_BUTTON,
                label: format!("Posição: {}", self.position.label()),
            }),
            Widget::Text(TextWidget {
                id: 4,
                text: "A API Android não expõe saturação atual, exaustão nem mão secundária. A prévia mostra a fome projetada e o ganho de saturação do alimento segurado.".into(),
                color: 0xAA_AA_AA,
            }),
        ]);
    }

    fn clear(&mut self) {
        if !self.last.is_empty() {
            render::set_hud(&[]);
            self.last.clear();
        }
    }

    fn draw(&mut self, item_id: &str, p: Preview) {
        let (w, h) = render::gui_size();
        let hunger_line = if p.hunger_gain > 0 {
            format!(
                "Fome {} -> {} (+{})",
                p.hunger_before, p.hunger_after, p.hunger_gain
            )
        } else {
            format!("Fome {} (cheia)", p.hunger_before)
        };
        let sat_line = format!(
            "Sat alimento: +{}",
            preview::tenths_string(p.saturation_gain_tenths)
        );
        let signature = format!(
            "{}|{}|{}|{}|{}|{:.0}x{:.0}",
            item_id,
            p.hunger_before,
            p.hunger_after,
            p.saturation_gain_tenths,
            self.compact as u8,
            w,
            h
        );
        if signature == self.last {
            return;
        }

        let (panel_w, panel_h, shapes) = if self.compact {
            let text = format!(
                "+{} fome | +{} sat",
                p.hunger_gain,
                preview::tenths_string(p.saturation_gain_tenths)
            );
            let width = render::text_width(&text) + 24.0;
            let shapes = vec![
                Shape::Fill(FillShape {
                    rect: Rect {
                        x: 0.0,
                        y: 0.0,
                        w: width,
                        h: 16.0,
                    },
                    color: BG,
                }),
                Shape::Item(ItemShape {
                    item: item_id.into(),
                    at: Point { x: 2.0, y: 0.0 },
                    scale: 1.0,
                }),
                Shape::Text(TextShape {
                    text,
                    at: Point { x: 20.0, y: 4.0 },
                    color: TEXT_COLOR,
                    centered: false,
                    shadow: true,
                }),
            ];
            (width, 16.0, shapes)
        } else {
            let width = render::text_width(&hunger_line).max(render::text_width(&sat_line)) + 24.0;
            let shapes = vec![
                Shape::Fill(FillShape {
                    rect: Rect {
                        x: 0.0,
                        y: 0.0,
                        w: width,
                        h: 26.0,
                    },
                    color: BG,
                }),
                Shape::Item(ItemShape {
                    item: item_id.into(),
                    at: Point { x: 2.0, y: 5.0 },
                    scale: 1.0,
                }),
                Shape::Text(TextShape {
                    text: hunger_line,
                    at: Point { x: 20.0, y: 3.0 },
                    color: HUNGER_COLOR,
                    centered: false,
                    shadow: true,
                }),
                Shape::Text(TextShape {
                    text: sat_line,
                    at: Point { x: 20.0, y: 14.0 },
                    color: SAT_COLOR,
                    centered: false,
                    shadow: true,
                }),
            ];
            (width, 26.0, shapes)
        };

        let (x, y) = match self.position {
            Position::HungerBar => (
                (w / 2.0 + 91.0 - panel_w).max(2.0),
                (h - 39.0 - panel_h - 4.0).max(2.0),
            ),
            Position::TopLeft => (4.0, 4.0),
            Position::TopRight => ((w - panel_w - 4.0).max(2.0), 4.0),
        };
        let placed: Vec<Shape> = shapes
            .into_iter()
            .map(|shape| match shape {
                Shape::Fill(mut s) => {
                    s.rect.x += x;
                    s.rect.y += y;
                    Shape::Fill(s)
                }
                Shape::Item(mut s) => {
                    s.at.x += x;
                    s.at.y += y;
                    Shape::Item(s)
                }
                Shape::Text(mut s) => {
                    s.at.x += x;
                    s.at.y += y;
                    Shape::Text(s)
                }
                other => other,
            })
            .collect();
        render::set_hud(&placed);
        self.last = signature;
    }

    fn status(&self) -> String {
        if !self.enabled {
            return "AppleSkin desligado. Use /appleskin para ligar.".into();
        }
        let Some(player) = game::get_player() else {
            return "AppleSkin ligado: entre em um mundo e segure uma comida.".into();
        };
        let Some(held) = input::held() else {
            return "AppleSkin ligado: segure uma comida na mão principal.".into();
        };
        match foods::lookup(&held.id).and_then(|f| preview::compute(player.food, held.count, f)) {
            Some(p) => format!(
                "AppleSkin ligado: fome {} -> {} (+{}), saturação do alimento +{}.",
                p.hunger_before,
                p.hunger_after,
                p.hunger_gain,
                preview::tenths_string(p.saturation_gain_tenths)
            ),
            None => "AppleSkin ligado: segure uma comida comestível na mão principal.".into(),
        }
    }
}

impl Mod for AppleSkin {
    fn init() -> Result<(Self, Registration), String> {
        let skin = Self::load();
        skin.page();
        Ok((skin, Registration::new().command("appleskin").on_tick()))
    }

    fn tick(&mut self, _tick: u64) {
        if !self.enabled {
            self.clear();
            return;
        }
        let Some(player) = game::get_player() else {
            self.clear();
            return;
        };
        let Some(held) = input::held().filter(|h| h.count > 0 && h.slot < 9) else {
            self.clear();
            return;
        };
        let Some(food) = foods::lookup(&held.id) else {
            self.clear();
            return;
        };
        let Some(p) = preview::compute(player.food, held.count, food) else {
            self.clear();
            return;
        };
        self.draw(&held.id, p);
    }

    fn command(&mut self, name: &str, args: &str) -> Result<Option<String>, String> {
        if name != "appleskin" {
            return Err("Comando desconhecido.".into());
        }
        match args.trim() {
            "" | "toggle" => {
                self.enabled = !self.enabled;
                self.save();
                self.clear();
                Ok(Some(
                    if self.enabled {
                        "AppleSkin ligado."
                    } else {
                        "AppleSkin desligado."
                    }
                    .into(),
                ))
            }
            "compact" => {
                self.compact = !self.compact;
                self.save();
                self.last.clear();
                Ok(Some(
                    if self.compact {
                        "Modo compacto ligado."
                    } else {
                        "Modo compacto desligado."
                    }
                    .into(),
                ))
            }
            "position" => {
                let idx = Position::ALL
                    .iter()
                    .position(|p| *p == self.position)
                    .unwrap_or(0);
                self.position = Position::ALL[(idx + 1) % Position::ALL.len()];
                self.save();
                self.last.clear();
                Ok(Some(format!("Posição: {}", self.position.label())))
            }
            "status" => Ok(Some(self.status())),
            "help" => Ok(Some(
                "Comandos: /appleskin [toggle|status|compact|position|help].".into(),
            )),
            other => Err(format!(
                "Argumento desconhecido: {other}. Use /appleskin help."
            )),
        }
    }

    fn ui_select(&mut self, id: u32, _row: u32) {
        match id {
            TOGGLE_BUTTON => self.enabled = !self.enabled,
            COMPACT_BUTTON => self.compact = !self.compact,
            POSITION_BUTTON => {
                let idx = Position::ALL
                    .iter()
                    .position(|p| *p == self.position)
                    .unwrap_or(0);
                self.position = Position::ALL[(idx + 1) % Position::ALL.len()];
            }
            _ => return,
        }
        self.save();
        self.page();
        self.clear();
    }

    fn ui_settings_opened(&mut self) {
        self.page();
    }
}

horizon_mod_sdk::export_mod!(AppleSkin);

#[cfg(test)]
mod tests {
    use super::*;
    use preview::FoodInfo;

    #[test]
    fn vanilla_foods_are_present() {
        assert_eq!(
            foods::lookup("minecraft:bread"),
            Some(FoodInfo {
                nutrition: 5,
                saturation_tenths: 60,
                can_always_eat: false
            })
        );
        assert_eq!(
            foods::lookup("minecraft:golden_carrot")
                .unwrap()
                .saturation_tenths,
            144
        );
        assert!(
            foods::lookup("minecraft:golden_apple")
                .unwrap()
                .can_always_eat
        );
    }

    #[test]
    fn non_foods_and_fish_buckets_are_absent() {
        assert!(foods::lookup("minecraft:diamond").is_none());
        assert!(foods::lookup("minecraft:cod_bucket").is_none());
        assert!(foods::lookup("minecraft:cake").is_none());
    }

    #[test]
    fn position_keys_round_trip() {
        for p in Position::ALL {
            assert_eq!(Position::from_key(p.key()), Some(p));
        }
        assert_eq!(Position::from_key("bad"), None);
    }
}
