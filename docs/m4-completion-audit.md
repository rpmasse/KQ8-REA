# M4 completion audit — 2026-11-08

**M4 is complete (100%) against its planned implementation and automated acceptance checks.** This audit follows the M4 requirements in DESIGN.md
and retains successful evidence rather than repeating unrelated tests.
It does not claim campaign completion or final original-driver equivalence.

| Requirement | Retained evidence | Outstanding work |
| --- | --- | --- |
| Menus, options, pause and save-screen integration | original-interface-resolutions.md and later native menu/options/keyboard/save checks | No additional implementation gap found in this bounded audit; later graphics batches supersede early limitations. |
| Inventory, HUD, item use and equipment | inventory-hud-completion.md, inventory-popup-completion.md, weapon-admission-completion.md, controller-exit-completion.md | Current-equipped view/list selection, Swap input locks, assignment/checkpoints and duplicate-load corrections pass the affected grouped matrices. |
| Help, prompts, cursor and map | help-cursor-lifecycle.md and later final-admission checks | Retain successful geometry, pause, navigation and tutorial scheduling evidence. |
| Conversation, choices, speech, captions, camera and interruption | conversation-admission.md, controller-exit-completion.md and retained native Wizard approach/speech/reload/exit checks | Forced replacement, shared Talk exit, midpoint camera aim, callback/queue order and camera input reset now pass grouped checks. |
| Graphics controls, persistence, palette and live changes | graphics-runtime-completion.md | Closed by the final production-default console route: actual exclusive 640/800/640 plus submitted-frame epochs, menu return/Quit and preference/original-file integrity. |
| Movies, errors, skip/return and credits | movie-failure-completion.md, credits-completion.md and retained native flows | No additional M4 implementation gap found in this bounded audit. Campaign ending reachability remains M6; final comparison and decoder distribution remain release work. |

The latest weapon/inventory batch initially passes eight core and nine
viewer checks, with one documented fixture correction. Review then found
the list-selection and loading cases above. The original study clarifies
that explicit list refresh and accepted F2 select by current mode, view
and equipped item; a saved old drawn-item identity is not the rule.
Those corrections and the controller-exit changes now pass the shared release
build (2m41s), two core checks and seven viewer checks (1.31s). The first
private-helper compile error and its public-wrapper correction are retained
in controller-exit-completion.md. Successful unrelated checks were not rerun.

The audit did not rerun tests, open a native game window, perform manual
testing, or inspect the original executable. It found no other concrete
M4 implementation omission in the reviewed requirements and evidence.
The gate identified by that audit was actual original-size exclusive
fullscreen on a supported host; narrow policy/window checks alone did not
close it.

Subsequent root checks found the Vulkan/native-HWND failures retained in
graphics-runtime-completion.md. The final implementation defaults Windows
games to the verified DX12 DirectComposition path while preserving explicit
choices, and requires an actual matching frame submitted for presentation.
Release build PASS 2m47s; new checks PASS 3/3 (0.01s); affected frontend PASS
1/1 (0.41s). The final native route, with renderer overrides absent, PASS
3.68s (child route 3.23s), artifact `1794119657462978500`. It confirms actual
640/800/640 and submitted epochs 1/2/3, duplicate 800 no-op, ordinary
Options/Main/Quit, correct isolated preferences and unchanged original
Options. Desktop dimensions were restored and no child remained.

The final read-only review found no blocking bridge or policy issue, and
the final native run closes the identified M4 gate. This does not close
M5/M6, prove original-driver pixels, or prove GPU scanout. Explicit Vulkan
and DX12 HWND overrides retain documented host-specific failures. Actual
Vulkan mismatch restoration failed; successful exceptional native recovery
and Winit internal-panic recovery remain unverified. No manual playtesting.

## Required-mode callback scope

The generated script-coverage inventory records two authored
`KQConner::setRequiredWeaponMode` uses, both in Dead City's `towSklTp.CS`.
Reading that installed archive script confirms an On request for mode 0
and an Off request for -1; both omit the optional callback. No matching
regional patch file was present in the inspected installation. This is
evidence about the shipped script path, not proof of arbitrary dynamically
constructed commands. The source study's unresolved destructive/reentrant
`WrongWeapon` callback behavior does not identify a missing authored M4
scene. The ordinary correction and callback rereading mechanism still have
the grouped checks described in weapon-admission-completion.md.

The CLI initially needed the explicit game path, then its documented
`vol:<World>/<volume>/<NAME>` archive syntax; a loose-file lookup did not
find this archived script. No original data was modified or added to Git.
