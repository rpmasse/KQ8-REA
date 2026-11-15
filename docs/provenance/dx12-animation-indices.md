# DX12 animation-index correction

This is a rendering-library compatibility correction, not a newly inferred
original-engine behavior. Implementation session B used existing
`anim_gpu.rs`, `object_light.wesl`, and the locked Bevy 0.20.0, WGPU 30.0.1
and Naga 30.0.1 sources. Bevy's `forward_io.wesl` documents the separate
scalar argument; Naga's HLSL writer and fixed-function input recognition
explain why a builtin inside a struct passed to a helper loses its offset.

Synthetic numerical GPU readback confirms the wrong animation-tag lookup
through the struct helper and correct lookup through a scalar argument.
Original data was used only by the separate CPU packing and production
scene checks. No original executable, decompilation or disassembly was
read, and no game data is included in the regression fixture.

Evidence, limitations and reproduction: [rendering regression](../geometry-rendering-regression.md).
