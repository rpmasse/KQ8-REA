# Item-drop continuation provenance

Study A's [items.md](../behaviour/items.md), rule 11, and
[traps.md](../behaviour/traps.md), Q203, supply common inventory removal,
old-identity registration, Rock versus non-Rock placement and callback
ordering. The installed item table and bounds supply categories and shapes.
[object-lighting.md](../behaviour/object-lighting.md) supplies the existing
item activity/distance rules shared with the renderer.

Implementation B used these prose notes, original data and existing Rust
code, without executable or disassembly access. Carried-pose serialization
preserves recreation continuation; old snapshots missing that pose use an
explicit origin fallback. Core matrices pass and viewer hooks compile;
the production ordered loader and original native comparison remain open.
[Evidence and commands](../item-drop-continuation.md).
