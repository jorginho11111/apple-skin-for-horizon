//! Gain-only food preview and opacity math, independent of the Horizon SDK.
//!
//! Hunger selection and saturation thresholds follow AppleSkin's original
//! `HUDOverlayHandler` at 66fcdb8b12820c9b1cf3147abfdcf5530559f62b.
//! Horizon Android v0.7.7 draws mod shapes before the opaque vanilla food
//! icons, so this port places the preview in a fixed row above the food bar.
//! Only opacity animates; positions never move.
//!
//! Current saturation is not exposed by API 0.1. Yellow icons therefore show
//! the food's potential gain from the right edge, capped by projected hunger,
//! rather than the player's actual saturation or its true position on the bar.

use crate::saturation_icons::{self, PixelRun};

/// A hunger icon touched by the food's gain, not an icon already filled.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HungerIcon {
    pub x: f32,
    pub y: f32,
    /// `true` for a half drumstick, `false` for a full one.
    pub half: bool,
}

/// The original AppleSkin yellow outlines have four discrete sizes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum SaturationIconKind {
    Quarter = 0,
    Half = 1,
    ThreeQuarter = 2,
    Full = 3,
}

impl SaturationIconKind {
    /// `tenths` is the saturation in this icon, from 0 to 20 tenths (2 points).
    /// Upstream thresholds: <=0.25, <=0.5, <1, >=1 of a two-point icon.
    pub fn for_tenths(tenths: u32) -> Option<Self> {
        match tenths {
            0 => None,
            1..=5 => Some(Self::Quarter),
            6..=10 => Some(Self::Half),
            11..=19 => Some(Self::ThreeQuarter),
            _ => Some(Self::Full),
        }
    }

    pub fn runs(self) -> &'static [PixelRun] {
        saturation_icons::ICONS[self as usize]
    }
}

/// A yellow saturation outline drawn over the preview row.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SaturationIcon {
    pub x: f32,
    pub y: f32,
    pub kind: SaturationIconKind,
}

pub const ICON: f32 = 9.0;
pub const PITCH: f32 = 8.0;
/// Fixed vertical offset from the vanilla food row to the preview row.
pub const PREVIEW_ROW_OFFSET: f32 = 10.0;
/// At most 10 empty backgrounds + 10 hunger sprites + 10 full yellow outlines
/// (16 pixel runs each), below the host's 1024-shape limit.
pub const MAX_SHAPES: usize = 180;

/// Vanilla `Hud` food-row anchor.
pub fn anchor(gui_w: f32, gui_h: f32) -> (f32, f32) {
    ((gui_w / 2.0).floor() + 91.0, gui_h - 39.0)
}

/// Compatible preview anchor: same right edge, fixed row above the food bar.
pub fn preview_anchor(gui_w: f32, gui_h: f32) -> (f32, f32) {
    let (x_right, y) = anchor(gui_w, gui_h);
    (x_right, y - PREVIEW_ROW_OFFSET)
}

fn icon_x(x_right: f32, n: u32) -> f32 {
    x_right - n as f32 * PITCH - ICON
}

/// Only the hunger icons restored by this food, using upstream's selection.
///
/// As in AppleSkin, an existing half icon is redrawn as full when it gains a
/// point. For example, 15 -> 17 draws a full icon at the old half-icon slot
/// and a half at the next slot; it does not repeat the seven full icons.
pub fn restored_icons(before: u32, after: u32, x_right: f32, y: f32) -> Vec<HungerIcon> {
    let before = before.min(20);
    let after = after.min(20);
    if after <= before {
        return Vec::new();
    }
    let start = before / 2;
    let end = after.div_ceil(2);
    (start..end)
        .map(|n| HungerIcon {
            x: icon_x(x_right, n),
            y,
            half: n * 2 + 1 == after,
        })
        .collect()
}

