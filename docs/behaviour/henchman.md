# Daventry's Henchman encounter (Q190)

Study session A, 2026-11-06. Sources: the installed GOG 1.03
`Mask.exe` (static study of the ordinary monster, Wait and Special
controllers), and the original Daventry files named below. This is a
description of behavior, not executable-derived implementation code.
No original-game runtime comparison or manual play test was performed.
The failed Rust campaign replays were not used to infer original rules.

## Purpose

Connect the earned windmill axe cutscene to the fight outside it, and
identify which ordinary fighting rules apply to the Henchman. In
particular, a hit does not interrupt his authored attack.

## Rules

1. **An ordinary monster.** Daventry's `Henchman` is a `KQMonster`
   created by the monster list, with the `henchman` type. He has no
   separate boss controller. His own script handles the projectile
   refusal and death presentation described below. Ordinary sight,
   active-list membership, alarm, Wait, damage and dying rules otherwise
   apply (`character-movement.md`, `fighting.md`).
2. **The axe starts the scene after the pickup.** A successful reachable
   pickup gives the Small Axe and 25 experience and loads `Henchtp`.
   Its first `On` sends the axe Cue 5. That removes the trigger, starts
   the camera sequence and weapon change, places Connor by the exit and
   sends him on the scripted walk. The weapon change may delay the walk
   as described in `anim-scripts.md` rule 32.
3. **The walk's completion begins Connor's cutscene list.** Cue 20
   places Connor at (141427, 249772, ground), sets the cut camera and
   subscribes the axe to both actors' movement ends. Connor plays
   `cdhm1`, `cdhm2`, `cdhm3`, `cdhm4` from `henchman.anm`. Their end
   notices advance the authored camera/music/speech scene. `cdhm4`'s
   end sets the Henchman's home and position to (136500, 250000, 15045),
   heading 1.57, creates `clicker2` and starts his Special list `cdhm5`.
4. **`cdhm5` ends into Wait.** A one-movement Special list is exhausted
   when its movement ends, and requests **Wait**, which asks for `Stop`.
   This ordinary state handling precedes the listener's Cue 300. In that
   cue the axe moves his home and position to **(137200, 250000,
   15045), heading 1.57**, subscribes to state changes and asks for
   Henchman's final **1000/3/0/6, sequences 1–4**. The speech is an
   ordinary `speak`, not a conversation/Talk-state encounter controller.
   The scene then removes both end-loop subscriptions and returns to the
   third-person camera.
5. **Final speech completion only releases the scene.** The matching
   `TalkComplete` clears the axe's state-change subscription, restores
   follow-Connor and player control, and deletes the axe's scene object.
   It does not set a fight state, a target, health, damage, immunity or
   a new attack deadline. In particular there is no added first-swing
   grace period starting at the last spoken line.
6. **His normal target is Connor, acquired normally.** After Special he
   is in Wait. Existing active/target/sight state is not cleared by this
   transition or by the final speech. If not already active/alarmed he
   joins the ordinary active list on sight and acquires Connor when
   within the 15000 alarm radius, subject to the world's list capacity
   (Daventry 6). The ordinary 0–1 second alarm reaction applies when an
   alarm is newly acquired. Sight is sampled every 15 updates. Do not
   make final speech completion create an unconditional attack or target.
7. **A stationary guard.** His type's movement mode is `guard`, with
   chase/home radii zero. In Wait he follows Connor by turning on the
   spot and attacks if Connor enters reach; he does not chase a
   retreating Connor. His movement named `walk` is also the same
   `hm_stop` standing model, with no locomotion supplied by that model.
   The cutscene's scripted placements are not chase movement.
8. **Attack.** The ordinary Wait attack requires sight, distance
   **strictly less than 2500**, an elapsed attack deadline and a current
   movement that can be replaced or has ended. It asks for `Attack`,
   which lasts **1.0 second**, sounds at phase 0.0625 and tries its hit
   at phase 0.5. His next attack is due **0.5 seconds after the completed
   attack**, giving a nominal 1.5-second start-to-start interval when
   nothing else intervenes. These are game-clock times, sampled at
   15 updates per second; the phase crossing may occur on the next tick.
9. **No interruptible attack window.** `henchman.anm`'s `attack` has
   both loop switches zero and **no `interruptable` cue**. The running
   swing therefore cannot be replaced by `react` at its start or later.
   A nonlethal axe hit during it takes health but does not cancel the
   swing. A lethal hit still enters Die. There is no Henchman-specific
   immunity to health damage during attacks.
10. **Hit reach is checked again.** The `Do` hit uses the ordinary
    feet-to-feet 3D test: distance **at most 2500** and Connor strictly
    in the front half. If Connor's backward key is held at that moment,
    the hit reach is **3125** (1.25 times reach). Stepping just outside
    2500 while still holding backward is insufficient. He keeps turning
    during the attack. His hit is **Normal**, strength **21**; neither
    his visible spear tip nor `clicker2` supplies a separate damage test.
11. **React.** In Wait, a surviving hit selects `React` only if the
    ordinary damage roll exceeds **0.9 times incoming strength**, the
    current movement can be replaced, and the previous reaction's
    end is **more than 2 seconds** in the past. His `react` lasts
    **1.0 second**, cannot be interrupted, and has no special stun timer.
    While `attack`, `react`, `laugh` or `cdhm5` is uninterruptible he
    can take health damage without flinching. There is no `reactPause`
    or `stunPause` override for this ordinary monster.
