//! Geometry and pulse math for the compatible hunger preview.
//!
//! Horizon Android v0.7.7 draws mod HUD shapes *before* the vanilla health/food
//! rows, even though the WIT documentation says they are drawn over the HUD.
//! An exact overlay on the food icons is therefore covered by the opaque
//! vanilla sprites. The compatible preview is a fixed row 10 GUI pixels above
//! the vanilla food row. Only opacity animates; positions never move.

/// One projected hunger icon in the preview row.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HungerIcon {
    pub x: f32,
    pub y: f32,
    /// `true` for a half drumstick, `false` for a full one.
    pub half: bool,
}

/// Yellow saturation highlight over the preview row (partial last icon).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SatRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub const ICON: f32 = 9.0;
pub const PITCH: f32 = 8.0;
/// Fixed vertical offset from the vanilla food row to the preview row.
pub const PREVIEW_ROW_OFFSET: f32 = 10.0;
/// The yellow highlight is a band across the middle of the drumsticks, not a
/// full 9x9 rectangle, so it reads as saturation on top of the meat icons.
pub const SAT_BAND_HEIGHT: f32 = 3.0;
pub const SAT_BAND_TOP: f32 = 3.0;

/// Vanilla `Hud` food-row anchor.
pub fn anchor(gui_w: f32, gui_h: f32) -> (f32, f32) {
    ((gui_w / 2.0).floor() + 91.0, gui_h - 39.0)
}

/// Compatible preview anchor: same right edge, fixed row above the food bar.
pub fn preview_anchor(gui_w: f32, gui_h: f32) -> (f32, f32) {
    let (x_right, y) = anchor(gui_w, gui_h);
    (x_right, y - PREVIEW_ROW_OFFSET)
}

fn level(food: u32, icon: u32) -> u32 {
    let first = icon * 2 + 1;
    if first < food {
        2
    } else if first == food {
        1
    } else {
        0
    }
}

/// The projected food row after eating: full/half icons for `hunger_after`.
pub fn projected_icons(hunger_after: u32, x_right: f32, y: f32) -> Vec<HungerIcon> {
    let mut out = Vec::new();
    for n in 0..10u32 {
        let a = level(hunger_after, n);
        if a > 0 {
            out.push(HungerIcon {
                x: x_right - n as f32 * PITCH - ICON,
                y,
                half: a == 1,
            });
        }
    }
    out
}

/// Potential saturation gain drawn as a yellow band over the preview row.
///
/// The Android API does not expose current saturation, so this is the food's
/// gain, capped by the projected hunger (vanilla saturation cannot exceed the
/// food level). It is not the player's current saturation.
pub fn saturation_rects(gain_tenths: u32, hunger_after: u32, x_right: f32, y: f32) -> Vec<SatRect> {
    let mut remaining = gain_tenths.min(hunger_after.saturating_mul(10)).min(200);
    let mut out = Vec::new();
    for n in 0..10u32 {
        if remaining == 0 {
            break;
        }
        let take = remaining.min(20);
        remaining -= take;
        let w = ICON * take as f32 / 20.0;
        let icon_x = x_right - n as f32 * PITCH - ICON;
        out.push(SatRect {
            x: icon_x + ICON - w,
            y: y + SAT_BAND_TOP,
            w,
            h: SAT_BAND_HEIGHT,
        });
    }
    out
}

/// Smooth opacity pulse: min at phase 0, max at half period, min again at end.
pub fn pulse_alpha(seconds: f32, min: u8, max: u8, period: f32) -> u8 {
    if period <= 0.0 || max <= min {
        return min;
    }
    let t = seconds.rem_euclid(period) / period;
    let wave = 0.5 - 0.5 * (core::f32::consts::TAU * t).cos();
    let value = min as f32 + (max - min) as f32 * wave;
    value.round().clamp(min as f32, max as f32) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchor_matches_vanilla_food_row() {
        assert_eq!(anchor(427.0, 240.0), (304.0, 201.0));
    }

    #[test]
    fn preview_row_is_fixed_above_vanilla() {
        let (x, y) = preview_anchor(427.0, 240.0);
        assert_eq!((x, y), (304.0, 191.0));
        assert_eq!(anchor(427.0, 240.0).1 - y, PREVIEW_ROW_OFFSET);
    }

    #[test]
    fn projected_row_after_bread_from_twelve() {
        // 12 + 5 = 17: eight full icons and one half icon.
        let icons = projected_icons(17, 304.0, 191.0);
        assert_eq!(icons.len(), 9);
        assert!(icons[..8].iter().all(|i| !i.half));
        assert!(icons[8].half);
        assert_eq!(icons[0].x, 304.0 - 9.0);
        assert_eq!(icons[8].x, 304.0 - 8.0 * 8.0 - 9.0);
        assert!(icons.iter().all(|i| i.y == 191.0));
    }

    #[test]
    fn projected_row_full_and_empty() {
        let full = projected_icons(20, 304.0, 191.0);
        assert_eq!(full.len(), 10);
        assert!(full.iter().all(|i| !i.half));
        assert!(projected_icons(0, 304.0, 191.0).is_empty());
    }

    #[test]
    fn saturation_is_capped_by_projected_hunger() {
        // Golden carrot: +14.4 saturation, but only 6 hunger after eating.
        let rects = saturation_rects(144, 6, 304.0, 191.0);
        assert_eq!(rects.len(), 3);
        assert!(rects.iter().all(|r| (r.w - 9.0).abs() < f32::EPSILON));
    }

    #[test]
    fn partial_saturation_icon_aligns_to_the_right() {
        // +9.6 saturation => 4 full icons and 0.8 of the fifth.
        let rects = saturation_rects(96, 20, 304.0, 191.0);
        assert_eq!(rects.len(), 5);
        let last = rects[4];
        assert!((last.w - 7.2).abs() < 0.001);
        assert!((last.x - (304.0 - 4.0 * 8.0 - 9.0 + 1.8)).abs() < 0.001);
    }

    #[test]
    fn saturation_band_stays_inside_the_icon() {
        let rects = saturation_rects(60, 20, 304.0, 191.0);
        for r in rects {
            assert_eq!(r.h, SAT_BAND_HEIGHT);
            assert_eq!(r.y, 191.0 + SAT_BAND_TOP);
            assert!(r.y + r.h <= 191.0 + ICON);
        }
    }

    #[test]
    fn pulse_only_changes_opacity_between_bounds() {
        assert_eq!(pulse_alpha(0.0, 90, 220, 1.6), 90);
        assert_eq!(pulse_alpha(0.8, 90, 220, 1.6), 220);
        assert_eq!(pulse_alpha(1.6, 90, 220, 1.6), 90);
        for i in 0..=32 {
            let a = pulse_alpha(i as f32 * 0.05, 90, 220, 1.6);
            assert!((90..=220).contains(&a));
        }
    }

    #[test]
    fn geometry_does_not_depend_on_pulse_phase() {
        let (x, y) = preview_anchor(427.0, 240.0);
        let icons = projected_icons(17, x, y);
        let rects = saturation_rects(60, 17, x, y);
        for phase in [0.0, 0.4, 0.8, 1.2, 1.6] {
            let _alpha = pulse_alpha(phase, 90, 220, 1.6);
            assert_eq!(preview_anchor(427.0, 240.0), (x, y));
            assert_eq!(projected_icons(17, x, y), icons);
            assert_eq!(saturation_rects(60, 17, x, y), rects);
        }
    }
}
