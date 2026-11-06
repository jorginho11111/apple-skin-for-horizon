//! Pure preview math, kept separate from the Horizon SDK so it can be tested
//! on the host without a WebAssembly runtime.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FoodInfo {
    pub nutrition: u32,
    /// Absolute saturation gain reported by vanilla 26.3, in tenths.
    pub saturation_tenths: u32,
    pub can_always_eat: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Preview {
    pub hunger_before: u32,
    pub hunger_after: u32,
    pub hunger_gain: u32,
    /// Saturation gain *of the food*. The Android mod API does not expose the
    /// player's current saturation, so the exact final saturation cannot be
    /// computed; this is the value the food would add before that hidden cap.
    pub saturation_gain_tenths: u32,
}

/// Vanilla food level is 0..=20. A food can be eaten when hungry, or when it
/// has `can_always_eat` (golden apples, chorus fruit, honey, suspicious stew).
pub fn compute(hunger: u32, count: u32, food: FoodInfo) -> Option<Preview> {
    if count == 0 {
        return None;
    }
    let hunger_before = hunger.min(20);
    if hunger_before >= 20 && !food.can_always_eat {
        return None;
    }
    let hunger_after = hunger_before.saturating_add(food.nutrition).min(20);
    Some(Preview {
        hunger_before,
        hunger_after,
        hunger_gain: hunger_after - hunger_before,
        saturation_gain_tenths: food.saturation_tenths,
    })
}

pub fn tenths_string(tenths: u32) -> String {
    format!("{}.{:01}", tenths / 10, tenths % 10)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BREAD: FoodInfo = FoodInfo { nutrition: 5, saturation_tenths: 60, can_always_eat: false };
    const GOLDEN_APPLE: FoodInfo = FoodInfo { nutrition: 4, saturation_tenths: 96, can_always_eat: true };

    #[test]
    fn bread_projects_hunger_and_saturation() {
        let p = compute(12, 1, BREAD).unwrap();
        assert_eq!(p.hunger_before, 12);
        assert_eq!(p.hunger_after, 17);
        assert_eq!(p.hunger_gain, 5);
        assert_eq!(p.saturation_gain_tenths, 60);
    }

    #[test]
    fn hunger_caps_at_twenty() {
        let p = compute(18, 3, BREAD).unwrap();
        assert_eq!(p.hunger_after, 20);
        assert_eq!(p.hunger_gain, 2);
    }

    #[test]
    fn full_player_cannot_eat_normal_food() {
        assert!(compute(20, 1, BREAD).is_none());
    }

    #[test]
    fn always_edible_food_works_when_full() {
        let p = compute(20, 1, GOLDEN_APPLE).unwrap();
        assert_eq!(p.hunger_gain, 0);
        assert_eq!(p.saturation_gain_tenths, 96);
    }

    #[test]
    fn empty_stack_has_no_preview() {
        assert!(compute(5, 0, BREAD).is_none());
    }

    #[test]
    fn saturation_formats_as_one_decimal() {
        assert_eq!(tenths_string(60), "6.0");
        assert_eq!(tenths_string(144), "14.4");
        assert_eq!(tenths_string(4), "0.4");
    }
}
