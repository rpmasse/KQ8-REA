//! Connor's inventory as the panels show it (M4 step 4c-2 on).
//!
//! Rules from `docs/behaviour/inventory.md` and `interface.md` (A,
//! 2026-10-18):
//!
//! - Everything he carries is one list of item objects; where an item
//!   shows follows from its type's category (`inventory.md` rule 2).
//! - His start (`Connor.kq` `[ConnerInventory]`): the first mask piece
//!   (a quest item), the three placeholders `INVITEM_Fists`,
//!   `INVITEM_NoRanged`, `INVITEM_NoArmor` (empty pictures, no stats) and
//!   the mask button's piece (`inventory.md` rule 4).
//! - The armour, hand-weapon and ranged slots show the picture of the
//!   item worn there (`interface.md` rule 34). Only body armour, under-
//!   armour, full suits and `GodArmor` change the armour picture; with a
//!   chain-mail suit and body armour both worn, the suit's picture unless
//!   the body armour is plate or bronze plate (rule 35).
//!
//! Q109/Q110 (2026-11-08): represented equipment follows assignment in
//! carried order, not a protection rank. Counts and the blink's observed
//! shared-clock phases are also independent of rendering.

use crate::item::Items;

/// Body armour whose picture wins over a suit's (rule 35).
const HEAVY_BODY: [&str; 2] = ["INVITEM_PlateMail", "INVITEM_BronzePlate"];
const LIGHT_BODY: [&str; 2] = ["INVITEM_LeatherArmor", "INVITEM_ChainMailShirt"];

/// The placeholder body armour.
pub const NO_ARMOR: &str = "INVITEM_NoArmor";

/// Represented object identities after a full panel rebuild. Replaying
/// carried order also retains same-picture replacements as new assignments.
pub fn slot_items(items: &Items, who: &str) -> ([Option<String>; 3], String) {
    let mut carried: Vec<_> = items.objects.iter().filter(|o| o.carried_by(who)).collect();
    carried.sort_by_key(|o| o.added);
    let mut slots = [None, None, None];
    let (mut body, mut under) = (None, None);
    for object in carried {
        let Some(kind) = items.kind(&object.kind) else { continue };
        let id = Some(object.name.clone());
        if kind.category.eq_ignore_ascii_case("Armor") {
            body = Some(object);
            slots[0] = if LIGHT_BODY.iter().any(|n| kind.name.eq_ignore_ascii_case(n)) {
                under.map(|o: &crate::item::ItemObject| o.name.clone()).or(id)
            } else { id };
        } else if kind.category.eq_ignore_ascii_case("UnderArmor") {
            under = Some(object);
            slots[0] = body.filter(|o| HEAVY_BODY.iter().any(|n| o.kind.eq_ignore_ascii_case(n)))
                .map(|o| o.name.clone()).or(id);
        } else if kind.category.eq_ignore_ascii_case("FullArmor") || kind.category.eq_ignore_ascii_case("GodArmor") {
            slots[0] = id;
        } else if kind.category.eq_ignore_ascii_case("ShortWeapon") {
            slots[1] = id;
        } else if kind.category.eq_ignore_ascii_case("LongWeapon") {
            slots[2] = id;
        }
    }
    (slots, "carried-order assignment; body/under-armour use exact item types (interface.md rule 35)".into())
}

/// The type names whose pictures the armour, hand-weapon and ranged
/// slots show (`None`: the layout's empty picture), and a note on how the
/// armour one was chosen.
pub fn slot_types(items: &Items, who: &str) -> ([Option<String>; 3], String) {
    let (slots, why) = slot_items(items, who);
    (slots.map(|name| name.and_then(|n| items.get(&n).map(|o| o.kind.clone()))), why)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountedPicture { Gold, Rocks }

impl CountedPicture {
    pub fn tag(self, count: i64) -> &'static str {
        match (self, count) {
            (Self::Gold, c) if c <= 0 => "IDBMP_2GOLD0",
            (Self::Gold, 1..=24) => "IDBMP_2GOLD1",
            (Self::Gold, 25..=49) => "IDBMP_2GOLD2",
            (Self::Gold, _) => "IDBMP_2GOLD3",
            (Self::Rocks, c) if c <= 0 => "IDBMP_2ROCKS0",
            (Self::Rocks, 1..=4) => "IDBMP_2ROCKS1",
            (Self::Rocks, 5..=14) => "IDBMP_2ROCKS2",
            (Self::Rocks, _) => "IDBMP_2ROCKS3",
        }
    }
}

