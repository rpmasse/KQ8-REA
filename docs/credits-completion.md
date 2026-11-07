# Original credits completion

This M4 batch implements [credits-completion.md](behaviour/credits-completion.md)
from study session A and the read-only original GUI, font, palette, picture
and message data. It replaces the earlier five-second cards, Enter/click
skipping, early score snapshot and return-to-Options behavior.

Both menu Credits and `doCredits` enter the same screen. Cases 1–25 are
selected numerically, preserving their authored line breaks, spaces and
mixed font order. A real-time absolute deadline uses two seconds plus
half a second per line, with twelve extra seconds for the final card.
A late update advances only one card and starts the next deadline from
the current time. Missing card text leaves a cleared screen and retries
that case after two seconds; malformed rows/formats are rejected safely.

At 640×480 and 800×600, row spacing is recalculated in output pixels,
fonts retain their native glyph sizes and the final photograph keeps its
documented ten-pixel top. Credits uses `saveload.ppl` when available,
falling back to `Menus.ppl`; the shared GUI rasterizer now accepts this
separate palette without changing ordinary menu labels or drawing.

Only preparation of card 25 samples the active-game flag and Connor's
level/current experience. An inactive session shows zero even if a
Connor object remains. Checked arithmetic rejects invalid levels and
overflow; the displayed score is not capped to the authored possible
total. The original English message spells that denominator `6631364`,
and the substituted score also has no grouping commas. `doCredits`
accepts no arguments and queues only a screen request.

Ordinary Enter, Space, arrows and clicks do not skip. Escape and final
expiry start a one-second real-time theme fade, then restore Main with
its active-game selection, save scan, menu palette, theme and input
delay. Reentry starts at card 1 without old text or the photograph.

## Batched checks

From the repository root:

```powershell
$env:KQ8_GAME_DIR = (Resolve-Path -LiteralPath "../King's Quest 8 GOG").Path
cargo test --locked --release -p kq8-core credits_grouped -- --ignored --nocapture
cargo test --locked --release -p kq8-viewer --test credits_menu -- --ignored --nocapture
```

The core batch has two matrices. The controlled-clock matrix covers
deadlines, delayed updates, ordinary input, both exit routes and fade,
late score sampling, level boundaries, inactive sessions, malformed
data and missing-card retry. The original-data matrix covers all 25
cards/132 lines/128 nominal seconds, representative sparse/dense/final
row families at both resolutions, original palettes and fonts,
photograph placement, score/reentry and final-script request ordering.
The native test is one abbreviated Options-to-Credits-to-Main flow;
it does not replay every card or rerun verified movies.

Validation in progress on 2026-11-07:

- The controlled-clock/input/score matrix **passed** in the initial core
  run (68-second build; the two-test run took 0.04 seconds).
- The original-data matrix initially failed because the study note
  counted 134 rows instead of 132. Study A confirmed that a terminal
  newline does not create another row: cases 19 and 20 have 8 and 12
  rows. The note, expected total (128 seconds) and dense-card geometry
  were corrected. The parser/timer implementation did not change.
- The focused retry reached the palette assertion and failed because
  the test assumed the optional `saveload.ppl` existed. The current
  install has only `Menus.ppl` in the shared GUI data. The implementation
  already retained that documented fallback; the assertion now checks
  the actual available palette. This retry built in 70 seconds and
  ran for 0.02 seconds.
- The corrected original-data matrix **passed**, 1/1, in 0.08 seconds
  after a 57.77-second build. Both matrices now have successful evidence.
- `cargo test --locked --release -p kq8-viewer --test credits_menu
  --no-run`: **passed**, 140 seconds. This compiled the affected viewer
  and integration test without launching a window.
- The already-built `credits_menu-908f723ee9c1f91f.exe` harness was then
  invoked directly with `--ignored --nocapture`: **1 passed, 0 failed**,
  12.14 seconds. This avoided rebuilding against the next parallel source
  batch. Its single 360-frame window logged Credits card 1, ignored the
  tested no-skip inputs, faded, returned to Main and exited successfully.
  No second native route or successful check was repeated.

The unaffected successful clock matrix is reused. Logs retain both
failures as `kq8-artifacts/credits-core.log` and
`kq8-artifacts/credits-core-corrected.log`. The successful focused result
is `kq8-artifacts/credits-core-palette-corrected.log`. Viewer build/native
logs are `kq8-artifacts/credits-viewer-build.log` and
`kq8-artifacts/credits-native.log`. Detailed native logs are retained in
`kq8-artifacts/menu-tests/credits-1794053663507441800/`. The tested viewer
SHA-256 is
`46404DA159BBCFDD4CB269180A3DDCE9E7AA2F3249DD39030751506809ABA818`.
The affected files pass `git diff --check`. Existing movie checks were
reused, not repeated.

## Limits

No manual testing or original-runtime visual comparison is included.
The full New Game-to-ending campaign remains an M6 integration gate.
In particular, the existing viewer consumes the ending's `EndGame`
request before the final movie and credits, removing its active-game
state; late credits sampling consequently yields zero on that path.
The original ending's game/Connor lifetime boundary is **not verified**
by this credits batch. The established late sampling rule is preserved;
an early score snapshot is not used to hide that separate uncertainty.
Modified Escape, OS focus/IME behavior, audio synchronization with the
ending movie and original palette-failure execution remain unverified
as documented by study A. Native validation with sound disabled can
verify scheduling and exit state, but not audible fade quality.
