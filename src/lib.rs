//! AppleSkin for Horizon Android (`.hmod`, sandboxed WebAssembly component).
//!
//! Reads the local player's hunger and the main-hand stack through the official
//! Horizon Android mod API. By default it draws only the hunger icons restored
//! by the held food, with AppleSkin's original yellow saturation outlines.
//! The opacity-only pulse uses a fixed row above the vanilla food bar, because
//! Horizon v0.7.7 draws mod HUD shapes before the vanilla food icons. A text
//! panel remains available.
//!
//! The API does not expose current saturation/exhaustion, the off hand, or the
//! game mode, so the yellow overlay is the food's potential gain, not the
//! player's current saturation.

mod foods;
mod overlay;
mod preview;
mod saturation_icons;

use horizon_mod_sdk::{
    Mod, Registration, game, input,
    render::{self, FillShape, ItemShape, Point, Rect, Shape, SpriteShape, TextShape},
    storage,
    ui::{self, ButtonWidget, TextWidget, Widget},
};
use preview::Preview;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const BG: u32 = 0xB0_10_10_10;
const HUNGER_COLOR: u32 = 0xFF_C8_5A;
const SAT_COLOR: u32 = 0xFF_E0_8A;
const TEXT_COLOR: u32 = 0xFF_FF_FF;

/// Opacity-only pulse: positions never move.
const PULSE_MIN: u8 = 90;
const PULSE_MAX: u8 = 220;
const PULSE_PERIOD: f32 = 1.6;
const PULSE_OFF_ALPHA: u8 = 160;

