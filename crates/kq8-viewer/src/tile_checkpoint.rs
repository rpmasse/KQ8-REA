use super::{tp, Cell, ConnorSim, Lines, Mode, Stage, Stones, TileCall, TileRun};
use crate::connor_view::SavedControllerPose;

/// The stone positions belong to the restored world. Death's hall rebuilds
/// that lookup on load (tile-puzzle.md, Edge cases).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SavedTile {
    expected: usize,
    can_exit: bool,
    lines: Lines,
    stage: Stage,
    lit: Option<Cell>,
    remembered: i8,
    mode: Mode,
    pose: SavedControllerPose,
    steps: [f32; 2],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pending_camera_hold: Option<bool>,
}

impl SavedTile {
    pub fn valid(&self, has_special: bool) -> bool {
        let cell = |c: Cell| {
            (tp::FIRST_ROW..=tp::LAST_ROW).contains(&c.row) && (0..tp::SIZE).contains(&c.col)
        };
        let neighbours = |a: Cell, b: Cell| {
            cell(a) && cell(b) && a != b && (a.row - b.row).abs() <= 1 && (a.col - b.col).abs() <= 1
        };
        let valid_stage = match self.stage {
            Stage::Keys => self.expected < tp::ORDER.len(),
            Stage::Turning { from, to, .. } | Stage::Jumping { from, to } => {
                self.expected < tp::ORDER.len() && neighbours(from, to)
            }
            Stage::Down { right, put, camera } => {
                right == (self.expected > 0) && put.into_iter().chain(camera).all(f32::is_finite)
            }
            Stage::Up { right } => right == (self.expected > 0),
        };
        valid_stage
            && self.expected <= tp::ORDER.len()
            && self.lit.is_none_or(|c| c.is_stone())
            && (-1..=1).contains(&self.remembered)
            && self.steps.iter().all(|s| s.is_finite() && *s >= 0.0)
            && self.pose.valid()
            && matches!(
                self.mode,
                Mode::Free | Mode::Fall | Mode::Land | Mode::Crash | Mode::Jump | Mode::Special
            )
            && (self.mode == Mode::Special) == has_special
    }
}

impl ConnorSim {
    pub fn restore_old_tile_entrance(&mut self, stones: Stones) {
        self.tile = Some(TileRun::start(stones));
        self.log.detail("restored Tile from the original hall entrance in an older save; rebuilt 64 stones; expects D");
    }

    pub fn saved_tile(&self) -> Option<SavedTile> {
        let t = self.tile.as_ref()?;
        Some(SavedTile {
            expected: t.expected,
            can_exit: t.can_exit,
            lines: t.lines.clone(),
            stage: t.stage,
            lit: t.lit,
            remembered: t.remembered,
            mode: self.walker.mode,
            pose: self.capture_controller_pose(),
            steps: [self.mover.step_up, self.mover.step_down],
            // A script can enter Tile after this tick's camera calls were
            // drained. The camera snapshot does not yet contain this hold.
            pending_camera_hold: self.tile_calls.iter().rev().find_map(|call| match call {
                TileCall::Hold(value) => Some(*value),
                _ => None,
            }),
        })
    }

    pub fn restore_tile(
        &mut self,
        saved: &SavedTile,
        stones: Stones,
        found: usize,
    ) -> Result<(), String> {
        self.restore_controller_pose(&saved.pose, saved.mode, "Tile")?;
        self.mover.step_up = saved.steps[0];
        self.mover.step_down = saved.steps[1];
        self.tile = Some(TileRun {
            stones,
            expected: saved.expected,
            can_exit: saved.can_exit,
            lines: saved.lines.clone(),
            stage: saved.stage,
            lit: saved.lit,
            remembered: saved.remembered,
        });
        self.tile_lit_changed = Some(saved.lit);
        if let Some(hold) = saved.pending_camera_hold {
            self.tile_calls.push(TileCall::Hold(hold));
        }
        self.log.detail(format!("restored Tile: {found} of 64 stones rebuilt; completed {} letters; stage {:?}, lit {:?}",saved.expected,saved.stage,saved.lit));
        Ok(())
    }
}
