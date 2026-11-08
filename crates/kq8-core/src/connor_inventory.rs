//! Connor's inventory rules (M4 step 4c-3): an item going in (stacking,
//! swapping equipment), applying and un-applying items, using the bar's
//! slots, the description on a right click.
//!
//! Rules from `docs/behaviour/inventory.md` (A, 2026-10-18; check 4
//! done by the user in the original):
//!
//! - Places (rule 3): a full suit covers body armour, pants, gloves and
//!   boots, not the helmet; the chain-mail suit (`UnderArmor`) pants,
//!   gloves and boots; `GodArmor` has its own place; each of the weapon
//!   and armour categories is one place.
//! - Going in (rule 8): a stacking item (`HealthItem`, `Money`, `Rock`)
//!   whose type has `Comb` 1 joins one Connor already has (the new object
//!   is deleted); everything else is a new entry. Equipment first takes
//!   off what shares a place: placeholders are deleted, other items put
//!   into the world where the new one lay; then the new one is put on
//!   (applied).
//! - Applying (rule 14): only if not applied; a stacking item loses 1;
//!   its five numbers are added to Connor's stats through the stats
//!   event; `Dur` 0 stays applied; `Dur` > 0 runs a timer and is
//!   un-applied at its end; `Dur` −1 is un-applied at once without taking
//!   the numbers back. Un-applying (rule 15): for `Dur` ≥ 0 the numbers
//!   are taken back; a stacking item at 0 leaves the inventory. Taking an
//!   item out un-applies it first (rule 17).
//! - Applying a quest item first un-applies other applied quest items in
//!   Connor's inventory (rule 16). Holding one in the pointer is separate.
//! - A left click on a slot (rule 9): ignored during handsOff; quest
//!   items, money, rocks and the rope are held (4c-4); the mask piece
//!   opens the main menu; everything else is applied, with its `useSnd`
//!   if it wasn't applied already; a hand weapon is also readied (M5).
//! - A right click describes the item (rule 10): `talking.md` rule 20,
//!   the voice of `0.MSG` (N V C S of the type's message column), played
//!   as a plain sound, a new one stopping the last.
//!
//! The item in hand (M4 4c-4; `inventory.md` rules 7, 9, 12): a left
//! click on a quest item, the coins, the rocks or the rope makes it the
//! hand (its type's `cursor` is the pointer, and it leaves the quest
//! strip); the same item again puts it back. The scroll can't be taken:
//! a remark, at most every 3 s and not while Connor talks. A type with
//! cursor `none` does nothing. A world click with the hand: accepted →
//! the hand is emptied, except coins while `goldCoinPersists 1`; refused or
//! out of reach → it stays.
//!
//! STAND-INS: "applied" is kept in the item's `userId` (rule 13) as 0 for
//! Connor (the original keeps an object id there). An old piece taken off
//! by `give` drops at the new item's stored position: zero when omitted
//! (`inventory.md` rule 8, Q109 e). The potions'
//! timers: `crate::potions` (M5 5g-2).

use crate::active::WorldState;
use crate::item::Place;
use crate::stats::Stats;

/// Connor's `userId` while an item is applied to him (stand-in).
pub const CONNOR_USER_ID: i64 = 0;

/// The placeholders that are deleted when taken off (rule 8).
pub const PLACEHOLDERS: [&str; 3] = ["INVITEM_Fists", "INVITEM_NoRanged", "INVITEM_NoArmor"];

/// The eight slots' types, left to right (`interface.md` rule 38).
pub const SLOT_TYPES: [&str; 8] = [
    "INVITEM_Mushroom",
    "INVITEM_Crystal",
    "INVITEM_SacredWater",
    "INVITEM_ElixerOfLife",
    "INVITEM_Invulnerable",
    "INVITEM_Strength",
    "INVITEM_Clarity",
    "INVITEM_Invisible",
];

/// The categories that stack (rule 8).
const STACKING: [&str; 3] = ["HealthItem", "Money", "Rock"];

/// The places a category takes (rule 3); empty for non-equipment.
pub fn places(category: &str) -> &'static [&'static str] {
    match category {
        "ShortWeapon" => &["hand"],
        "LongWeapon" => &["ranged"],
        "Armor" => &["body"],
        "Gloves" => &["gloves"],
        "FootWear" => &["boots"],
        "Helmet" => &["helmet"],
        "Pants" => &["pants"],
        "UnderArmor" => &["pants", "gloves", "boots"],
        "FullArmor" => &["body", "pants", "gloves", "boots"],
        "GodArmor" => &["god"],
        _ => &[],
    }
}

pub fn stacks(category: &str) -> bool {
    STACKING.contains(&category)
}

/// A description voice to play (`talking.md` rule 20): the file, or
/// `None` to only stop the last one.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemVoice {
    pub path: Option<String>,
    pub why: String,
}

fn category(w: &WorldState, kind: &str) -> String {
    w.items.kind(kind).map_or(String::new(), |t| t.category.clone())
}

/// Connor's stats changed by `d` (`interface.md` rule 24: one stats
/// event); logged with before and after.
fn connor_add(w: &mut WorldState, d: Stats, why: &str) -> String {
    let Some(c) = w.connor_stats.as_mut() else { return "Connor's stats unknown".into() };
    let before = *c;
    let level = c.change(d);
    let after = *c;
    w.connor_stats_events += 1;
    format!(
        "{why}: stats {:?} level {} -> {:?} level {}{}",
        before.stats.0,
        before.level,
        after.stats.0,
        after.level,
        level.map_or(String::new(), |l| format!("; {l}"))
    )
}

/// Rule 8: item `name`, now an object, goes into Connor's inventory.
/// `lay_at`: where it lay in the world (for what it pushes out).
pub fn receive(w: &mut WorldState, name: &str, lay_at: Option<[f32; 3]>, why: &str) {
    let Some(i) = w.items.index(name) else { return };
    let o = w.items.objects[i].clone();
    if o.pending_delete || o.place == Place::Gone { return; }
    let Some(t) = w.items.kind(&o.kind).cloned() else { return };
    added_extras(w, &o.kind, &t.category);
    // What he had of this type, the new object left out (a give makes it
    // carried already).
    let before = w.items.count("Connor", &o.kind) - if o.carried_by("Connor") { o.quantity } else { 0 };
    // Stacking: joins the entry he has.
    if stacks(&t.category) && t.combine {
        if let Some(k) = w.items.objects.iter().position(|x| x.name != o.name && x.kind == o.kind && x.carried_by("Connor")) {
            w.items.objects[k].quantity += o.quantity;
            let into = w.items.objects[k].name.clone();
            w.carry_item(&into);
            w.request_item_deletion(name, why);
            w.item_note(format!("inventory ({why}): {} x{} joins {into}: {} {before} -> {}; {} deletion requested (inventory.md rule 8; traps.md Q203)", o.name, o.quantity, o.kind, before + o.quantity, o.name));
            return;
        }
    }
    w.carry_item(name);
    w.items.mark_added(name);
    let mut line = format!("inventory ({why}): {} ({}, {}) x{} is a new entry: {} {before} -> {}", o.name, o.kind, t.category, o.quantity, o.kind, before + o.quantity);
    let mine = places(&t.category);
    if !mine.is_empty() {
        // Rule 8: everything sharing a place comes off.
        let off: Vec<String> = w
            .items
            .objects
            .iter()
            .filter(|x| x.name != o.name && x.carried_by("Connor"))
            .filter(|x| places(&category(w, &x.kind)).iter().any(|p| mine.contains(p)))
            .map(|x| x.name.clone())
            .collect();
        for old in off {
            let kind = w.items.get(&old).map(|x| x.kind.clone()).unwrap_or_default();
            let undone = unapply(w, &old, "taken off");
            let k = w.items.index(&old).expect("listed");
            if PLACEHOLDERS.iter().any(|p| p.eq_ignore_ascii_case(&kind)) {
                w.request_item_deletion(&old, "placeholder replaced");
                line.push_str(&format!("; placeholder {old} ({kind}) deleted"));
            } else {
                let (at, whence) = match lay_at {
                    Some(p) => (p, "where the new one lay"),
                    None => (o.stored_pose().position, "at the new item's stored position (zero when omitted)"),
                };
                w.items.objects[k].place = Place::World { position: at, angles: [0.0; 3] };
                line.push_str(&format!("; {old} ({kind}) taken off and put into the world at ({:.0}, {:.0}, {:.0}) {whence}{undone}", at[0], at[1], at[2]));
            }
        }
        line.push_str(" (inventory.md rule 8)");
        w.item_note(line);
        let on = apply(w, name, "put on");
        w.item_note(on);
        // The user's check in the original (5d fix 10): a hand weapon taken
        // while one is ready is readied in its place.
        if t.category == "ShortWeapon" && w.weapon_mode == crate::weapons::WeaponMode::Hand {
            let was = w.weapon_ready.replace(o.kind.clone());
            w.weapon_swapped = true;
            w.item_note(format!("{}: a hand weapon was ready ({was:?}): {} is ready now (the user's check)", o.name, o.kind));
        }
        if t.category == "LongWeapon" {
            // ranged-weapons.md rules 17, 21 (M5 5f-4).
            let line = w.take_ranged_item(&o.kind);
            w.item_note(format!("{}: {line}", o.name));
            w.fight_note("Connor", format!("{} goes into his inventory: {line}", o.name));
            // "Edge cases": picked up with the ranged mode ready: the new
            // list at once, no swap.
            if w.weapon_mode == crate::weapons::WeaponMode::Ranged && !o.kind.eq_ignore_ascii_case(crate::weapons::NO_RANGED) {
                let was = w.weapon_ready.replace(o.kind.clone());
                w.weapon_swapped = true;
                w.fight_note("Connor", format!("the ranged mode was ready ({was:?}): {} is ready now, its list at once, no swap (ranged-weapons.md, Edge cases)", o.kind));
            }
        }
        if o.kind.eq_ignore_ascii_case("INVITEM_TempleArmor") {
            w.item_note("INVITEM_TempleArmor: Connor's animation set would change (contpsd2.anm, conicbw2.anm); not done");
        }
        return;
    }
    w.item_note(line + " (inventory.md rule 8)");
}