const TOGGLE_BUTTON: u32 = 1;
const OVERLAY_BUTTON: u32 = 2;
const PULSE_BUTTON: u32 = 3;
const COMPACT_BUTTON: u32 = 4;
const POSITION_BUTTON: u32 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    /// Draw the icon preview above the hunger bar instead of a text panel.
    overlay: bool,
    /// Animate opacity only (no movement).
    pulse: bool,
    compact: bool,
    position: Position,
    phase: f32,
    /// Last drawn signature, so an unchanged frame costs no host call.
    last: String,
    current: Option<Preview>,
    current_item: String,
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
            overlay: bool_key("overlay", true),
            pulse: bool_key("pulse", true),
            compact: bool_key("compact", false),
            position,
            phase: 0.0,
            last: String::new(),
            current: None,
            current_item: String::new(),
        }
    }

    fn save(&self) {
        let put = |key: &str, value: &str| {
            if let Err(err) = storage::set(key, value.as_bytes()) {
                horizon_mod_sdk::warn(&format!("não foi possível salvar {key}: {err}"));
            }
        };
        put("enabled", if self.enabled { "true" } else { "false" });
        put("overlay", if self.overlay { "true" } else { "false" });
        put("pulse", if self.pulse { "true" } else { "false" });
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
                id: OVERLAY_BUTTON,
                label: format!("Prévia de ícones acima da barra: {}", on(self.overlay)),
            }),
            Widget::Button(ButtonWidget {
                id: PULSE_BUTTON,
                label: format!("Pulsação de opacidade: {}", on(self.pulse)),
            }),
            Widget::Button(ButtonWidget {
                id: COMPACT_BUTTON,
                label: format!("Modo compacto (texto): {}", on(self.compact)),
            }),
            Widget::Button(ButtonWidget {
                id: POSITION_BUTTON,
                label: format!("Posição (texto): {}", self.position.label()),
            }),
            Widget::Text(TextWidget {
                id: 6,
                text: "Somente a fome restaurada aparece na linha fixa acima da barra. Os contornos amarelos são o ganho potencial de saturação do alimento, não a saturação atual. Só a opacidade pulsa; os ícones não se movem.".into(),
                color: 0xAA_AA_AA,
            }),
        ]);
    }

    fn alpha(&self) -> u8 {
        if self.pulse {
            overlay::pulse_alpha(self.phase, PULSE_MIN, PULSE_MAX, PULSE_PERIOD)
        } else {
            PULSE_OFF_ALPHA
        }
    }

    fn reset(&mut self) {
        self.current = None;
        self.current_item.clear();
        if !self.last.is_empty() {
            render::set_hud(&[]);
            self.last.clear();
        }
    }

    fn draw_overlay(&mut self, p: Preview) {
        let (w, h) = render::gui_size();
        let alpha = self.alpha();
        let signature = format!(
            "gain-only-original-yellow|{}|{}|{}|{}|{:.0}x{:.0}",
            p.hunger_before, p.hunger_after, p.saturation_gain_tenths, alpha, w, h
        );
        if signature == self.last {
            return;
        }
        let (x_right, y) = overlay::preview_anchor(w, h);
        let mut shapes = Vec::with_capacity(overlay::MAX_SHAPES);
        // AppleSkin's hunger convention: show only gained slots, not the whole
        // projected food row. Empty backgrounds are 1/4 of foreground opacity.
        for icon in overlay::restored_icons(p.hunger_before, p.hunger_after, x_right, y) {
            let rect = Rect {
                x: icon.x,
                y: icon.y,
                w: overlay::ICON,
                h: overlay::ICON,
            };
            shapes.push(Shape::Sprite(SpriteShape {
                sprite: "hud/food_empty".into(),
                rect,
                color: ((u32::from(alpha) / 4) << 24) | 0x00FF_FFFF,
            }));
            shapes.push(Shape::Sprite(SpriteShape {
                sprite: if icon.half {
                    "hud/food_half".into()
                } else {
                    "hud/food_full".into()
                },
                rect,
                color: (u32::from(alpha) << 24) | 0x00FF_FFFF,
            }));
        }
        // Exact original yellow outlines, not a band or a tint of red meat.
        // The host draws this list in order, after the hunger sprites above.
        // API 0.1 has no current saturation: potential gain starts at the right.
        for icon in overlay::potential_saturation_icons(
            p.saturation_gain_tenths,
            p.hunger_after,
            x_right,
            y,
        ) {
            for run in icon.kind.runs() {
                shapes.push(Shape::Fill(FillShape {
                    rect: Rect {
                        x: icon.x + f32::from(run.x),
                        y: icon.y + f32::from(run.y),
                        w: f32::from(run.w),
                        h: 1.0,
                    },
                    color: (u32::from(alpha) << 24) | run.rgb,
                }));
            }
        }
        render::set_hud(&shapes);
        self.last = signature;
    }

    fn draw_text(&mut self, item_id: &str, p: Preview) {
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
            "text|{}|{}|{}|{}|{}|{:.0}x{:.0}",
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
        Ok((
            skin,
            Registration::new()
                .command("appleskin")
                .on_tick()
                .on_frame(),
        ))
    }

    fn tick(&mut self, _tick: u64) {
        if !self.enabled {
            self.reset();
            return;
        }
        let Some(player) = game::get_player() else {
            self.reset();
            return;
        };
        let Some(held) = input::held().filter(|h| h.count > 0 && h.slot < 9) else {
            self.reset();
            return;
        };
        let Some(food) = foods::lookup(&held.id) else {
            self.reset();
            return;
        };
        let Some(p) = preview::compute(player.food, held.count, food) else {
            self.reset();
            return;
        };
        self.current = Some(p);
        self.current_item = held.id;
        if self.overlay {
            self.draw_overlay(p);
        } else {
            let item = self.current_item.clone();
            self.draw_text(&item, p);
        }
    }

    fn frame(&mut self, seconds: f32) {
        if !self.enabled || !self.overlay || !self.pulse {
            return;
        }
        self.phase = (self.phase + seconds.max(0.0)).rem_euclid(PULSE_PERIOD);
        if let Some(p) = self.current {
            self.draw_overlay(p);
        }
    }

    fn command(&mut self, name: &str, args: &str) -> Result<Option<String>, String> {
        if name != "appleskin" {
            return Err("Comando desconhecido.".into());
        }
        match args.trim() {
            "" | "toggle" => {
                self.enabled = !self.enabled;
                self.save();
                self.reset();
                Ok(Some(
                    if self.enabled {
                        "AppleSkin ligado."
                    } else {
                        "AppleSkin desligado."
                    }
                    .into(),
                ))
            }
            "overlay" => {
                self.overlay = !self.overlay;
                self.save();
                self.last.clear();
                Ok(Some(
                    if self.overlay {
                        "Prévia de ícones acima da barra ligada."
                    } else {
                        "Prévia de ícones desligada (painel de texto)."
                    }
                    .into(),
                ))
            }
            "pulse" => {
                self.pulse = !self.pulse;
                self.save();
                self.last.clear();
                Ok(Some(
                    if self.pulse {
                        "Pulsação de opacidade ligada."
                    } else {
                        "Pulsação de opacidade desligada."
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
                "Comandos: /appleskin [toggle|overlay|pulse|status|compact|position|help].".into(),
            )),
            other => Err(format!(
                "Argumento desconhecido: {other}. Use /appleskin help."
            )),
        }
    }

    fn ui_select(&mut self, id: u32, _row: u32) {
        match id {
            TOGGLE_BUTTON => self.enabled = !self.enabled,
            OVERLAY_BUTTON => self.overlay = !self.overlay,
            PULSE_BUTTON => self.pulse = !self.pulse,
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
        self.reset();
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