/// Potential saturation gain as original yellow meat outlines, right-aligned.
///
/// This is a gain preview from an assumed zero baseline, not current saturation.
/// The hidden player's saturation could shift or reduce the actual gained range.
pub fn potential_saturation_icons(
    gain_tenths: u32,
    hunger_after: u32,
    x_right: f32,
    y: f32,
) -> Vec<SaturationIcon> {
    let mut remaining = gain_tenths.min(hunger_after.saturating_mul(10)).min(200);
    let mut out = Vec::new();
    for n in 0..10u32 {
        if remaining == 0 {
            break;
        }
        let take = remaining.min(20);
        remaining -= take;
        if let Some(kind) = SaturationIconKind::for_tenths(take) {
            out.push(SaturationIcon {
                x: icon_x(x_right, n),
                y,
                kind,
            });
        }
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
    fn bread_from_twelve_draws_only_two_and_a_half_gained_icons() {
        let icons = restored_icons(12, 17, 304.0, 191.0);
        assert_eq!(icons.len(), 3);
        assert!(!icons[0].half);
        assert!(!icons[1].half);
        assert!(icons[2].half);
        assert_eq!(icons[0].x, 304.0 - 6.0 * 8.0 - 9.0);
        assert_eq!(icons[2].x, 304.0 - 8.0 * 8.0 - 9.0);
        assert!(icons.iter().all(|i| i.y == 191.0));
    }

    #[test]
    fn even_hunger_does_not_repeat_the_current_full_icons() {
        let icons = restored_icons(16, 18, 304.0, 191.0);
        assert_eq!(icons.len(), 1);
        assert!(!icons[0].half);
        assert_eq!(icons[0].x, 304.0 - 8.0 * 8.0 - 9.0);
    }

    #[test]
    fn odd_hunger_follows_appleskins_partial_icon_convention() {
        let icons = restored_icons(15, 17, 304.0, 191.0);
        assert_eq!(icons.len(), 2);
        assert!(!icons[0].half);
        assert!(icons[1].half);
        assert_eq!(icons[0].x, 304.0 - 7.0 * 8.0 - 9.0);
        assert_eq!(icons[1].x, 304.0 - 8.0 * 8.0 - 9.0);
    }

    #[test]
    fn hunger_gain_is_capped_at_twenty() {
        let icons = restored_icons(18, 20, 304.0, 191.0);
        assert_eq!(icons.len(), 1);
        assert!(!icons[0].half);
        assert_eq!(icons[0].x, 304.0 - 9.0 * 8.0 - 9.0);
    }

    #[test]
    fn no_hunger_icons_when_nothing_is_restored() {
        for (before, after) in [(20, 20), (17, 17), (0, 0), (19, 18), (25, 20)] {
            assert!(restored_icons(before, after, 304.0, 191.0).is_empty());
        }
    }

    #[test]
    fn saturation_thresholds_match_the_original_handler() {
        use SaturationIconKind::*;
        for (tenths, expected) in [
            (0, None),
            (1, Some(Quarter)),
            (4, Some(Quarter)),
            (5, Some(Quarter)),
            (6, Some(Half)),
            (10, Some(Half)),
            (11, Some(ThreeQuarter)),
            (19, Some(ThreeQuarter)),
            (20, Some(Full)),
            (200, Some(Full)),
        ] {
            assert_eq!(SaturationIconKind::for_tenths(tenths), expected);
        }
    }

    #[test]
    fn tiny_salmon_gain_still_gets_the_smallest_yellow_outline() {
        let icons = potential_saturation_icons(4, 18, 304.0, 191.0);
        assert_eq!(icons.len(), 1);
        assert_eq!(icons[0].kind, SaturationIconKind::Quarter);
        assert_eq!(icons[0].x, 304.0 - 9.0);
    }

    #[test]
    fn saturation_is_capped_by_projected_hunger() {
        let icons = potential_saturation_icons(144, 6, 304.0, 191.0);
        assert_eq!(icons.len(), 3);
        assert!(icons.iter().all(|i| i.kind == SaturationIconKind::Full));
    }

    #[test]
    fn partial_saturation_uses_the_original_three_quarter_outline() {
        let icons = potential_saturation_icons(96, 20, 304.0, 191.0);
        assert_eq!(icons.len(), 5);
        assert!(
            icons[..4]
                .iter()
                .all(|i| i.kind == SaturationIconKind::Full)
        );
        assert_eq!(icons[4].kind, SaturationIconKind::ThreeQuarter);
        assert_eq!(icons[4].x, 304.0 - 4.0 * 8.0 - 9.0);
    }

    #[test]
    fn yellow_icons_can_show_saturation_only_when_hunger_is_full() {
        assert!(restored_icons(20, 20, 304.0, 191.0).is_empty());
        assert_eq!(potential_saturation_icons(96, 20, 304.0, 191.0).len(), 5);
        assert!(potential_saturation_icons(0, 20, 304.0, 191.0).is_empty());
        assert!(potential_saturation_icons(96, 0, 304.0, 191.0).is_empty());
    }

    #[test]
    fn generated_outlines_are_valid_nested_meat_shapes() {
        use std::collections::HashSet;
        let masks: Vec<HashSet<(u8, u8)>> = saturation_icons::ICONS
            .iter()
            .map(|runs| {
                let mut pixels = HashSet::new();
                for run in *runs {
                    assert!(run.x + run.w <= 9);
                    assert!(run.y < 9);
                    assert!(run.w > 0);
                    assert!(run.rgb <= 0xFF_FFFF);
                    for dx in 0..run.w {
                        assert!(pixels.insert((run.x + dx, run.y)));
                    }
                }
                pixels
            })
            .collect();
        let run_counts: Vec<usize> = saturation_icons::ICONS
            .iter()
            .map(|runs| runs.len())
            .collect();
        assert_eq!(run_counts, [7, 11, 12, 16]);
        let pixel_counts: Vec<usize> = masks.iter().map(HashSet::len).collect();
        assert_eq!(pixel_counts, [9, 14, 17, 21]);
        for pair in masks.windows(2) {
            assert!(pair[0].is_subset(&pair[1]));
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
        let hunger = restored_icons(12, 17, x, y);
        let saturation = potential_saturation_icons(60, 17, x, y);
        for phase in [0.0, 0.4, 0.8, 1.2, 1.6] {
            let _alpha = pulse_alpha(phase, 90, 220, 1.6);
            assert_eq!(preview_anchor(427.0, 240.0), (x, y));
            assert_eq!(restored_icons(12, 17, x, y), hunger);
            assert_eq!(potential_saturation_icons(60, 17, x, y), saturation);
        }
    }

    #[test]
    fn worst_case_fits_the_host_shape_budget() {
        let hunger = restored_icons(0, 20, 304.0, 191.0);
        let saturation = potential_saturation_icons(200, 20, 304.0, 191.0);
        let shapes = hunger.len() * 2
            + saturation
                .iter()
                .map(|i| i.kind.runs().len())
                .sum::<usize>();
        assert_eq!(shapes, 180);
        assert_eq!(shapes, MAX_SHAPES);
        assert!(shapes < 1024);
    }
}