/// `potions.md` rules 40–41 (M5 5g-8): what Connor does when any item is
/// added, every time: built-in experience for some quest items (0.3 s
/// later or at once), and the original inventory help requests.
fn added_extras(w: &mut WorldState, kind: &str, category: &str) {
    if let Some(&(_, exp, later)) = crate::potions::PICKUP_EXPERIENCE.iter().find(|(k, ..)| k.eq_ignore_ascii_case(kind)) {
        if later {
            let due = w.now() + crate::potions::PICKUP_DELAY;
            w.pickup_experience.push((due, exp, kind.to_string()));
            w.item_note(format!("{kind} added: +{exp} experience due 0.3 s later, at game time {due:.3} (potions.md rules 40-41)"));
        } else {
            let line = connor_add(w, Stats([0, 0, 0, 0, exp]), &format!("{kind} added: +{exp} experience at once (potions.md rule 41)"));
            w.item_note(line);
        }
    }
    use crate::starting_help::{HEALTH_ITEM, QUEST_ITEM, ROPE};
    let (case, delay, bit) = match category {
        "HealthItem" => (131, 3.0, HEALTH_ITEM),
        "QuestItem" => (132, 3.0, QUEST_ITEM),
        "Rope" if kind.eq_ignore_ascii_case("INVITEM_RopeAndHook") => (128, 4.5, ROPE),
        _ => return,
    };
    if !w.help_prompts.consume(bit) {
        return;
    }
    let deadline = w.now() as f32 + delay;
    w.help_prompts.schedule_at(deadline, case);
    w.item_note(format!("{kind} added: inventory help case {case}, first eligible game time {deadline:.6}"));
}

/// Rules 14, 16: applies item `name` to Connor, replacing an applied
/// quest item first. Returns the log line.
pub fn apply(w: &mut WorldState, name: &str, why: &str) -> String {
    let Some(i) = w.items.index(name) else { return format!("apply {name}: no such item") };
    let o = w.items.objects[i].clone();
    let Some(t) = w.items.kind(&o.kind).cloned() else { return format!("apply {name}: unknown type {}", o.kind) };
    if o.third != -1 {
        return format!("apply {name} ({why}): already applied (userId {}); nothing (inventory.md rule 14)", o.third);
    }
    let mut replaced = String::new();
    if t.category == "QuestItem" {
        let old: Vec<_> = w.items.objects.iter()
            .filter(|x| x.carried_by("Connor") && x.third != -1)
            .filter(|x| category(w, &x.kind) == "QuestItem")
            .map(|x| x.name.clone())
            .collect();
        for old in old {
            let undone = unapply(w, &old, "another quest item applied");
            replaced.push_str(&format!("un-applied {old} first{undone} (inventory.md rule 16); "));
        }
    }
    // potions.md rule 13: a potion's effect switches on first.
    let (added, effect) = if t.duration > 0.0 { crate::potions::effect_on(w, crate::potions::Effect::of(&o.kind)) } else { (0, String::new()) };
    if stacks(&t.category) {
        w.items.objects[i].quantity -= 1;
    }
    w.items.objects[i].third = CONNOR_USER_ID;
    let stats = connor_add(w, Stats(t.stats), &format!("its numbers {:?} added", t.stats));
    let effect = if effect.is_empty() { effect } else { format!("{effect}; ") };
    let mut line = format!("apply {name} ({}, {why}): {replaced}{effect}x{} -> x{}; {stats}", o.kind, o.quantity, w.items.objects[i].quantity);
    if t.duration > 0.0 {
        // potions.md rule 33: whole seconds, counted from this update.
        let mut r = crate::potions::Running::new(name, &o.kind, t.duration.round() as i64, w.updates);
        r.added = added;
        line.push_str(&format!("; Dur {} s: the timer starts at {} s left (game time {:.3}), the next second off at update {} (potions.md rule 33)", t.duration, r.left, w.now(), r.next_tick));
        // Rule 35: its icon in the first free place.
        r.place = crate::potions::take_place(&mut w.status_icons, crate::potions::StatusIcon::Potion(name.to_string()));
        line.push_str(&match r.place {
            Some(k) => format!("; icon {} in IDCTL_TIMER_{}, 100 % (potions.md rule 35)", t.extra.as_deref().unwrap_or("none"), k + 1),
            None => "; all 7 icon places taken: no icon (potions.md rule 35)".into(),
        });
        w.potions.push(r);
    } else if t.duration < 0.0 {
        // Dur -1: un-applied at once, the numbers kept.
        w.items.objects[i].third = -1;
        line.push_str("; Dur -1: un-applied at once, the numbers kept");
        line.push_str(&remove_if_empty(w, name));
    } else {
        line.push_str("; Dur 0: stays applied while carried");
    }
    line + " (inventory.md rule 14)"
}

/// A stacking item at 0 leaves the inventory (rule 15).
fn remove_if_empty(w: &mut WorldState, name: &str) -> String {
    let Some(i) = w.items.index(name) else { return String::new() };
    let o = &w.items.objects[i];
    if o.quantity <= 0 && stacks(&category(w, &o.kind)) && o.carried_by("Connor") {
        w.request_item_deletion(name, "empty inventory entry");
        return format!("; count 0: {name} leaves the inventory");
    }
    String::new()
}

/// Rule 15: un-applies item `name`. Returns a note for the log ("" if it
/// wasn't applied).
pub fn unapply(w: &mut WorldState, name: &str, why: &str) -> String {
    let Some(i) = w.items.index(name) else { return String::new() };
    let o = w.items.objects[i].clone();
    if o.third == -1 {
        return String::new();
    }
    let Some(t) = w.items.kind(&o.kind).cloned() else { return String::new() };
    w.items.objects[i].third = -1;
    let running = w.potions.iter().position(|r| r.item == name).map(|k| w.potions.remove(k));
    let icon = crate::potions::free_place(&mut w.status_icons, &crate::potions::StatusIcon::Potion(name.to_string()));
    // potions.md rule 13: at the end the stats event, then the effect off.
    let back = Stats(t.stats.map(|v| -v));
    let stats = connor_add(w, back, &format!("{name} un-applied ({why}): its numbers taken back"));
    let effect = running.map(|r| crate::potions::effect_off(w, &r)).filter(|e| !e.is_empty()).map_or(String::new(), |e| format!("; {e}"));
    let icon = icon.map_or(String::new(), |k| format!("; icon place IDCTL_TIMER_{} freed (potions.md rule 35)", k + 1));
    format!("; {stats}{effect}{icon}{} (inventory.md rule 15)", remove_if_empty(w, name))
}

/// The potions' timers (`potions.md` rule 33): one second off each
/// running potion per second of game time; un-applies those over.
pub fn timers(w: &mut WorldState) {
    // potions.md rules 40-41: experience due for items added.
    let t = w.now();
    let (due, later): (Vec<_>, Vec<_>) = std::mem::take(&mut w.pickup_experience).into_iter().partition(|(at, ..)| *at <= t + 1e-9);
    w.pickup_experience = later;
    for (_, exp, kind) in due {
        let line = connor_add(w, Stats([0, 0, 0, 0, exp]), &format!("{kind} added 0.3 s ago: +{exp} experience (potions.md rules 40-41)"));
        w.item_note(line);
    }
    let now = w.updates;
    let mut over = Vec::new();
    let mut lines = Vec::new();
    for r in &mut w.potions {
        let (seconds, ended) = r.tick(now);
        for left in seconds {
            lines.push(format!("{} ({}): {left} s left of {}, {} % (potions.md rule 33)", r.item, r.kind, r.duration, left * 100 / r.duration.max(1)));
        }
        if ended {
            over.push(r.item.clone());
        }
    }
    for line in lines {
        w.item_note(line);
    }
    for name in over {
        let line = unapply(w, &name, "its time is up");
        w.item_note(format!("{name}: its time is up at game time {:.3}{line}", w.now()));
    }
}

/// The scroll's exact refusal tuple (`inventory.md` rule 9, Q110 a).
const SCROLL: &str = "INVITEM_Scroll";
const SCROLL_REMARK: kq8_data::msg::Tuple = kq8_data::msg::Tuple { file: 0, noun: 92, verb: 14, case: 0, seq: 1 };
const ROCK_REMARK: kq8_data::msg::Tuple = kq8_data::msg::Tuple { file: 0, noun: 0, verb: 0, case: 0, seq: 3 };
/// The rope away from a rope place (rule 9 writes (0, 0, 58, 37); read the
/// same way, Q110).
const ROPE_REMARK: kq8_data::msg::Tuple = kq8_data::msg::Tuple { file: 0, noun: 0, verb: 58, case: 37, seq: 1 };

/// `ropeandhook.md` rule 9: the camera looks this far above Connor when
/// the rope is taken at the top.
const ROPE_TOP_AIM: f32 = 1000.0;

