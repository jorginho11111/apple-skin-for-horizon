//! Geometry and pulse math for the hunger-bar overlay.
//!
//! The layout mirrors vanilla's HUD food row: ten 9x9 icons, right-to-left,
//! 8 pixels apart, with the right edge at `gui_width / 2 + 91` and the top at
//! `gui_height - 39`. Only opacity animates; positions never move.

/// One predicted hunger icon to draw over the vanilla row.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HungerIcon {
    pub x: f32,
    pub y: f32,
    /// `true` for a half drumstick, `false` for a full one.
    pub half: bool,
}

/// Yellow saturation rectangles, one per covered icon (partial last icon).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SatRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
}

pub const ICON: f32 = 9.0;
pub const PITCH: f32 = 8.0;

/// Vanilla `Hud` food-row anchor.
pub fn anchor(gui_w: f32, gui_h: f32) -> (f32, f32) {
    ((gui_w / 2.0).floor() + 91.0, gui_h - 39.0)
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

/// Icons that would gain hunger if the held food were eaten.
pub fn hunger_icons(before: u32, after: u32, x_right: f32, y: f32) -> Vec<HungerIcon> {
    let mut out = Vec::new();
    for n in 0..10u32 {
        let b = level(before, n);
        let a = level(after, n);
        if a > b {
            out.push(HungerIcon {
                x: x_right - n as f32 * PITCH - ICON,
                y,
                half: a == 1,
            });
        }
    }
    out
}

/// Potential saturation gain drawn as yellow overlays on the food row.
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
            y,
            w,
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
    fn bread_from_twelve_adds_two_and_a_half_icons() {
        let icons = hunger_icons(12, 17, 304.0, 201.0);
        assert_eq!(icons.len(), 3);
        assert!(!icons[0].half);
        assert!(!icons[1].half);
        assert!(icons[2].half);
        assert_eq!(icons[0].x, 304.0 - 6.0 * 8.0 - 9.0);
        assert_eq!(icons[2].x, 304.0 - 8.0 * 8.0 - 9.0);
        assert!(icons.iter().all(|i| i.y == 201.0));
    }

    #[test]
    fn no_icons_when_hunger_does_not_change() {
        assert!(hunger_icons(20, 20, 304.0, 201.0).is_empty());
        assert!(hunger_icons(17, 17, 304.0, 201.0).is_empty());
    }

    #[test]
    fn saturation_is_capped_by_projected_hunger() {
        // Golden carrot: +14.4 saturation, but only 6 hunger after eating.
        let rects = saturation_rects(144, 6, 304.0, 201.0);
        assert_eq!(rects.len(), 3);
        assert!(rects.iter().all(|r| (r.w - 9.0).abs() < f32::EPSILON));
    }

    #[test]
    fn partial_saturation_icon_aligns_to_the_right() {
        // +9.6 saturation => 4 full icons and 0.8 of the fifth.
        let rects = saturation_rects(96, 20, 304.0, 201.0);
        assert_eq!(rects.len(), 5);
        let last = rects[4];
        assert!((last.w - 7.2).abs() < 0.001);
        assert!((last.x - (304.0 - 4.0 * 8.0 - 9.0 + 1.8)).abs() < 0.001);
    }

    #[test]
    fn pulse_only_changes_opacity_between_bounds() {
        assert_eq!(pulse_alpha(0.0, 48, 136, 1.6), 48);
        assert_eq!(pulse_alpha(0.8, 48, 136, 1.6), 136);
        assert_eq!(pulse_alpha(1.6, 48, 136, 1.6), 48);
        for i in 0..=32 {
            let a = pulse_alpha(i as f32 * 0.05, 48, 136, 1.6);
            assert!((48..=136).contains(&a));
        }
    }
}
