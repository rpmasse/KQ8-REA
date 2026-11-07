# Ordered world loading

The required lifecycle is world Init, full world Load with each inline
object load completed in order, startup event service, then Begin on the
first drawn frame. Restored fields and eligible contacts must precede
each object's Init. The current bulk loader does not yet meet that contract.
The implementation sequence is recorded in [DESIGN.md](DESIGN.md).

## Completed preparation: shared actor constructors

`actor_recipes.rs` now constructs the CPU animation movements, doors,
chests, scripted objects and characters used by the existing renderer.
The constructors consume resolved placement and animation data without
creating rendering resources or running Init. They preserve movement/list
indices, caller-owned random state and the existing animation-start rules.
The future ordered loader can therefore construct an actor before drawing
resources are ready, and the renderer can adopt that same actor later.

The existing door and animated-object spawn paths consume these shared
constructors. Existing original-data test fixtures also consume the same
player and character constructors; they do not maintain parallel versions.
This is a code extraction from the existing recreation, with no new
claim about original behavior or a second source of actor state.

## Automated evidence

The shared viewer test build passes in **1 minute 53 seconds**:

```powershell
cargo test --locked --release -p kq8-viewer --bin kq8-viewer --no-run
```

The three affected original-data checks pass first attempt, **3/3 in
4.22 seconds**, from that same compiled executable:

```powershell
$env:KQ8_GAME_DIR = "C:\Users\Robert\Documents\Codex\KQ8\King's Quest 8 GOG"
& target/release/deps/kq8_viewer-4aebd42ecd1f894a.exe original_temple2_pushed_book_enters_its_trigger_and_starts_the_hall_sequence temple4_altar_changes_lucretos_movement_list_and_plays_the_new_clips dead_city_tower_melee_stagger_falls_and_releases_the_chain_mail --ignored --nocapture --test-threads=1
```

These cover a scripted-object trap sequence, alternate animation lists
and cues, and the tower actor's character/AI/fight/movement path. The
tower fixture places its physical mover at the same explicit platform
edge as its animation player. No new native run or manual test was used.
Logs: `kq8-artifacts/logs/ordered-loading-shared-viewer-build.log` and
`kq8-artifacts/logs/actor-recipes-original-data.log`.

## Remaining implementation

The live startup host and asset provider, per-object registration, shared
saved-world context, persistent object order, disk/regional entry wiring
and original elevator-consumer checks remain unfinished. The existing
data-only script pass cannot supply authoritative logical state because
it does not evaluate conditional branches. Merely extracting constructors
does not correct that loading behavior or establish Q201 completion.