/// The rope's first-person lock on or off (`ropeandhook.md` rules 9, 12,
/// 13, 26). Restore the view only if this rope interaction forced it off.
pub fn rope_lock(w: &mut WorldState, on: bool, why: &str) -> String {
    if w.rope_first_person_lock == on {
        return String::new();
    }
    w.rope_first_person_lock = on;
    if on {
        w.resume_after_rope = w.first_person;
        w.first_person = false;
    } else if std::mem::take(&mut w.resume_after_rope) {
        w.first_person = w.connor_at.is_some();
    }
    format!("; {why}: the first-person key {}, first person {}", if on { "locked" } else { "allowed again" }, w.first_person)
}

/// Rule 9: at most one scroll remark every 3 s.
const SCROLL_GAP: f64 = 3.0;

/// What a click on a bar slot did.
#[derive(Debug, Clone, PartialEq)]
pub enum BarResult {
    Ignored,
    /// Now the item in hand (4c-4).
    Hold,
    /// The item in hand went back (4c-4).
    PutBack,
    /// The main menu (the mask button).
    Menu,
    Applied,
    Described,
    Nothing,
}

/// Live controller ownership supplied by the bar frontend. This is not a
/// region inference or persistent state (`inventory.md` rule 9, Q110 a).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BarOwner {
    #[default]
    Ordinary,
    Tile,
    Tile2,
    Tile3,
    Scales,
}

/// An ordinary empty inventory cell is a selection, not a world-use
/// target. Its left click returns the previous held item (Q110 e).
pub fn empty_bar_click(w: &mut WorldState, hands_off: bool, how: &str) -> BarResult {
    if hands_off || w.connor_dying { return BarResult::Ignored; }
    if clear_bar_selection(w, how) { BarResult::PutBack } else { BarResult::Nothing }
}

fn clear_bar_selection(w: &mut WorldState, why: &str) -> bool {
    let Some(name) = w.hand.take() else { return false };
    // Generic clearing does not perform the rope's own put-back action.
    // Its independent Top view lock survives until the ordinary release.
    w.item_note(format!("bar selection ({why}): {name} returns to its inventory slot; hand cleared (inventory.md rule 9, Q110)"));
    true
}

/// An ordinary SwapWeapons assignment has happened, rather than merely
/// being requested (inventory.md 12a; ranged-weapons.md 6–8). Like item
/// application, this does not release the rope's independent view lock.
pub fn weapon_mode_assigned(w: &mut WorldState) -> bool {
    clear_bar_selection(w, "ordinary weapon mode/list assignment")
}

/// Rules 9–10: a click on the slot of item `name`; `left` or right;
/// `hands_off`: Connor's handsOff. Logged to `items.log`.
pub fn bar_click(w: &mut WorldState, name: &str, left: bool, hands_off: bool, how: &str) -> BarResult {
    bar_click_with_owner(w, name, left, hands_off, BarOwner::Ordinary, how)
}

/// The normal bar path with the actual viewer controller's ownership.
pub fn bar_click_with_owner(w: &mut WorldState, name: &str, left: bool, hands_off: bool, owner: BarOwner, how: &str) -> BarResult {
    let Some(o) = w.items.get(name).cloned() else {
        w.item_note(format!("bar click on {name} ({how}): no such item; nothing"));
        return BarResult::Nothing;
    };
    let Some(t) = w.items.kind(&o.kind).cloned() else { return BarResult::Nothing };
    let slot = SLOT_TYPES.iter().position(|s| s.eq_ignore_ascii_case(&o.kind)).map_or(String::new(), |k| format!("slot {}, ", k + 1));
    let head = format!("bar click {} on {name} ({slot}{}, {}, x{}; {how})", if left { "left" } else { "right" }, o.kind, t.category, o.quantity);
    if !left {
        // Rule 10, talking.md rule 20: the description, as a plain voice.
        let words: Vec<u8> = t.message.split_whitespace().filter_map(|v| v.parse().ok()).collect();
        let voice = match words[..] {
            [n, v, c, s] => {
                let tuple = kq8_data::msg::Tuple::new(0, n, v, c, s);
                let folder = w.talk.book.line(tuple).map(|(f, _)| f.to_string());
                let path = folder.and_then(|f| w.talk.voices.as_ref().and_then(|v| v(&f, tuple))).map(|v| v.path);
                w.item_voices.push(ItemVoice { path: path.clone(), why: format!("description of {}", o.kind) });
                format!("0.MSG {tuple}: {}", path.as_deref().unwrap_or("no voice file"))
            }
            _ => format!("message '{}' is not 4 numbers; nothing", t.message),
        };
        w.item_note(format!("{head}: describes it: {voice}; no caption, no ducking (talking.md rule 20)"));
        return BarResult::Described;
    }
    if hands_off {
        w.item_note(format!("{head}: refused (handsOff): nothing, no sound, no remark (inventory.md rule 9, potions.md rule 2)"));
        return BarResult::Ignored;
    }
    if w.connor_dying {
        w.item_note(format!("{head}: refused (Connor is dying or dead): nothing, no sound, no remark (potions.md rule 2)"));
        return BarResult::Ignored;
    }
    match t.category.as_str() {
        "QuestItem" | "Money" | "Rock" | "Rope" => hold(w, name, &t, owner, &head),
        "MaskPiece" => {
            w.item_note(format!("{head}: the mask button requests the main menu (interface.md rule 41)"));
            BarResult::Menu
        }
        _ => {
            // Applying a bar item uses its own category, never the held
            // item's world accept-list path. Even an already-running potion
            // clears the prior selection before its ordinary no-op.
            clear_bar_selection(w, how);
            let was = o.third != -1;
            if was && t.category == "HealthItem" {
                // potions.md rule 7: the same potion while it runs: no
                // sound, no count change, the timer untouched.
                w.item_note(format!("{head}: already running: nothing (no sound, count stays x{}, timer untouched; potions.md rule 7)", o.quantity));
                return BarResult::Nothing;
            }
            let line = apply(w, name, "slot used");
            // potions.md rule 4: the use sound only when it takes effect.
            let sound = match (&t.use_sound, was) {
                (Some(s), false) => {
                    w.sounds.push(crate::sound::Start::plain_one_shot(s, format!("use of {}", o.kind)));
                    format!("; use sound {s} (plain, not placed)")
                }
                (None, false) => "; no use sound".into(),
                (_, true) => "; already applied: no use sound".into(),
            };
            // ranged-weapons.md rule 1 (M5 5f-3): a weapon slot asks for
            // its mode column's mode; the viewer takes it (rule 5).
            let ready = match t.category.as_str() {
                "ShortWeapon" | "LongWeapon" => {
                    w.ask_weapon(crate::weapons::Ask::Slot { item: o.kind.clone(), column: t.mode.clone() }, how);
                    format!("; asks for its mode ({})", t.mode)
                }
                _ => String::new(),
            };
            w.item_note(format!("{head}: {line}{sound}{ready} (inventory.md rule 9)"));
            BarResult::Applied
        }
    }
}

/// `inventory.md` rule 9, `fighting.md` rule 39 (M5 5c-5): clicking the
/// hand-weapon slot readies that weapon (`INVITEM_Fists` too: Connor
/// readies his fists); **stand-in**: clicking it again while it is ready
/// puts it away (how a weapon is put away was not traced). Returns the log
/// text.
pub fn ready_hand_weapon(w: &mut WorldState, kind: &str) -> String {
    if w.weapon_ready.as_deref().is_some_and(|r| r.eq_ignore_ascii_case(kind)) {
        w.set_weapon(crate::weapons::WeaponMode::None, None);
        return format!("puts {kind} away (stand-in: a second click on a ready weapon)");
    }
    w.set_weapon(crate::weapons::WeaponMode::Hand, Some(kind.to_string()));
    format!("readies {kind}: a left click in the world now swings (fighting.md rule 39; the input side is a stand-in)")
}

/// The hand weapon Connor carries (the first carried `ShortWeapon`; the
/// placeholder `INVITEM_Fists` when he has no other), its type name.
pub fn hand_weapon(w: &WorldState) -> Option<String> {
    w.items.objects.iter().filter(|o| o.carried_by(crate::talk::CONNOR)).find(|o| w.items.types.find(&o.kind).is_some_and(|t| t.category == "ShortWeapon")).map(|o| o.kind.clone())
}

