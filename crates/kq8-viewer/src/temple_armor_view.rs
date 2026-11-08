//! Inventory rule 8: list refresh is independent of weapon assignment.
use super::*;

fn carried_type(world: &kq8_core::active::WorldState, mode: WeaponMode) -> Option<&str> {
    let category = match mode {
        WeaponMode::Hand => "ShortWeapon",
        WeaponMode::Ranged => "LongWeapon",
        _ => return None,
    };
    world
        .items
        .objects
        .iter()
        .find(|item| {
            item.carried_by(kq8_core::talk::CONNOR)
                && world
                    .items
                    .kind(&item.kind)
                    .is_some_and(|kind| kind.category == category)
        })
        .map(|item| item.kind.as_str())
}

impl ConnorSim {
    /// Query the live table, not the startup late-file catalogue: repeated
    /// helmetize calls can replace any carried weapon's third-person entry.
    pub(crate) fn missing_inventory_banks(
        &self,
        world: &kq8_core::active::WorldState,
    ) -> Vec<String> {
        let Some(banks) = &world.connor_fight else {
            return Vec::new();
        };
        [
            banks.third_person.stowed.as_deref(),
            banks.bank(
                false,
                WeaponMode::Hand,
                carried_type(world, WeaponMode::Hand),
            ),
            banks.bank(
                false,
                WeaponMode::Ranged,
                carried_type(world, WeaponMode::Ranged),
            ),
        ]
        .into_iter()
        .flatten()
        .filter(|file| {
            !self.movements.iter().any(|movement| {
                !movement.first_person
                    && movement
                        .qualified
                        .split_once('/')
                        .is_some_and(|(loaded, _)| loaded.eq_ignore_ascii_case(file))
            })
        })
        .map(str::to_ascii_lowercase)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
    }

