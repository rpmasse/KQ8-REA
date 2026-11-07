# Ordinary terrain-liquid damage

Implementation B, 2026-11-07. Rules come from study A's
[terrain-hazards.md](behaviour/terrain-hazards.md), rules 1–13. This
change implements the ordinary non-`ontop`, non-special-status path.
No original executable, decompiled material or game assets were copied.

## Behavior and integration

The source priority is poison, fast poison, then swamp. Their original
world-script settings provide the amount: zero means a 1,000-point
attack, while positive settings use Connor's level and, for amounts
below one, the shared world-clock phase. Standing 250 units below the
liquid surface qualifies; the separate water-animation threshold stays
600. The complete material flags are checked against Connor's immunity
mask. `KQConner::addImmunity` adds the low 16 bits of its decimal argument.
Generic objects' initial all-immune case is represented by the reusable
calculation; no new monster damage behavior is introduced.

After Connor's existing movement update, the viewer samples the final
surface, material and feet position. A qualifying result queues a normal
`KQAttackEvent` from the world. Fire overrides ice when choosing its
kind; bare fast-poison material uses Normal. The ordinary hit handler
still owns randomness, armor, reaction eligibility, rope/dying guards
and death. No direct health subtraction or potion-specific hazard
exemption is added.

World hits suppress the ordinary ouch without advancing its one-second
timer in the no-reaction branch. Reactions retain their existing timer
behavior. A separate gate plays poison1, poison2, poison3 in order,
rearming after 2.5 world seconds. Hits during that interval do not
postpone it. Armor's zero-loss result and the fatal hit still reach this
sound branch; already-dying and rope hits do not.

`terrain_hazards.rs` contains the arithmetic, immunity and sound state.
`active.rs` queues the ordinary event and exposes the command;
`fight.rs` carries the source distinction into reaction/audio handling.
`door_view.rs` connects the post-movement query; `connor_view.rs` only
changes the ordinary ouch decision. `journey.rs` retains immunity and
the sound gate across Rust saves/travel, with defaults for older saves;
`save_slots.rs` rejects invalid sound continuation. This is **Rust
continuation**, not proof of original save serialization or clock
restoration.

## Grouped verification

The focused core suite **passes 3/3**, with a 63-second release compile
and no failed cases. It groups all source priorities, signed-zero
settings, level arithmetic, stride/rounded-slot/whole-second boundaries,
249/250 and 599/600 depths, full-material immunity, fire/ice priority,
command mask/usage behavior, ordinary protection 100 and fatal
protection 11/50, rope/dying guards, world sound timing/cycling, and Rust
Journey roundtrip/legacy defaults.

```powershell
cargo test --locked --release -p kq8-core terrain_hazards_grouped -- --nocapture
```

The original-data matrix reads the twelve regional WORLD.CS files and
installed regional overrides directly from the named GOG installation.
The representative native fixture uses the actual Dead City deep river
material, first with temporary protection 100 to cover repeated zero-loss
dispatch/audio, then protection 11 for ordinary death. Fixture commands
are explicit setup, not earned campaign evidence. They do not change the
original Time settings or write to the game installation. The check
compares Options.cs bytes, uses audio dispatch logs with the device off,
and does not involve manual playtesting.

```powershell
$env:KQ8_GAME_DIR=(Resolve-Path -LiteralPath "../King's Quest 8 GOG").Path
cargo test --locked --release -p kq8-viewer --test terrain_hazards -- --ignored --nocapture --test-threads=1
```

The original-data and native fixture checks **pass 2/2 on their first
run in 49.15 seconds**. The viewer build takes 154 seconds; a test-only
regional-override path correction identified during review rebuilds in
2.15 seconds before the native launch. There are no failed test cases
in this batch and no successful game run is repeated.

The follow-up coverage audit catches a study-note naming error before
publication: the registered command is `addImmunity`, which original
Swamp WHISP.CS Cue 10 uses with 256. `addImmune` is only the misspelled
original usage message. The dispatcher and command matrix now use the
registered name, retain the diagnostic spelling, and do not add an
unproven alias. The command matrix is split into its own focused case;
the original Wisp callback is added as a headless original-data check.
Only those changed checks are rerun; the arithmetic, Journey and native
dispatch/audio/death results above remain valid. The changed command and
original Wisp callback checks **pass 2/2**, with a 61-second core compile
and no native launch. The callback grants swamp immunity while leaving
poison eligibility unchanged. An earlier command used a short name with
`--exact`, selecting zero tests after a 57.76-second compile; it is not
counted as a check. The asset check stays in the core module to avoid
rebuilding the native viewer for headless script execution.

```powershell
$env:KQ8_GAME_DIR=(Resolve-Path -LiteralPath "../King's Quest 8 GOG").Path
cargo test --locked --release -p kq8-core terrain_hazards_grouped_immunity -- --include-ignored --nocapture
```

Evidence is under
`kq8-artifacts/terrain-hazards/1794050003478390700/kq8-artifacts/logs/`:

- `doors.log`: the actual material is `0x80`, surface depth 864, selecting
  FastPoisonTime and a Normal base-1,000 world attack from update 7.
- `fight.log`: protection 100 keeps health 120. Poison sounds start at
  updates 7, 45, 83 and 121, cycling 1, 2, 3, 1 with the 2.5-second
  gate resolved on the 15-Hz update clock. The ordinary no-flinch branch
  logs that its combat ouch is suppressed.
- `sound.log`: all three original poison files resolve from GAME/SOUND
  and dispatch at Connor's actual position (32000,198000,0), group 0.
- At update 126, protection 11 permits loss 576: health 120 becomes
  -456 and requests Normal Die. `viewer.log` records the `die` movement
  completing at update 170, then the normal death-menu hold ending at
  update 215. Later hits are ignored because Connor is dying.
- The test compares original Options.cs bytes before all later log
  assertions; they are unchanged. The twelve regional setting checks,
  including installed Barren/IceWorld overrides, also pass.

## Limits retained

The `ontop` sinking/control branch is excluded rather than falling
through into ordinary liquid damage. Its control gate and gravity/
keep-on-ground restoration remain open, as do the special-status
lifecycle and exact GodArmor path. Negative/nonfinite settings and
nonpositive levels are outside the studied original data and produce no
new speculative damage in this implementation.

The original hazard routine has no displacement-change guard; an
unchanged result of a move request still qualifies. Its invocation on
every original idle-controller path is **not tested**; the Rust normal
update already requests the mover's step while standing. Original
world-clock, immunity and poison-sound save restoration are **not
tested**. No native original-game comparison or manual test is claimed.