/// Rule 9: a left click on a quest item, coins, rocks or the rope.
fn hold(w: &mut WorldState, name: &str, t: &kq8_data::items::ItemType, owner: BarOwner, head: &str) -> BarResult {
    if t.category == crate::rope::ROPE_CATEGORY && w.connor_roping {
        // ropeandhook.md rule 8: ignored in the rope state.
        w.item_note(format!("{head}: Connor is in the rope state: ignored (ropeandhook.md rule 8)"));
        return BarResult::Ignored;
    }
    if t.category == "Rock" && matches!(owner, BarOwner::Tile | BarOwner::Tile3 | BarOwner::Scales) {
        let said = w.connor_remark(ROCK_REMARK, "no time to trifle with that");
        w.item_note(format!("{head}: refused by {owner:?}; previous held selection retained; {said} (inventory.md rule 9, Q110 a)"));
        return BarResult::Nothing;
    }
    if w.hand.as_deref().is_some_and(|h| h.eq_ignore_ascii_case(name)) {
        w.hand = None;
        let mut extra = String::new();
        if t.category == crate::rope::ROPE_CATEGORY {
            // Rules 8, 9: the first-person lock released; the camera stays
            // where it is (the user's check).
            extra = rope_lock(w, false, "the rope put back");
            extra += "; the camera stays where it is (rule 9, the user's check)";
            w.rope_note(format!("rope button: the rope was in hand: put back{extra} (rule 8)"));
        }
        w.item_note(format!("{head}: it is the item in hand: put back, the play pointer again{extra} (inventory.md rule 9)"));
        return BarResult::PutBack;
    }
    if t.name.eq_ignore_ascii_case(SCROLL) {
        clear_bar_selection(w, "scroll selected");
        let now = w.now();
        let recent = w.scroll_remark_at.is_some_and(|t| now - t < SCROLL_GAP);
        let line = if recent {
            "the scroll can't be taken; its remark was less than 3 s ago: nothing".to_string()
        } else if w.talk.is_talking(crate::talk::CONNOR) {
            "the scroll can't be taken; Connor is talking: no remark".to_string()
        } else {
            w.scroll_remark_at = Some(now);
            format!("the scroll can't be taken: {}", w.connor_remark(SCROLL_REMARK, "scroll"))
        };
        w.item_note(format!("{head}: {line} (inventory.md rule 9)"));
        return BarResult::Nothing;
    }
    let Some(cursor) = t.cursor.clone() else {
        w.item_note(format!("{head}: its type has cursor none: can't be held; nothing (inventory.md rule 9)"));
        return BarResult::Nothing;
    };
    let mut extra = String::new();
    if t.category == crate::rope::ROPE_CATEGORY {
        // ropeandhook.md rules 8, 9 (M6 6a-2; answers Q110 b).
        match w.rope.status {
            crate::rope::Status::None => {
                let said = w.connor_remark(ROPE_REMARK, "not near anything to climb");
                w.rope_note(format!("rope button at status none: \"I am not near anything to climb.\" ({said}); not held (rule 8)"));
                w.item_note(format!("{head}: not at a rope place: {said}; the rope is not held (ropeandhook.md rule 8)"));
                return BarResult::Nothing;
            }
            crate::rope::Status::Bottom => extra = "; at the foot of a rope (ropeandhook.md rule 8)".into(),
            crate::rope::Status::Top => {
                // Rule 9: third person forced, the first-person key locked,
                // the camera to the setRopeCam2 spot looking 1,000 above him.
                let lock = rope_lock(w, true, "the rope taken at the top");
                let camera = match (w.rope.top_cam, w.connor_at) {
                    (Some(spot), Some(c)) => {
                        let aim = [c[0], c[1], c[2] + ROPE_TOP_AIM];
                        w.requests.push(crate::active::Request::RopeCamera { spot, aim, at: "rope button".into() });
                        format!("the camera to the setRopeCam2 spot ({:.0}, {:.0}, {:.0}), looking at ({:.0}, {:.0}, {:.0}) (1,000 above Connor)", spot[0], spot[1], spot[2], aim[0], aim[1], aim[2])
                    }
                    (None, _) => "no setRopeCam2 spot was ever set: the camera stays".into(),
                    (_, None) => "Connor's place unknown: the camera stays".into(),
                };
                w.rope_note(format!("rope button at status top: third person forced{lock}; {camera} (rule 9)"));
                extra = format!("; at the top of a rope: {camera} (ropeandhook.md rule 9)");
            }
        }
    }
    let before = w.hand.replace(name.to_string());
    let back = before.map_or(String::new(), |b| format!("; {b} goes back to its slot"));
    w.item_note(format!("{head}: now the item in hand, the pointer {cursor}{back}{extra} (inventory.md rule 9)"));
    if t.category == crate::rope::ROPE_CATEGORY {
        w.rope_note(format!("rope button at status {}: the rope is in hand, the pointer {cursor}{back} (rule 8)", w.rope.status.name()));
    }
    BarResult::Hold
}

/// Rule 12: after a world click made while holding `held`: `accepted`
/// (the object's script got `ConnorAction <item> <type>`) empties the
/// hand, except coins while `goldCoinPersists 1`; otherwise it stays.
pub fn after_world_click(w: &mut WorldState, held: &str, accepted: bool) {
    let kind = w.items.get(held).map(|o| o.kind.clone()).unwrap_or_default();
    let money = w.items.kind(&kind).is_some_and(|t| t.category == "Money");
    let line = if !accepted {
        "not accepted or out of reach: it stays in hand".to_string()
    } else if money && w.gold_persists {
        "accepted; coins while goldCoinPersists 1: they stay in hand".to_string()
    } else {
        w.hand = None;
        let gone = if w.items.get(held).is_some_and(|o| o.carried_by(crate::talk::CONNOR)) { "still in the inventory" } else { "taken by the script" };
        format!("accepted: the hand is emptied, the play pointer back; the item is {gone}")
    };
    w.item_note(format!("world click with {held} ({kind}) in hand: {line} (inventory.md rule 12)"));
}

/// The item in hand, if it still exists and is carried (a script may
/// delete or take it; then the hand is emptied).
pub fn hand(w: &mut WorldState) -> Option<(String, String)> {
    let name = w.hand.clone()?;
    match w.items.get(&name) {
        Some(o) if o.carried_by(crate::talk::CONNOR) => Some((o.name.clone(), o.kind.clone())),
        _ => {
            w.hand = None;
            w.item_note(format!("the item in hand {name} is no longer carried: the hand is emptied"));
            None
        }
    }
}

