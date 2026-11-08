# Top-bar input completion

Pointer-triggered opening now respects interface.md rule 12: sword, bow and
the two rope-action cursor pictures cannot open the top inventory bar.
Ordinary play/item cursors can. Existing right-button and moving-panel
restrictions, closing behavior and the independent news popup timer remain.

Physical and named Mask clicks share the existing handsOff-only menu action
before item lookup. The named automation path previously discarded the
returned menu action. Physical Mask clicks were already supported; this
change does not claim they were broken. Other inventory results cannot
clear a previously requested menu.

One grouped check drives the production helpers and panel state, including
opening/closing sounds, six cursor classes, drag/movement restrictions,
news timer retention and Mask dispatch. It passes first attempt. It ran
together with the movie failure matrix: **2/2 in 1.53 s** after the shared
2m56s build. No existing inventory-use or pickup route was repeated.

```powershell
cargo test --locked --release -p kq8-viewer --bin kq8-viewer top_bar_pointer_eligibility_and_mask_dispatch_matrix -- --nocapture
```

Evidence: `kq8-artifacts/logs/m4-input-and-movie-failures.log`.
Sources: `behaviour/interface.md` rules 12–14 and 41,
`behaviour/main-menu.md` rule 7, and existing panel/item dispatch.
This is headless input/state evidence; a new physical-pointer native route
is **not tested**. Inventory picture/count/blink fidelity is separate work.
