# Live Lucreto finale test provenance

`crates/kq8-viewer/tests/live_lucreto_finale.rs` uses the installed GOG
Temple 4 `altar.cs` Cue 6/7 to enter the last fight, `monsters.kq` and
`lucend.kq` for the two forms, `vortrap.kq` for the trap placement, and
`vortrap.cs` for its victory animation and ending commands. The test
reads these files at runtime through the game loader; none are stored
in the repository.

The distance handoff comes from `docs/behaviour/lucreto-forms.md` rules
1–5. Sword hit, reaction and protection expectations come from
`docs/behaviour/fighting.md` rules 16 and 23. Trap contact and scripted
events use `docs/behaviour/traps.md`. The shield input uses
`docs/behaviour/potions.md` rules 17 and 33. The check makes no claim
about original executable runtime, movie playback or final image
fidelity.
