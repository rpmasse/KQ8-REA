//! Game rules for the KQ8 rewrite, without any drawing code.
//!
//! The rules come from the behaviour notes in `docs/behaviour/`; each
//! module names the note it follows. The game logic runs at a fixed
//! [`UPDATES_PER_SECOND`] (docs/DESIGN.md, M2 "Physics rate").

pub mod active;
pub mod main_menu;
pub mod credits;
pub mod graphics_devices;
pub mod graphics_menu;
pub mod keyboard_menu;
mod menu_layout;
pub mod options_menu;
pub mod save_menu;
pub mod anim;
pub mod camera;
pub mod camera_first_person;
pub mod camera_flight;
pub mod camera_shake;
pub mod caption;
pub mod character;
pub mod character_ai;
pub mod chest;
pub mod choice;
pub mod click;
pub mod collide;
pub mod connor;
pub mod connor_auto;
pub mod connor_inventory;
pub mod connor_mover;
pub mod connor_start;
pub mod connor_update;
pub mod conversation;
pub mod door;
pub mod emitter;
pub mod events;
pub mod fight;
pub mod ground_anim;
pub mod gui_draw;
pub mod inventory;
pub mod item;
#[cfg(test)]
mod item_traps;
#[cfg(test)]
mod prop_traps;
pub mod journey;
pub mod kept_interactions;
pub mod magic_map;
pub mod meters;
pub mod music;
pub mod named_anim;
pub mod object_checkpoint;
pub mod panels;
pub mod portal;
pub mod potions;
pub mod rope;
pub mod script;
pub mod shadow;
pub mod sound;
pub mod starting_help;
pub mod stats;
pub mod talk;
pub mod trap;
#[cfg(test)]
mod prop_placement_tests;
#[cfg(test)]
mod item_registration_tests;
#[cfg(test)]
mod item_removal_tests;
pub mod spirit_trap;
pub mod tile_puzzle;
pub mod tile_objects;
pub mod secondary_tiles;
pub mod turn_keys;
pub mod water;
pub mod terrain_hazards;
pub mod scripted_character;
pub mod weapons;
pub mod weeper;
pub mod tower_skeleton;

/// The original's update rate (`KQGame::FPSLimit 15`); every per-update
/// value in the rules assumes it.
pub const UPDATES_PER_SECOND: f64 = 15.0;
