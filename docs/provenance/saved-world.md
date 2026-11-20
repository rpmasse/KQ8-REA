# Shared saved-world core provenance

The source is Study A's [saved-world-events.md](../behaviour/saved-world-events.md),
rules 1–15 and bounded follow-up, plus the clarified ordinary-trap Lookup
rule 7 in [scripted-controller-commands.md](../behaviour/scripted-controller-commands.md).
Implementation B uses these prose descriptions, existing Rust code and
original asset/script data only. No executable, disassembly or `re/`
material was consulted by implementation B.

The studies establish persistent event-family selection, queue ordering,
clock precision and reassignment, saved target lifetime binding, startup
service and listener precedence. Rust uses its existing text-token event
payload model; unstudied native resource or rare subtype codecs are not
claimed. The existing fixed 15-update scheduler is separate from the traced
clock contract. The core tests do not establish production disk/regional
loading order or original runtime parity. [Evidence and commands](../saved-world-core.md).

The teleporter-pad reload reconciliation in `save_slots.rs` follows
[`magic-map.md`](../behaviour/magic-map.md) rule 36 (`onTeleport` sets
the transient map flag) and the saved trap membership described in
[`traps.md`](../behaviour/traps.md). Deriving the flag directly from an
occupied restored pad is our continuation choice; it avoids replaying
the pad's On script when the saved trap already contains Connor. A
focused test and fresh-process Gnome pad load verified this Rust path,
not original runtime parity.