/// Gold/rocks explicitly allow zero; health/potion labels suppress it.
pub fn count_text(count: Option<i64>, show_zero: bool) -> Option<String> {
    let count = count.unwrap_or(0);
    (show_zero || count > 0).then(|| count.to_string())
}

/// Q109: count observed empty-to-normal returns, not elapsed private time.
/// Missing a clock phase does not synthesize an unseen picture change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PictureBlink { remaining: u8, blank: bool }

impl PictureBlink {
    pub fn assigned() -> Self { Self { remaining: 4, blank: false } }
    pub fn blank(&self) -> bool { self.blank }
    pub fn finished(&self) -> bool { self.remaining == 0 }
    pub fn draw(&mut self, shared_ms: f64) {
        if self.finished() { return; }
        let blank = ((shared_ms / 250.0) as u64) % 2 == 0;
        if self.blank && !blank { self.remaining -= 1; }
        self.blank = blank;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::Place;
    use kq8_data::items::ItemTypes;

    fn items() -> Items {
        let line = |n: usize, name: &str, cat: &str, pic: &str| format!("item{n}=\tKQInventoryItemType\t{name}\t1\t0\t0\t0\t0\t0\t0\t{cat}\t{pic}\t1 0 0 1\tNoModeChange\t0\t0\tnone\tnone\tnone\tx.kqs\tnone\t1\t256\n");
        let list = [
            ("INVITEM_Fists", "ShortWeapon", "IDBMP_2SORD0"),
            ("INVITEM_NoRanged", "LongWeapon", "IDBMP_2RNG0"),
            ("INVITEM_NoArmor", "Armor", "IDBMP_2ARMR0"),
            ("INVITEM_ChainMailSuit", "UnderArmor", "IDBMP_2ARMR5"),
            ("INVITEM_LeatherArmor", "Armor", "IDBMP_2ARMR1"),
            ("INVITEM_PlateMail", "Armor", "IDBMP_2ARMR3"),
            ("INVITEM_Helmet", "Helmet", "IDBMP_2ARMR6"),
            ("INVITEM_ChainMailShirt", "Armor", "IDBMP_2ARMR2"),
            ("INVITEM_BronzePlate", "Armor", "IDBMP_2ARMR4"),
            ("INVITEM_Full", "FullArmor", "IDBMP_2ARMR7"),
            ("INVITEM_God", "GodArmor", "IDBMP_2ARMR8"),
        ];
        let mut text = format!("[InventoryItemTypeList]\nnumItems={}\n", list.len());
        for (n, (name, cat, pic)) in list.iter().enumerate() {
            text.push_str(&line(n, name, cat, pic));
        }
        Items::new(ItemTypes::parse(text.as_bytes()).0, [])
    }

    fn give(it: &mut Items, kind: &str) {
        it.make(kind, 1, Place::Carried("Connor".into())).unwrap();
    }

    #[test]
    fn the_slots_show_what_he_wears() {
        let mut it = items();
        for k in ["Fists", "NoRanged", "NoArmor", "Helmet"] {
            give(&mut it, k);
        }
        let (s, _) = slot_types(&it, "Connor");
        assert_eq!(s, [Some("INVITEM_NoArmor".into()), Some("INVITEM_Fists".into()), Some("INVITEM_NoRanged".into())], "a helmet changes only the number");
        give(&mut it, "ChainMailSuit");
        assert_eq!(slot_types(&it, "Connor").0[0].as_deref(), Some("INVITEM_ChainMailSuit"), "NoArmor beside a suit: the suit");
        give(&mut it, "LeatherArmor");
        assert_eq!(slot_types(&it, "Connor").0[0].as_deref(), Some("INVITEM_ChainMailSuit"), "leather: the suit's picture");
        let mut plate = items();
        give(&mut plate, "PlateMail");
        give(&mut plate, "ChainMailSuit");
        assert_eq!(slot_types(&plate, "Connor").0[0].as_deref(), Some("INVITEM_PlateMail"), "plate: the body armour's picture");
        assert_eq!(slot_types(&plate, "Wizard").0, [None, None, None]);
    }

    #[test]
    fn inventory_hud_order_count_and_observed_clock_matrix() {
        for (order, expected) in [
            (vec!["LeatherArmor", "ChainMailSuit"], "ChainMailSuit"),
            (vec!["ChainMailSuit", "LeatherArmor"], "ChainMailSuit"),
            (vec!["ChainMailShirt", "ChainMailSuit"], "ChainMailSuit"),
            (vec!["ChainMailSuit", "ChainMailShirt"], "ChainMailSuit"),
            (vec!["PlateMail", "ChainMailSuit"], "PlateMail"),
            (vec!["ChainMailSuit", "PlateMail"], "PlateMail"),
            (vec!["BronzePlate", "ChainMailSuit"], "BronzePlate"),
            (vec!["ChainMailSuit", "BronzePlate"], "BronzePlate"),
            (vec!["NoArmor", "ChainMailSuit"], "ChainMailSuit"),
            (vec!["ChainMailSuit", "NoArmor"], "NoArmor"),
            (vec!["God", "Full", "Helmet"], "Full"),
            (vec!["Full", "God", "Helmet"], "God"),
            (vec!["God", "ChainMailSuit", "LeatherArmor"], "ChainMailSuit"),
        ] {
            let mut carried = items();
            for name in &order { give(&mut carried, name); }
            // Object storage order is not the inventory's arrival order.
            carried.objects.reverse();
            assert_eq!(slot_types(&carried, "Connor").0[0], Some(format!("INVITEM_{expected}")), "{order:?}");
            let name = slot_items(&carried, "Connor").0[0].clone().unwrap();
            assert_eq!(carried.get(&name).unwrap().kind, format!("INVITEM_{expected}"));
        }
        for (picture, cases) in [
            (CountedPicture::Gold, vec![(0, 0), (1, 1), (24, 1), (25, 2), (49, 2), (50, 3)]),
            (CountedPicture::Rocks, vec![(0, 0), (1, 1), (4, 1), (5, 2), (14, 2), (15, 3)]),
        ] {
            for (count, frame) in cases {
                assert!(picture.tag(count).ends_with(&frame.to_string()));
                assert_eq!(count_text(Some(count), true), Some(count.to_string()));
            }
        }
        assert_eq!(count_text(None, true), Some("0".into()));
        assert_eq!(count_text(None, false), None);
        assert_eq!(count_text(Some(0), false), None);
        assert_eq!(count_text(Some(1), false), Some("1".into()));

        for assigned_ms in [0.0, 125.0, 249.0, 250.0, 499.0] {
            let mut blink = PictureBlink::assigned();
            assert!(!blink.blank(), "assignment installs normal before draw");
            blink.draw(assigned_ms);
            assert_eq!(blink.blank(), assigned_ms < 250.0);
            let mut returns = 0;
            for band in 1..=12 {
                let now = band as f64 * 250.0;
                if now <= assigned_ms { continue; }
                let before = blink.blank();
                blink.draw(now);
                returns += usize::from(before && !blink.blank());
                if blink.finished() { break; }
            }
            assert_eq!(returns, 4);
            assert!(blink.finished() && !blink.blank());
            blink.draw(10000.0);
            assert!(!blink.blank(), "finished stays normal in an even band");
        }
        let mut sparse = PictureBlink::assigned();
        sparse.draw(0.0);
        sparse.draw(2000.0);
        assert_eq!(sparse.remaining, 4, "missed phases do not count");
        sparse.draw(2250.0);
        assert_eq!(sparse.remaining, 3);
        sparse.draw(2750.0);
        assert_eq!(sparse.remaining, 3, "two normal draws are not another cycle");
    }
}