    /// Called after loading the required banks and before advancing Connor.
    /// A non-cut-short old model remains qualified and follows the ordinary
    /// movement request's end/transition gate, including across a checkpoint.
    pub(crate) fn refresh_inventory_lists(
        &mut self,
        world: &mut kq8_core::active::WorldState,
    ) -> Result<Option<String>, String> {
        let Some(_) = world
            .connor_fight
            .as_ref()
            .filter(|f| f.list_refresh_pending)
        else {
            return Ok(None);
        };
        let initialize = world.weapon_swapped && !self.readying;
        let mode = if initialize {
            world.weapon_mode
        } else {
            self.set_mode
        };
        if matches!(mode, WeaponMode::Raw(_)) {
            // Raw modes do not select another list. Table persistence still
            // changes, but the active bank and its movement/phase stay put.
            world
                .connor_fight
                .as_mut()
                .expect("pending bank exists")
                .list_refresh_pending = false;
            if initialize {
                world.weapon_swapped = false;
            }
            return Ok(Some("inventory mappings retained; raw mode keeps its active list and movement (inventory.md rule 8; ranged-weapons.md rule 9)".into()));
        }
        let target_mode = if initialize {
            world.weapon_mode
        } else {
            self.target_mode
        };
        let select = |mode: WeaponMode, retained: &Option<String>| {
            if matches!(mode, WeaponMode::Raw(_)) {
                retained.clone()
            } else {
                world.equipped_weapon_bank(world.first_person, mode)
            }
        };
        let next = select(mode, &self.weapon_set);
        let target = select(target_mode, &self.weapon_target);
        let pending = self
            .pending_set
            .as_ref()
            .map(|(file, mode, at)| (select(*mode, file), *mode, *at));
        for bank in [
            next.as_deref(),
            target.as_deref(),
            pending.as_ref().and_then(|(file, _, _)| file.as_deref()),
        ]
        .into_iter()
        .flatten()
        {
            if !self.movements.iter().any(|movement| {
                movement
                    .qualified
                    .split_once('/')
                    .is_some_and(|(loaded, _)| loaded.eq_ignore_ascii_case(bank))
            }) {
                return Err(format!("inventory list refresh waits for bank {bank}"));
            }
        }
        let action = kq8_core::connor::plain_movement_name(self.walker.movement);
        let old = self
            .drawn(self.walker.movement)
            .ok_or_else(|| {
                format!(
                    "inventory list refresh cannot resolve old movement {}",
                    self.walker.movement
                )
            })?
            .qualified;
        if find(&self.movements, next.as_deref(), action).is_none() {
            return Err(format!(
                "inventory list refresh cannot resolve {action} in {next:?}"
            ));
        }
        let queued = if let Some((file, _, at)) = &pending {
            if let Some(kq8_core::connor::SpecialStep::Play(movement, _)) =
                self.walker.special.get(*at)
            {
                let name = kq8_core::connor::plain_movement_name(movement);
                Some((
                    *at,
                    find(&self.movements, file.as_deref(), name)
                        .ok_or_else(|| {
                            format!("inventory list refresh cannot resolve queued {name}")
                        })?
                        .name,
                ))
            } else {
                None
            }
        } else {
            None
        };
        // These are list identities only. Preserve mode, swap-controller
        // progress and ordinary-assignment provenance; never call ready_set.
        self.weapon_set = next;
        self.weapon_target = target;
        self.ready_item = world.weapon_item(target_mode);
        self.pending_set = pending;
        if initialize {
            self.set_mode = mode;
            self.target_mode = target_mode;
            self.ready_item = world.weapon_ready.clone();
            self.walker.hand_mode = if mode == WeaponMode::None {
                kq8_core::connor::EMPTY_HANDS
            } else {
                mode.number()
            };
        }
        if !matches!(self.set_mode, WeaponMode::Raw(_)) {
            self.first_person = world.first_person;
        }
        if let Some((at, selected)) = queued {
            if let Some(kq8_core::connor::SpecialStep::Play(movement, _)) =
                self.walker.special.get_mut(at)
            {
                *movement = selected;
            }
        }
        let movements = &self.movements;
        let bank = self.weapon_set.as_deref();
        let loops = |name: &str| find(movements, bank, name).map(|m| &m.movement_loop);
        self.walker.animation_input_lock = false;
        let notes = self.walker.rerequest_view(old, action, &loops);
        let selected_attack = |movement: &'static str, waiting: bool| {
            if waiting {
                find(
                    movements,
                    bank,
                    kq8_core::connor::plain_movement_name(movement),
                )
                .map_or(movement, |m| m.name)
            } else if movement.eq_ignore_ascii_case(old) {
                self.walker.movement
            } else {
                movement
            }
        };
        if let Some(swing) = &mut self.swing {
            swing.movement = selected_attack(swing.movement, swing.waiting);
        }
        if let Some(shot) = &mut self.shot {
            shot.movement = selected_attack(shot.movement, shot.waiting);
        }
        world
            .connor_fight
            .as_mut()
            .expect("pending bank exists")
            .list_refresh_pending = false;
        if initialize {
            world.weapon_swapped = false;
        }
        self.redraw_from_walker();
        Ok(Some(format!("inventory animation lists refreshed: bank {:?}; re-request {action} from {old}: {} (inventory.md rule 8)", self.weapon_set, notes.join("; "))))
    }
}

/// Root schedules this immediately after the post-door late-set loader.
pub(crate) fn inventory_list_step(
    connor: Option<ResMut<ConnorSim>>,
    sim: Option<ResMut<crate::door_view::DoorSim>>,
) {
    let (Some(mut connor), Some(mut sim)) = (connor, sim) else {
        return;
    };
    match connor.refresh_inventory_lists(&mut sim.active.world) {
        Ok(Some(line)) | Err(line) => sim.active.world.fight_note("Connor", line),
        Ok(None) => {}
    }
}

#[cfg(test)]
#[path = "temple_armor_view_tests.rs"]
mod tests;
