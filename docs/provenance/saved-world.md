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
