//! Forced Choice replaces the old owner; it does not finish that action.
//! Source: characters.md, "Forced replacement and the previous action".
use super::*;
use kq8_core::active::{Active, Request};

impl ConnorSim {
    /// Full-controller boundary before borrowing the ordinary dispatcher.
    /// Internal tick callbacks can reenter Choice from Swap; dedicated owners
    /// enter through this wrapper, including Rock/Hammer completion requests.
    pub(crate) fn choice_replaces_owner(&self, incoming: &Request) -> bool {
        let remembers = self.forced_choice.is_some() || matches!(incoming, Request::ConnorChoice { .. });
        let dedicated = self.tile.is_some() || self.secondary_tile.is_some()
            || self.scales.is_some() || self.rope.is_some() || self.rock.is_some() || self.throw.is_some();
        remembers && (!self.choosing() || dedicated)
            && !matches!(incoming, Request::ConnorTransition(kq8_core::conversation::StateRequest::RetryPending))
    }

    pub(super) fn discard_owner_for_choice(&mut self, active: &mut Active) {
        // A remembered Choice may coexist with the action it temporarily
        // admitted. Discard the old ring as well: the fresh entry calls
        // StartChoice, even if a dedicated owner happened to use Free mode.
        self.choice = None;
        let tile = self.tile.take().is_some();
        let secondary = self.secondary_tile.take();
        let scales = self.scales.take().is_some();
        if tile || secondary.is_some() || scales { self.walker.lean_allowed = true; }
        if tile {
            active.world.requests.push(Request::CameraHold { on: false, at: "Tile exit before forced Choice".into() });
        }
        if let Some(old) = secondary {
            self.walker.controller_owned_input = false;
            if old.kind == kq8_core::secondary_tiles::Kind::Barren {
                if self.choice_music_available {
                    active.test_command("KQMusic::pop mlava.aud", "Tile3 exit before forced Choice");
                }
                if !old.solved {
                    // Barren's coordinates are transposed in its authored name:
                    // seven movement columns by five rows -> tile00..tile46.
                    for col in 0..7 {
                        for row in 0..5 {
                            let name = old.kind.name(kq8_core::secondary_tiles::Cell::new(col, row));
                            if active.world.known.contains(&name.to_ascii_lowercase()) && !active.world.is_deleted(&name) {
                                let now = active.world.now();
                                active.world.queue.send(now, 0., &name, "KQCueEvent", &["2402".into(), "0".into()], "Tile3 exit before forced Choice");
                            }
                        }
                    }
                }
            }
        }
        if self.rope.take().is_some() {
            self.rope_ended.clear();
            self.rope_was_climbing = false;
            active.world.connor_roping = false;
            // rope_model is a separately created animated world object.
        }
        let throwing = self.rock.take().is_some() | self.throw.take().is_some();
        if throwing {
            self.throw_ended.clear();
            self.throw_cues.clear();
            // No restored hand, projectile, view, handsOff or world deletion.
        }
        if self.walker.mode == Mode::Talk {
            active.force_conversation_exit();
        }
        // Physics and actor settings are deliberately untouched. The following
        // Choice entry alone requests Stop and clears the input state.
    }
}
