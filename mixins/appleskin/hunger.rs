use crate::game::Game;
use horizon_client::items::ItemInfo;

#[derive(Clone, Copy, Debug)]
pub struct NativeHunger {
    pub food: i32,
    pub saturation: f32,
    pub exhaustion: f32,
    pub exhaustion_visible: bool,
    pub health: f32,
    /// Nutrition and absolute saturation gain of the food the player can eat now.
    pub held: Option<(u32, f32)>,
    pub held_regeneration: Option<(i32, i32)>,
    pub natural_regeneration: bool,
    pub peaceful: bool,
    pub regeneration_blocked: bool,
}

impl Game<'_> {
    /// Reads the live hand stacks and the world's item table on every HUD frame.
    /// The active hand wins while eating; otherwise the selected hotbar slot wins.
    /// The off hand is a fallback only when it is enabled and the main food cannot be eaten.
    fn appleskin_food(&self, stack: &horizon_client::items::ItemStack) -> Option<&ItemInfo> {
        if stack.count == 0 {
            return None;
        }
        let info = self.items().get(&stack.item)?;
        if info.food.is_some() && info.consumable.is_some() && self.can_eat(stack, info.can_always_eat) {
            Some(info)
        } else {
            None
        }
    }

    pub(super) fn native_hunger(&self, offhand: bool) -> Option<NativeHunger> {
        let p = self.player.as_ref()?;
        let food = &p.health.food;
        let main = self.inventory.get(self.selected).and_then(|s| self.appleskin_food(s));
        let active = self.held().and_then(|s| self.appleskin_food(s));
        let secondary = if offhand {
            self.offhand.as_ref().and_then(|s| self.appleskin_food(s))
        } else {
            None
        };
        let held_item = active.or(main).or(secondary);
        let held = held_item.and_then(|info| info.food);
        let held_regeneration = held_item.and_then(|info| {
            info.consumable.as_ref().and_then(|consumable| {
                consumable.effects.iter().find_map(|effect| match effect {
                    horizon_client::items::ConsumeEffect::ApplyEffects { effects, .. } => effects
                        .iter()
                        .find(|e| {
                            horizon_client::brewing::effect(&e.id)
                                .is_some_and(|d| d.id == "regeneration")
                        })
                        .map(|e| (e.duration, e.amplifier)),
                    _ => None,
                })
            })
        });
        let rules = self.world.health_rules;
        let active = &self.potions.effects;
        Some(NativeHunger {
            food: food.food,
            saturation: food.saturation,
            exhaustion: food.exhaustion,
            // Vanilla does not synchronize exhaustion to a remote client.
            exhaustion_visible: !p.client_side,
            health: p.health.health(),
            held,
            held_regeneration,
            natural_regeneration: rules.natural_regeneration,
            peaceful: rules.difficulty == horizon_world::food::Difficulty::Peaceful,
            regeneration_blocked: ["poison", "wither", "regeneration"]
                .iter()
                .any(|id| active.get(id).is_some()),
        })
    }
}