/// `interface.md` rule 40: the carried item of slot `k` (0–7).
pub fn slot_item(w: &WorldState, k: usize) -> Option<String> {
    let kind = SLOT_TYPES.get(k)?;
    w.items.objects.iter().find(|o| o.carried_by("Connor") && o.kind.eq_ignore_ascii_case(kind) && o.quantity > 0).map(|o| o.name.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::active::Active;
    use crate::item::Items;
    use crate::stats::ConnorStats;
    use kq8_data::items::ItemTypes;

    /// Types in `Invinfo.itl`'s layout: name, Hlth SDam LDam Prot Exp,
    /// Comb, category, Dur, use sound.
    /// name, Hlth SDam LDam Prot Exp, Comb, category, Dur, use sound.
    type Row = (&'static str, [i64; 5], u8, &'static str, i32, &'static str);

    fn types() -> ItemTypes {
        let rows: [Row; 12] = [
            ("INVITEM_Clarity", [0; 5], 1, "HealthItem", 60, "clarity.aud"),
            ("INVITEM_Invisible", [0; 5], 1, "HealthItem", 15, "invisibl.aud"),
            ("INVITEM_Invulnerable", [0; 5], 1, "HealthItem", 20, "invuln.aud"),
            ("INVITEM_Mushroom", [5, 0, 0, 0, 0], 1, "HealthItem", -1, "healing.aud"),
            ("INVITEM_Strength", [0; 5], 1, "HealthItem", 20, "none"),
            ("INVITEM_Fists", [0; 5], 0, "ShortWeapon", 0, "none"),
            ("INVITEM_Dagger", [0, 5, 0, 0, 0], 0, "ShortWeapon", 0, "unsheath.aud"),
            ("INVITEM_LeatherGloves", [0, 0, 0, 1, 0], 0, "Gloves", 0, "none"),
            ("INVITEM_FullArmor", [0, 0, 0, 20, 0], 0, "FullArmor", 0, "none"),
            ("INVITEM_SilverCoins", [0; 5], 1, "Money", 0, "none"),
            ("INVITEM_Candle", [0, 0, 0, 0, 25], 0, "QuestItem", 60, "none"),
            ("INVITEM_RopeAndHook", [0; 5], 0, "Rope", 0, "hook.aud"),
        ];
        let mut text = format!("[InventoryItemTypeList]\nnumItems={}\n", rows.len());
        for (n, (name, s, comb, cat, dur, snd)) in rows.iter().enumerate() {
            let cursor = if matches!(*cat, "QuestItem" | "Money" | "Rope") { "IDBMA_CURS" } else { "none" };
            text.push_str(&format!(
                "item{n}=\tKQInventoryItemType\t{name}\t1\t{}\t{}\t{}\t{}\t{}\t{comb}\t{cat}\tIDBMP_X\t1 0 0 1\tNoModeChange\t{dur}\t1\tnone\tnone\t{snd}\tx.kqs\t{cursor}\t1\t256\n",
                s[0], s[1], s[2], s[3], s[4]
            ));
        }
        ItemTypes::parse(text.as_bytes()).0
    }

    fn active() -> Active {
        let mut a = Active::new(Default::default(), vec![], vec![], ["Connor".to_string()], Vec::<(String, Vec<u8>)>::new());
        a.world.items = Items::new(types(), []);
        a.set_connor_stats(ConnorStats { stats: Stats([10, 10, 0, 0, 0]), level: 1 });
        a.world.connor_at = Some([1.0, 2.0, 3.0]);
        a
    }

    fn carry(a: &mut Active, kind: &str, n: i64) -> String {
        let name = a.world.items.make(kind, n, Place::World { position: [100.0, 200.0, 0.0], angles: [0.0; 3] }).unwrap().name.clone();
        receive(&mut a.world, &name, Some([100.0, 200.0, 0.0]), "test");
        name
    }

    fn stats(a: &Active) -> [i64; 5] {
        a.world.connor_stats.unwrap().stats.0
    }

    #[test]
    fn stacks_join_and_health_items_heal_for_good() {
        let mut a = active();
        let first = carry(&mut a, "Mushroom", 2);
        let second = carry(&mut a, "Mushroom", 3);
        assert_eq!(a.world.items.get(&first).map(|o| o.quantity), Some(5));
        assert!(a.world.items.get(&second).unwrap().pending_delete, "receive requests deletion; the outer event completes it");
        assert!(!a.world.items.get(&second).unwrap().carried_by("Connor"));
        a.world.connor_stats.as_mut().unwrap().stats.0[0] = 3;
        assert_eq!(slot_item(&a.world, 0).as_deref(), Some(first.as_str()));
        assert_eq!(bar_click(&mut a.world, &first, true, false, "key 1"), BarResult::Applied);
        assert_eq!((stats(&a)[0], a.world.items.get(&first).map(|o| (o.quantity, o.third))), (8, Some((4, -1))), "+5, Dur -1: not applied any more, the heal kept");
        assert_eq!(a.world.sounds.len(), 1, "healing.aud");
        // Capped at 10 x level.
        bar_click(&mut a.world, &first, true, false, "key 1");
        assert_eq!(stats(&a)[0], 10);
        // Ignored during handsOff.
        assert_eq!(bar_click(&mut a.world, &first, true, true, "key 1"), BarResult::Ignored);
        for _ in 0..3 {
            bar_click(&mut a.world, &first, true, false, "key 1");
        }
        assert!(a.world.items.get(&first).unwrap().pending_delete, "count 0: leaves the inventory before event-end destruction");
        assert!(!a.world.items.get(&first).unwrap().carried_by("Connor"));
        assert_eq!(slot_item(&a.world, 0), None);
    }

    #[test]
    fn a_potion_runs_out_and_its_numbers_go_back() {
        let mut a = active();
        let p = carry(&mut a, "Strength", 2);
        bar_click(&mut a.world, &p, true, false, "key 6");
        assert_eq!(stats(&a)[1], 15);
        bar_click(&mut a.world, &p, true, false, "key 6");
        assert_eq!((stats(&a)[1], a.world.items.get(&p).map(|o| o.quantity)), (15, Some(1)), "already applied: nothing");
        a.world.updates = 20 * 15 - 1;
        timers(&mut a.world);
        assert_eq!((stats(&a)[1], a.world.potions[0].left), (15, 1), "1 s left at 19 s");
        a.world.updates = 20 * 15;
        timers(&mut a.world);
        assert_eq!((stats(&a)[1], a.world.items.get(&p).map(|o| o.third)), (10, Some(-1)));
        assert!(a.world.potions.is_empty());
    }

    #[test]
    fn the_last_dose_runs_at_count_0_and_dying_refuses() {
        let mut a = active();
        let p = carry(&mut a, "Strength", 1);
        assert_eq!(bar_click(&mut a.world, &p, true, false, "key 6"), BarResult::Applied);
        assert_eq!(a.world.sounds.len(), 0, "the test type's use sound is none");
        // potions.md rule 8: kept at count 0 while it runs, the slot empty.
        let o = a.world.items.get(&p).unwrap();
        assert_eq!((o.quantity, o.carried_by("Connor")), (0, true));
        assert_eq!(slot_item(&a.world, 5), None);
        // A new one picked up meanwhile joins it; clicking it is the
        // running potion again: nothing (rule 7).
        carry(&mut a, "Strength", 1);
        assert_eq!(a.world.items.get(&p).map(|o| o.quantity), Some(1));
        assert_eq!(bar_click(&mut a.world, &p, true, false, "key 6"), BarResult::Nothing);
        assert_eq!(a.world.items.get(&p).map(|o| o.quantity), Some(1));
        // Rule 2: refused while dying.
        let m = carry(&mut a, "Mushroom", 1);
        a.world.connor_dying = true;
        assert_eq!(bar_click(&mut a.world, &m, true, false, "key 1"), BarResult::Ignored);
        assert_eq!(a.world.items.get(&m).map(|o| o.quantity), Some(1));
        assert!(a.world.sounds.is_empty());
    }

    #[test]
    fn the_shield_raises_protection_and_takes_off_only_what_it_added() {
        let mut a = active();
        a.world.connor_stats.as_mut().unwrap().stats.0[3] = 40;
        let v = carry(&mut a, "Invulnerable", 1);
        bar_click(&mut a.world, &v, true, false, "key 5");
        assert_eq!(stats(&a)[3], 60);
        // The last temple: a hit uses 40 + 20 / 2 = 50 (potions.md rule 19).
        a.world.connor_in_temple4 = true;
        a.world.hit_connor("goblin", 5, crate::fight::HitKind::Normal);
        assert!(a.world.fight_log.iter().any(|l| l.contains("protection 60 used as 50")), "{:?}", a.world.fight_log);
        // Armour put on meanwhile (+1) is kept at the end: 61 - 20 = 41.
        a.world.connor_stats.as_mut().unwrap().stats.0[3] += 1;
        a.world.updates = 20 * 15;
        timers(&mut a.world);
        assert_eq!(stats(&a)[3], 41);
        assert!(a.world.status_icons.iter().all(Option::is_none));
    }

    #[test]
    fn invisible_reports_him_100000_below() {
        let mut a = active();
        let p = carry(&mut a, "Invisible", 1);
        assert_eq!(a.world.connor_reported(), Some([1.0, 2.0, 3.0]));
        bar_click(&mut a.world, &p, true, false, "key 8");
        assert!(a.world.connor_invisible());
        assert_eq!(a.world.connor_reported(), Some([1.0, 2.0, 3.0 - 100_000.0]));
        assert_eq!(a.world.connor_at, Some([1.0, 2.0, 3.0]), "his true place is kept");
        a.world.updates = 15 * 15;
        timers(&mut a.world);
        assert_eq!(a.world.connor_reported(), Some([1.0, 2.0, 3.0]));
        // M5 5g-9: a cutscene does the same, but checkInvisible stays 0.
        a.world.connor_hands_off = true;
        assert_eq!(a.world.connor_reported(), Some([1.0, 2.0, 3.0 - 100_000.0]));
        assert!(!a.world.connor_invisible());
        // potions.md rule 26 (Q146): his talk state too, not standing.
        a.world.connor_hands_off = false;
        a.world.connor_state = Some(crate::character::state_number::TALK);
        assert_eq!(a.world.connor_hidden_why(), Some("his talk state (a conversation)"));
        a.world.connor_state = Some(crate::character::state_number::WAIT);
        assert_eq!(a.world.connor_reported(), Some([1.0, 2.0, 3.0]));
    }

    #[test]
    fn clarity_tells_the_reveal_objects_at_start_and_end() {
        let mut a = active();
        // D-6: `tileshow` is prepared for a loadKQ and not loaded yet: not
        // called (rules 31, 32).
        a.world.reveal_objects = vec!["keepwall".into(), "wizchest1".into(), "Tileshow".into()];
        a.world.known.extend(["keepwall".to_string(), "wizchest1".to_string()]);
        let p = carry(&mut a, "Clarity", 1);
        bar_click(&mut a.world, &p, true, false, "key 7");
        assert_eq!(std::mem::take(&mut a.world.reveal_calls), vec![("keepwall".to_string(), true), ("wizchest1".to_string(), true)]);
        a.world.updates = 60 * 15;
        timers(&mut a.world);
        assert_eq!(a.world.reveal_calls, vec![("keepwall".to_string(), false), ("wizchest1".to_string(), false)]);
    }

    #[test]
    fn native_item_receipts_consume_their_bits_before_delivery() {
        let mut a = active();
        a.world.updates = 300;
        a.world.advance_clock_for_test(20.);
        // Installation of an existing carried stack is not a fresh receipt.
        a.world
            .items
            .add_named("found", "Mushroom", 1, -1, Place::Carried("Connor".into()))
            .unwrap();
        assert_eq!(a.world.help_prompts.history(300).start_help, 0);
        carry(&mut a, "Mushroom", 2);
        assert_eq!(a.world.items.count("Connor", "INVITEM_Mushroom"), 3);
        carry(&mut a, "Clarity", 1);
        carry(&mut a, "Candle", 1);
        carry(&mut a, "Candle", 1);
        carry(&mut a, "RopeAndHook", 1);
        assert_eq!(a.world.help_prompts.history(300).start_help, 7);
        let q = &mut a.world.help_prompts;
        assert_eq!(q.due(344, true, false, false), None);
        assert_eq!(q.due(345, true, false, false), Some(131));
        assert_eq!(q.due(345, true, false, false), None);
        assert_eq!(q.due(368, true, false, false), None);
        assert_eq!(q.due(390, true, false, false), Some(132));
        assert_eq!(q.due(435, true, false, false), Some(128));
        a.world.updates = 500;
        carry(&mut a, "RopeAndHook", 1);
        assert_eq!(a.world.help_prompts.due(568, true, false, false), None);
    }

    #[test]
    fn quest_and_rope_have_separate_delays_and_ignore_the_current_preference_at_receipt() {
        let mut a = active();
        a.world.updates = 300;
        a.world.advance_clock_for_test(20.);
        carry(&mut a, "Candle", 1);
        assert_eq!(a.world.help_prompts.history(300).start_help, 2);
        assert_eq!(a.world.help_prompts.due(344, true, false, false), None);
        assert_eq!(a.world.help_prompts.due(345, false, false, false), None);
        a.world.updates = 400;
        carry(&mut a, "Candle", 1);
        assert_eq!(a.world.help_prompts.due(445, true, false, false), None);
        a.world.updates = 500;
        a.world.advance_clock_for_test(500f32 / 15.);
        carry(&mut a, "RopeAndHook", 1);
        assert_eq!(a.world.help_prompts.due(567, true, false, false), None);
        assert_eq!(a.world.help_prompts.due(568, true, true, false), None);
        assert_eq!(a.world.help_prompts.due(582, true, false, false), None);
        assert_eq!(a.world.help_prompts.due(583, true, false, false), Some(128));
    }

    #[test]
    fn stop_help_keeps_older_requests_and_blocks_future_receipts() {
        let mut a = active();
        carry(&mut a, "Candle", 1);
        assert_eq!(a.world.help_prompts.due(45, true, true, false), None);
        assert_eq!(a.test_command("stopHelp", "test"), Some("0".into()));
        assert_eq!(a.world.help_prompts.history(45).held_cases, [132]);
        assert_eq!(a.world.help_prompts.history(45).start_help, 127);
        carry(&mut a, "Clarity", 1);
        carry(&mut a, "RopeAndHook", 1);
        assert_eq!(a.world.help_prompts.due(60, true, false, false), Some(132));
        a.world.updates = 200;
        a.world.advance_clock_for_test(200f32 / 15.);
        assert_eq!(a.test_command("doPopup 127", "test"), Some("0".into()));
        assert_eq!(a.world.help_prompts.due(245, true, false, false), Some(127));
        assert_eq!(a.world.help_prompts.due(300, true, false, false), None);
    }

    #[test]
    fn the_candle_gives_25_experience_0_3_s_after_it_is_added() {
        let mut a = active();
        carry(&mut a, "Candle", 1);
        assert_eq!(stats(&a)[4], 0);
        a.world.updates = 4;
        a.world.advance_clock_for_test(4f32/15.);
        timers(&mut a.world);
        assert_eq!(stats(&a)[4], 0, "0.27 s: not yet");
        a.world.updates = 5;
        a.world.advance_clock_for_test(5f32/15.);
        timers(&mut a.world);
        // +25 at level 1 (threshold 10): level 2, 15 left.
        let level = |a: &Active| a.world.connor_stats.unwrap().level;
        assert_eq!((level(&a), stats(&a)[4]), (2, 15));
        // Never taken back (rule 40): its Dur is not used.
        a.world.updates = 2000;
        a.world.advance_clock_for_test(2000f32/15.);
        timers(&mut a.world);
        assert_eq!((level(&a), stats(&a)[4]), (2, 15));
    }

    #[test]
    fn equipment_pushes_out_what_shares_its_place() {
        let mut a = active();
        let fists = a.world.items.add_named("Fist1", "Fists", 1, -1, Place::Carried("Connor".into())).unwrap().name.clone();
        let dagger = carry(&mut a, "Dagger", 1);
        assert!(a.world.items.get(&fists).unwrap().pending_delete, "a replaced placeholder is requested for deletion");
        assert!(!a.world.items.get(&fists).unwrap().carried_by("Connor"));
        assert_eq!(stats(&a)[1], 15, "the dagger is put on: +5");
        let gloves = carry(&mut a, "LeatherGloves", 1);
        assert_eq!(stats(&a)[3], 1);
        // The full suit covers the gloves: they come off and lie where the
        // suit lay; their number is taken back.
        let suit = a.world.items.make("FullArmor", 1, Place::World { position: [7.0, 8.0, 9.0], angles: [0.0; 3] }).unwrap().name.clone();
        receive(&mut a.world, &suit, Some([7.0, 8.0, 9.0]), "picked up");
        assert_eq!(a.world.items.get(&gloves).map(|o| o.place.clone()), Some(Place::World { position: [7.0, 8.0, 9.0], angles: [0.0; 3] }));
        assert_eq!(stats(&a)[3], 20);
        // Gloves again (a give: omitted place): the suit drops at the origin.
        let g2 = a.world.items.make("LeatherGloves", 1, Place::Carried("Connor".into())).unwrap().name.clone();
        receive(&mut a.world, &g2, None, "give");
        assert_eq!(a.world.items.get(&suit).map(|o| o.place.clone()), Some(Place::World { position: [0.0; 3], angles: [0.0; 3] }));
        assert_eq!(stats(&a)[3], 1);
        assert!(a.world.items.get(&dagger).unwrap().carried_by("Connor"));
    }

    #[test]
    fn quest_application_replacement_lifecycle_matrix() {
        use crate::journey::Journey;
        use crate::potions::StatusIcon;

        // Synthetic quest types make both permanent and timed statistics
        // observable without the Candle's separate pickup experience rule.
        for old_duration in [0, 4] {
            for new_duration in [0, 6] {
                let fixture = || {
                    let mut a = active();
                    let template = a.world.items.kind("Candle").unwrap().clone();
                    for (kind, duration, stats) in [
                        ("INVITEM_QuestOld", old_duration, [0, 2, 3, 4, 0]),
                        ("INVITEM_QuestNew", new_duration, [0, 7, 1, 2, 0]),
                    ] {
                        let mut t = template.clone();
                        t.name = kind.into();
                        t.duration = duration as f32;
                        t.stats = stats;
                        a.world.items.types.types.push(t);
                    }
                    a
                };
                let mut a = fixture();
                let dagger = carry(&mut a, "Dagger", 1);
                let clarity = carry(&mut a, "Clarity", 2);
                apply(&mut a.world, &clarity, "unrelated potion");
                let baseline = stats(&a);
                let old = carry(&mut a, "QuestOld", 1);
                let new = carry(&mut a, "QuestNew", 1);
                let old_unapplied = a.world.items.get(&old).unwrap().clone();
                let mut new_applied = a.world.items.get(&new).unwrap().clone();
                new_applied.third = CONNOR_USER_ID;
                assert_eq!(bar_click(&mut a.world, &old, true, false, "hold"), BarResult::Hold);
                apply(&mut a.world, &old, "first quest item");
                assert_eq!(stats(&a), [10, 17, 3, 4, 0]);
                a.world.updates = 17;
                timers(&mut a.world);
                let clarity_running = a.world.potions.iter().find(|r| r.item == clarity).unwrap().clone();
                let before_events = a.world.connor_stats_events;

                apply(&mut a.world, &new, "replace quest item");
                assert_eq!(stats(&a), [10, 22, 1, 2, 0], "old Dur {old_duration}, new Dur {new_duration}");
                assert_eq!(a.world.connor_stats_events, before_events + 2, "one removal, then one application");
                assert_eq!(a.world.items.get(&old), Some(&old_unapplied), "old ownership, count and strip order are kept");
                assert_eq!(a.world.items.get(&new), Some(&new_applied));
                assert_eq!(a.world.hand.as_deref(), Some(old.as_str()), "applying does not change the held item");
                assert_eq!(a.world.items.get(&dagger).unwrap().third, CONNOR_USER_ID);
                assert_eq!(a.world.potions.iter().find(|r| r.item == clarity), Some(&clarity_running));
                assert!(!a.world.potions.iter().any(|r| r.item == old));
                assert_eq!(a.world.potions.iter().filter(|r| r.item == new).count(), usize::from(new_duration > 0));
                assert_eq!(a.world.status_icons[0], Some(StatusIcon::Potion(clarity.clone())));
                assert_eq!(a.world.status_icons[1], (new_duration > 0).then(|| StatusIcon::Potion(new.clone())));
                assert!(a.world.status_icons[2..].iter().all(Option::is_none));

                // A repeated application changes neither statistics nor the
                // remaining timer, icon, inventory or pointer state.
                a.world.updates = 21;
                timers(&mut a.world);
                let before = Journey::take(&a.world, &a.scripts.vars, "Daventry");
                let icons = a.world.status_icons.clone();
                let before_events = a.world.connor_stats_events;
                assert!(apply(&mut a.world, &new, "repeat").contains("already applied"));
                assert_eq!(Journey::take(&a.world, &a.scripts.vars, "Daventry"), before);
                assert_eq!(a.world.status_icons, icons);
                assert_eq!(a.world.connor_stats_events, before_events);

                // Round-trip the actual carried-state record onto a different
                // update clock, then remove through the ordinary drop command.
                let bytes = serde_json::to_vec(&before).unwrap();
                let saved: Journey = serde_json::from_slice(&bytes).unwrap();
                let mut loaded = fixture();
                loaded.world.updates = 500;
                saved.give(&mut loaded.world, &mut loaded.scripts.vars);
                let restored = Journey::take(&loaded.world, &loaded.scripts.vars, "Daventry");
                assert_eq!((restored.stats, &restored.items, &restored.hand, &restored.potions),
                    (before.stats, &before.items, &before.hand, &before.potions));
                assert_eq!(loaded.world.status_icons, icons);
                assert_eq!(loaded.console_command("KQMonster::dropItem Connor QuestNew 7 8 9 0 0 0", "remove applied quest item"), Some("0".into()));
                assert_eq!(stats(&loaded), baseline);
                let dropped = loaded.world.items.get(&new).unwrap();
                assert_eq!(dropped.third, -1);
                assert_eq!(dropped.place, Place::World { position: [7.0, 8.0, 9.0], angles: [0.0; 3] });
                assert_eq!(loaded.world.hand.as_deref(), Some(old.as_str()));
                assert_eq!(loaded.world.potions.len(), 1);
                assert_eq!(loaded.world.potions[0].item, clarity);
                assert!(loaded.world.status_icons[1..].iter().all(Option::is_none));
                let after_drop = loaded.world.connor_stats_events;
                loaded.world.updates += 90;
                timers(&mut loaded.world);
                assert_eq!(stats(&loaded), baseline, "the removed timer must not subtract statistics later");
                assert_eq!(loaded.world.connor_stats_events, after_drop);

                // Without removal, a timed replacement expires exactly once;
                // a permanent replacement remains on after both old deadlines.
                a.world.updates = 107;
                timers(&mut a.world);
                assert_eq!(stats(&a), if new_duration > 0 { baseline } else { [10, 22, 1, 2, 0] });
                assert_eq!(a.world.items.get(&new).unwrap().third, if new_duration > 0 { -1 } else { CONNOR_USER_ID });
                assert_eq!(a.world.connor_stats_events, before_events + u64::from(new_duration > 0));
                assert_eq!(a.world.potions.len(), 1);
                assert_eq!(a.world.potions[0].item, clarity);
                assert!(a.world.status_icons[1..].iter().all(Option::is_none));
                a.world.updates += 1;
                timers(&mut a.world);
                assert_eq!(a.world.connor_stats_events, before_events + u64::from(new_duration > 0));
            }
        }
    }

    #[test]
    fn quest_items_money_are_held_the_order_kept() {
        let mut a = active();
        let c = carry(&mut a, "Candle", 1);
        let coins = carry(&mut a, "SilverCoins", 10);
        carry(&mut a, "SilverCoins", 5);
        assert_eq!(a.world.items.get(&coins).map(|o| o.quantity), Some(15));
        assert!(a.world.items.get(&c).unwrap().added < a.world.items.get(&coins).unwrap().added);
        assert_eq!(bar_click(&mut a.world, &c, true, false, "click"), BarResult::Hold);
        assert_eq!(bar_click(&mut a.world, &coins, false, false, "right click"), BarResult::Described);
        assert_eq!(a.world.item_voices.len(), 1);
        assert_eq!(stats(&a)[4], 0, "a bar click never applies a quest item");
        assert_eq!(a.world.hand.as_deref(), Some(c.as_str()));
        // The coins: they become the hand, the candle goes back.
        assert_eq!(bar_click(&mut a.world, &coins, true, false, "click"), BarResult::Hold);
        assert_eq!(a.world.hand.as_deref(), Some(coins.as_str()));
        // The same again: put back.
        assert_eq!(bar_click(&mut a.world, &coins, true, false, "click"), BarResult::PutBack);
        assert_eq!(a.world.hand, None);
    }

    #[test]
    fn ordinary_bar_selection_held_category_and_empty_matrix() {
        let mut a = active();
        let candle = carry(&mut a, "Candle", 1);
        let money = carry(&mut a, "SilverCoins", 50);
        let mushroom = carry(&mut a, "Mushroom", 2);
        let strength = carry(&mut a, "Strength", 2);
        let weapon = carry(&mut a, "Dagger", 1);
        let rope = carry(&mut a, "RopeAndHook", 1);
        let mut cursorless = a.world.items.types.find("Candle").unwrap().clone();
        cursorless.name = "INVITEM_AntiPoisonFlower".into();
        cursorless.cursor = None;
        a.world.items.types.types.push(cursorless);
        let flower = carry(&mut a, "AntiPoisonFlower", 1);

        assert_eq!(bar_click(&mut a.world, &candle, true, false, "quest"), BarResult::Hold);
        assert_eq!(bar_click(&mut a.world, &money, true, false, "coins"), BarResult::Hold);
        assert_eq!(a.world.hand.as_deref(), Some(money.as_str()));
        assert!(a.world.items.get(&candle).unwrap().carried_by("Connor"));
        for item in [&flower, &rope] {
            assert_eq!(bar_click(&mut a.world, item, true, false, "category refused"), BarResult::Nothing);
            assert_eq!(a.world.hand.as_deref(), Some(money.as_str()), "failed category path retains selection");
        }
        assert_eq!(bar_click(&mut a.world, &candle, false, false, "describe"), BarResult::Described);
        assert_eq!(a.world.hand.as_deref(), Some(money.as_str()));
        a.world.connor_stats.as_mut().unwrap().stats.0[0] = 3;
        assert_eq!(bar_click(&mut a.world, &mushroom, true, false, "heal"), BarResult::Applied);
        assert_eq!(a.world.hand, None);
        assert_eq!(stats(&a)[0], 8);
        assert_eq!(a.world.items.get(&mushroom).unwrap().quantity, 1);
        assert_eq!(a.world.items.get(&money).unwrap().quantity, 50);

        bar_click(&mut a.world, &money, true, false, "hold money");
        assert_eq!(bar_click(&mut a.world, &weapon, true, false, "weapon"), BarResult::Applied);
        assert!(a.world.hand.is_none());
        assert!(a.world.items.get(&money).unwrap().carried_by("Connor"));
        assert_eq!(bar_click(&mut a.world, &strength, true, false, "potion"), BarResult::Applied);
        let running = (a.world.potions.clone(), a.world.items.get(&strength).unwrap().quantity);
        bar_click(&mut a.world, &candle, true, false, "hold quest");
        assert_eq!(bar_click(&mut a.world, &strength, true, false, "already running"), BarResult::Nothing);
        assert!(a.world.hand.is_none());
        assert_eq!((a.world.potions.clone(), a.world.items.get(&strength).unwrap().quantity), running);

        for held in [&candle, &money] {
            bar_click(&mut a.world, held, true, false, "hold");
            assert_eq!(empty_bar_click(&mut a.world, true, "handsOff"), BarResult::Ignored);
            assert_eq!(a.world.hand.as_deref(), Some(held.as_str()));
            a.world.connor_dying = true;
            assert_eq!(empty_bar_click(&mut a.world, false, "dead"), BarResult::Ignored);
            assert_eq!(a.world.hand.as_deref(), Some(held.as_str()));
            a.world.connor_dying = false;
            assert_eq!(empty_bar_click(&mut a.world, false, "empty cell"), BarResult::PutBack);
            assert!(a.world.hand.is_none());
            assert!(a.world.items.get(held).unwrap().carried_by("Connor"));
            assert_eq!(empty_bar_click(&mut a.world, false, "empty cell again"), BarResult::Nothing);
        }
        assert_eq!(a.world.items.get(&money).unwrap().quantity, 50);
    }

    #[test]
    fn inventory_selection_scroll_rock_owner_and_rope_lock_matrix() {
        fn extras(a: &mut Active) {
            for (name, category) in [("INVITEM_Scroll", "QuestItem"), ("INVITEM_Rock", "Rock")] {
                let mut kind = a.world.items.types.find("Candle").unwrap().clone();
                kind.name = name.into();
                kind.category = category.into();
                a.world.items.types.types.push(kind);
            }
        }
        fn queued(w: &WorldState) -> serde_json::Value {
            serde_json::to_value(w.talk.saved(w.now())).unwrap()["queues"][0]["lines"][0]["tuple"].clone()
        }
        for owner in [BarOwner::Ordinary, BarOwner::Tile, BarOwner::Tile2, BarOwner::Tile3, BarOwner::Scales] {
            let restricted = matches!(owner, BarOwner::Tile | BarOwner::Tile3 | BarOwner::Scales);
            for previous in [None, Some("Candle"), Some("Rock")] {
                let mut a = active();
                extras(&mut a);
                let rock = carry(&mut a, "Rock", 2);
                let candle = carry(&mut a, "Candle", 1);
                a.world.hand = previous.map(|kind| if kind == "Rock" { rock.clone() } else { candle.clone() });
                let before = a.world.hand.clone();
                let result = bar_click_with_owner(&mut a.world, &rock, true, false, owner, "owner matrix");
                if restricted {
                    assert_eq!(result, BarResult::Nothing, "{owner:?}");
                    assert_eq!(a.world.hand, before, "{owner:?}: even an already-held rock is retained");
                    assert_eq!(queued(&a.world), serde_json::json!({"file":0,"noun":0,"verb":0,"case":0,"seq":3}));
                } else if previous == Some("Rock") {
                    assert_eq!(result, BarResult::PutBack);
                    assert!(a.world.hand.is_none());
                } else {
                    assert_eq!(result, BarResult::Hold);
                    assert_eq!(a.world.hand.as_deref(), Some(rock.as_str()));
                }
                assert_eq!(a.world.items.get(&rock).unwrap().quantity, 2);
                assert!(a.world.items.get(&candle).unwrap().carried_by("Connor"));
            }
        }
        for (elapsed, recent, busy, remark) in [(0.0_f32, None, false, true),
            (f32::from_bits(3.0_f32.to_bits() - 1), Some(0.0), false, false),
            (3.0, Some(0.0), false, true), (4.0, None, true, false)] {
            for held in ["Candle", "RopeAndHook"] {
                let mut a = active();
                extras(&mut a);
                let scroll = carry(&mut a, "Scroll", 1);
                let prior = carry(&mut a, held, 1);
                a.world.rope.status = crate::rope::Status::Top;
                a.world.first_person = true;
                assert_eq!(bar_click(&mut a.world, &prior, true, false, "prior selection"), BarResult::Hold);
                // Remarks use the simulation clock, not the diagnostic
                // update counter. Cover the adjacent f32 value below 3 s.
                a.world.advance_clock_for_test(elapsed);
                a.world.scroll_remark_at = recent;
                if busy { a.world.connor_remark(kq8_data::msg::Tuple::new(0, 1, 1, 1, 1), "already talking"); }
                let talk_before = serde_json::to_value(a.world.talk.saved(a.world.now())).unwrap();
                assert_eq!(bar_click(&mut a.world, &scroll, true, false, "scroll gate"), BarResult::Nothing);
                assert!(a.world.hand.is_none(), "selection clears before every speech gate");
                assert!(a.world.items.get(&prior).unwrap().carried_by("Connor"));
                assert_eq!(a.world.rope_first_person_lock, held == "RopeAndHook");
                if remark {
                    assert_eq!(queued(&a.world), serde_json::json!({"file":0,"noun":92,"verb":14,"case":0,"seq":1}),
                        "held={held}, time={elapsed}, prior={recent:?}, busy={busy}; {:?}", a.world.item_log);
                    assert_eq!(a.world.scroll_remark_at, Some(a.world.now()));
                } else {
                    assert_eq!(serde_json::to_value(a.world.talk.saved(a.world.now())).unwrap(), talk_before);
                    assert_eq!(a.world.scroll_remark_at, recent);
                }
            }
        }
        for target in [None, Some("Mushroom"), Some("Scroll"), Some("Candle"), Some("SilverCoins"), Some("Rock"), Some("RopeAndHook")] {
            let mut a = active();
            extras(&mut a);
            let rope = carry(&mut a, "RopeAndHook", 1);
            let item = target.map(|kind| if kind == "RopeAndHook" { rope.clone() } else { carry(&mut a, kind, 2) });
            a.world.rope.status = crate::rope::Status::Top;
            a.world.first_person = true;
            bar_click(&mut a.world, &rope, true, false, "hold rope at Top");
            match item {
                Some(item) => { bar_click(&mut a.world, &item, true, false, "replace rope"); }
                None => { empty_bar_click(&mut a.world, false, "empty cell"); }
            }
            let put_back_rope = target == Some("RopeAndHook");
            assert_eq!(a.world.rope_first_person_lock, !put_back_rope, "{target:?}");
            assert_eq!(a.world.first_person, put_back_rope, "{target:?}");
            assert!(a.world.items.get(&rope).unwrap().carried_by("Connor"));
        }
        // Existing outer guards still win over Scroll clearing and Rock remarks.
        for kind in ["Scroll", "Rock"] {
            for dying in [false, true] {
                let mut a = active();
                extras(&mut a);
                let item = carry(&mut a, kind, 1);
                let candle = carry(&mut a, "Candle", 1);
                a.world.hand = Some(candle.clone());
                a.world.connor_dying = dying;
                assert_eq!(bar_click_with_owner(&mut a.world, &item, true, !dying, BarOwner::Tile, "outer guard"), BarResult::Ignored);
                assert_eq!(a.world.hand.as_deref(), Some(candle.as_str()));
                assert!(!a.world.talk.is_talking("Connor"));
            }
        }
    }

    #[test]
    fn inventory_scripted_equipment_replacement_placement_matrix() {
        for (command, position) in [
            ("give Dagger", [0.0; 3]),
            ("KQMonster::inventory Connor Dagger 1", [0.0; 3]),
            ("KQMonster::inventory Connor Dagger 1 700 -800 -1 0.1 0.2 0.3", [700.0, -800.0, -1.0]),
        ] {
            let mut a = active();
            a.world.connor_at = Some([9000.0, 8000.0, 7000.0]);
            let old = carry(&mut a, "Dagger", 1);
            assert_eq!(a.test_command(command, "scripted replacement").as_deref(), Some("0"));
            assert_eq!(a.world.items.get(&old).unwrap().place, Place::World { position, angles: [0.0; 3] });
            assert_eq!(a.world.items.get(&old).unwrap().third, -1);
            let new = a.world.items.find("Connor", "Dagger").unwrap();
            assert_ne!(new.name, old);
            assert_eq!(new.stored_pose(), crate::item::StoredPose::default(),
                "placement precedes exchange; the established carry-add rule then resets it");
            assert_eq!(new.third, CONNOR_USER_ID);
        }
        // Ordinary world pickup still exchanges at the pickup, not either origin or Connor.
        let mut a = active();
        let old = carry(&mut a, "Dagger", 1);
        let position = [123.0, 456.0, 789.0];
        let new = a.world.items.make("Dagger", 1, Place::World { position, angles: [0.4, 0.5, 0.6] }).unwrap().name.clone();
        receive(&mut a.world, &new, Some(position), "world pickup");
        assert_eq!(a.world.items.get(&old).unwrap().place, Place::World { position, angles: [0.0; 3] });
    }

    #[test]
    fn the_hand_empties_when_accepted_except_coins_that_persist() {
        let mut a = active();
        let c = carry(&mut a, "Candle", 1);
        let coins = carry(&mut a, "SilverCoins", 10);
        bar_click(&mut a.world, &c, true, false, "click");
        after_world_click(&mut a.world, &c, false);
        assert_eq!(a.world.hand.as_deref(), Some(c.as_str()), "refused: it stays");
        after_world_click(&mut a.world, &c, true);
        assert_eq!(a.world.hand, None, "accepted: emptied");
        assert!(a.world.items.get(&c).unwrap().carried_by("Connor"), "the item itself is not removed by this");
        bar_click(&mut a.world, &coins, true, false, "click");
        a.world.gold_persists = true;
        after_world_click(&mut a.world, &coins, true);
        assert_eq!(a.world.hand.as_deref(), Some(coins.as_str()), "goldCoinPersists 1: the coins stay");
        // A script deleted the held item: the hand is emptied.
        a.world.items.objects.iter_mut().find(|o| o.name == coins).unwrap().place = Place::Gone;
        assert_eq!(hand(&mut a.world), None);
        assert_eq!(a.world.hand, None);
    }

    #[test]
    fn on_the_rope_hits_weapons_are_ignored_and_characters_lose_him() {
        // ropeandhook.md rules 27-30 (M6 6a-6).
        let mut a = active();
        a.world.connor_roping = true;
        let health = stats(&a)[0];
        a.world.hit_connor("goblin", 5, crate::fight::HitKind::Normal);
        assert_eq!(stats(&a)[0], health, "no damage");
        assert!(a.world.requests.is_empty(), "no hurt or dying request");
        a.world.ask_weapon(crate::weapons::Ask::Draw, "Delete");
        let r = a.world.weapon_requests.pop().unwrap();
        assert_eq!(a.world.take_weapon_request(&r, Ok(()), false), None, "the rope state ignores it");
        assert_eq!(a.world.connor_reported(), Some([1.0, 2.0, 3.0 - crate::potions::HIDDEN_DROP]));
        assert_eq!(a.world.connor_hidden_why(), Some("the rope state"));
        a.world.connor_roping = false;
        a.world.hit_connor("goblin", 5, crate::fight::HitKind::Normal);
        assert!(stats(&a)[0] < health, "off the rope he is hurt again");
    }

    #[test]
    fn saved_rope_lock_returns_to_the_view_used_before_the_rope() {
        for first in [true, false] {
            let mut original = active();
            original.world.first_person = first;
            rope_lock(&mut original.world, true, "rope held at top");
            rope_lock(&mut original.world, true, "already locked");
            assert!(!original.world.first_person);
            let bytes = serde_json::to_vec(&original.world.view_flags()).unwrap();
            let mut loaded = active();
            loaded.world.restore_view_flags(serde_json::from_slice(&bytes).unwrap());
            assert!(!loaded.world.first_person_key(false));
            rope_lock(&mut loaded.world, false, "rope put back");
            assert_eq!(loaded.world.first_person, first);
            assert!(loaded.world.first_person_key(false));
        }
    }

    #[test]
    fn the_rope_button_by_status() {
        let words = |l: &str| l.split_whitespace().map(str::to_string).collect::<Vec<_>>();
        let mut a = active();
        let rope = carry(&mut a, "RopeAndHook", 1);
        // ropeandhook.md rule 8: away from a rope place, only the remark.
        assert_eq!(bar_click(&mut a.world, &rope, true, false, "click"), BarResult::Nothing);
        assert_eq!(a.world.hand, None);
        // At the foot: held.
        a.world.rope.command(&words("KQConner::setRope Bottom 1 2 3 0 0 0"), true, 0.0);
        assert_eq!(bar_click(&mut a.world, &rope, true, false, "click"), BarResult::Hold);
        assert_eq!(a.world.hand.as_deref(), Some(rope.as_str()));
        assert!(a.world.requests.is_empty(), "no camera move at the foot");
        // Rule 6: leaving the place puts it back.
        let r = a.test_command("KQConner::setRope None", "test");
        assert_eq!(r.as_deref(), Some("0"));
        assert_eq!(a.world.hand, None);
        // At the top: the camera to setRopeCam2 looking 1,000 above him, the lock.
        a.test_command("KQConner::setRopeCam2 10 20 30", "test");
        a.test_command("KQConner::setRope Top 1 2 3 0 0 0", "test");
        assert_eq!(bar_click(&mut a.world, &rope, true, false, "click"), BarResult::Hold);
        assert!(a.world.rope_first_person_lock);
        assert!(a.world.requests.iter().any(|r| matches!(r, crate::active::Request::RopeCamera { spot, aim, .. } if *spot == [10.0, 20.0, 30.0] && *aim == [1.0, 2.0, 1003.0])));
        // Again: put back, the lock released.
        assert_eq!(bar_click(&mut a.world, &rope, true, false, "click"), BarResult::PutBack);
        assert!(!a.world.rope_first_person_lock);
        // In the rope state: ignored.
        a.world.connor_roping = true;
        assert_eq!(bar_click(&mut a.world, &rope, true, false, "click"), BarResult::Ignored);
    }
}