12. **Wait attack-deadline correction to `fighting.md` 11/22.7.** The
    standing hurt path repairs an attack deadline marked "never" only
    on the path that actually selects a reaction (or a forced reaction).
    A health-losing hit that cannot make him react does **not** repair
    that deadline or shorten his running attack. Attack-end and
    entering-Wait/Move repairs still apply. The older statement that
    every hit repairs the deadline was too broad for Wait. This matters
    for the Henchman's uninterruptible attack; do not schedule an extra
    attack merely because he was hit halfway through it.
13. **Projectile refusal.** His live collision type is **0x4042**;
    the extra 0x4000 is the projectile-immunity flag. The ordinary
    `ProjImmune` script callback asks him for Special `laugh`, after
    which the ordinary exhausted-list rule returns him to Wait. It is
    not a general melee-damage shield. His normal health-loss rules have
    no weapon-name test requiring the Small Axe specifically.
14. **Death.** Ordinary health below zero enters Die, clamps health to
    zero, retires him from the active list and removes collision/gravity
    as in `fighting.md` rule 30. His script's self state-change listener
    sees new state **8** and takes over the camera with hands off. His
    `die`, `firedie` and `icedie` all use the same **6.0-second**
    `hm_die` animation and its opening sound. At the ordinary death end
    Connor receives **30 experience**, the applicable corpse is made,
    the script receives `Die` and the actor is removed. For Normal the
    corpse is `hm_dead.kqs`; Fire/Ice retain the ordinary corpse rules,
    not an inferred exception from their shared animation.
15. **After the death animation.** The `Die` script restores player
    control and follow-Connor, deletes `clicker2`, asks Connor for
    1000/3/14/65 and pops `mbattle.aud`. The drop list contains one
    Strength, two Crystals and one Invulnerability item; these follow
    the ordinary death/drop path, not an award of those items to Connor
    at the start of the encounter.

## Numbers and units

| Quantity | Value at normal combat level |
| --- | --- |
| Health / hand damage / ranged damage / protection / experience | 300 / 21 / 21 / 5 / 30 |
| Alarm radius / chase radius / home radius | 15000 / 0 / 0 game units |
| Attack reach / backward-key hit reach | 2500 / 3125 game units |
| Attack duration / hit phase / repeat delay | 1.0 s / 0.5 / 0.5 s after end |
| Reaction duration / minimum gap after its end | 1.0 s / strictly more than 2.0 s |
| Death animation duration | 6.0 s |
| Final scene placement | 137200, 250000, 15045; heading 1.57 radians |

Combat difficulty scales health and damage by the ordinary rule. The
animation durations above are from the models, not the advisory `frames`
keys: `hm_react.kqs` has 16 sequences and a 1.0-second loop despite
`frames4=15` in the movement list.

## Data used

Daventry `monsters.kq`, `henchman.kq`, `henchman.cs`, `henchman.anm`,
`henchtp.cs`, `ax.cs`, `clicker2.kq`; shared `Monstats.mtl` and the
ordinary bounds/items tables; models `hm_atack.kqs`, `hm_react.kqs`,
`hm_stop.kqs`, `hm_die.kqs` and `cdhm1`–`cdhm5`.

## Edge cases

- **Zero health is alive.** The ordinary lethal threshold is below
  zero. A fixed number of hits should be checked using actual rolls,
  protection and remaining health, not a supposed scripted hit count.
- **Insufficient replay tactics are plausible.** Derived from the
  ordinary damage rules, at hand damage 35 and protection 5 a normal
  axe hit loses roughly 24–42 health. Henchman's 300 health therefore
  needs 8–13 such hits. His strength-21 hit against protection-2 Connor
  loses 4–20 health; 30-health Connor can die in two hits. These ranges
  do not prove the Rust encounter right or wrong. A standing trade of
  blows is not a demonstrated winning strategy.
- **A retreat is a useful diagnostic.** The ordinary guard does not
  close distance. A non-backward-key sample beyond 2500, or a backward
  sample beyond 3125, must miss at his `Do`, even if his swing began
  within reach. This supplies a source-backed next check, not an
  untested promise of a complete winning replay.

## Do not reproduce

No new exception is recommended. Preserve the uninterruptible swing,
zero chase radius, ordinary reaction gate and authored death scene.
Do not weaken his stats or award healing items to make a replay pass.

## How to check

Use numerical logs, not visual inspection. These checks are specified;
they have **not been run against the original runtime** in this study.

- Record `cdhm5` end, state after end, final speech completion, player
  control, position/home, target, active membership and attack deadline.
  Expect Wait and the ordinary acquisition rules above; speech completion
  must not reset the deadline or install a special combat state.
- At a fixed in-reach placement, log attack start, sound, `Do`, end and
  next start. Apply one nonlethal melee hit early and one late in a
  separate attack. Health changes; the attack and its one `Do` continue.
- From Stop outside the two-second reaction lock, use a roll above the
  threshold: one React. Another hit during React and hits during its
  following lock may lose health but do not restart React.
- Log feet distance and backward-key state at `Do` for 2499, 2501,
  3124 and 3126 units. Check the corresponding reach and front tests.
- Retreat beyond reach and wait through several attack intervals.
  Henchman's feet remain at the guard spot. Turn changes are expected.
- On a lethal normal hit, check one dying state, immediate loss of
  collision, one six-second animation, one 30-experience award, the
  authored drops, one death callback and control/camera/music restoration.

## Open questions

- Exact initial active/target values at each frame of an uninterrupted
  original playthrough were not captured. The transition and acquisition
  rules are traced; no fixed absolute first-attack tick is asserted.
- `clicker2.kq` names `clicker2.cs`, but two direct lookups in the
  Daventry KQ archive failed. Its auxiliary click-script behavior was
  not resolved. The Henchman's own hit is the ordinary `Do` path and
  does not call that helper.
- A winning ordinary-controls Rust campaign replay remains the
  implementation session's work; this study changes no replay or Rust.
