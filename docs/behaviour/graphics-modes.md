# Graphics modes and resolution changes

## Purpose and scope

Describe the original Graphics menu, its device choices, and the shared
interface rules for 640 × 480 and 800 × 600. These are original Windows
presentation rules, not a proposal for new resolutions or graphics features.
This note answers the bounded, reusable parts of Q207 and Q108.

Evidence is a static study of the read-only GOG 1.03 `Mask.exe` and the
installed game data on 2026-11-07. Original execution, failed-driver
experiments and manual visual comparison are **not tested**. The findings
below are marked traced or data; unresolved behavior is listed separately.

## Rules

### Device choices and availability

1. **Three fixed rows, traced/data.** The Graphics screen has DirectDraw,
   Direct3D and 3Dfx rows. Their captions come from the game's text tags;
   the placeholder text in `Graphics.gui` is not the final caption. The
   chosen row is displayed as `> name <`. The resolution rows use the
   same selection markers.
2. **Unavailable rows are disabled, traced.** The menu retains the row,
   turns off its selection eligibility, and draws the disabled text with
   a checker pattern. It does not remove the row and close the gap.
3. **DirectDraw means Software, traced.** The saved device name for the
   DirectDraw renderer is `Software`. The user-facing 3Dfx renderer is
   saved as `Glide`. Direct3D saves the enumerated device name, formed
   from `D3D-` followed by its driver name; `D3D-display` is an observed
   installed value. These names are not interchangeable with the menu's
   localized labels.
4. **Availability comes from enumeration, traced.** A row needs a device
   of its renderer family with fullscreen availability. DirectDraw is
   additionally disabled while the current resolution is 800 × 600.
   The Direct3D row excludes the device named `D3D-dd3dfx` and is also
   suppressed when a native Glide-family device occurs in the enumerated
   list. The latter check is against presence in that list, not a second
   fullscreen-availability check. If several eligible Direct3D entries
   survive, the row retains the last eligible entry encountered.
5. **Resolution availability is restricted, traced.** The 640 × 480 row
   is enabled. The 800 × 600 row starts disabled and is enabled only by
   the active device's 800-mode capability. In this executable,
   Software/DirectDraw and Direct3D both report that capability false.
   Glide reports it from its hardware query: the first reported board
   must have type 0 or 1 and at least 4 MB of framebuffer memory. The
   numeric board types correspond to Voodoo and SST96 in the local Glide
   interface declaration. This describes the original capability rule;
   the installed wrapper's current reported hardware was not queried.

### Applying a selection

6. **A resolution click applies immediately, traced.** Selecting the
   other enabled resolution requests the new window and rendering
   dimensions from inside the Graphics screen, updates the 800-mode
   setting, refreshes the interface dimensions, and redraws that screen.
   It does not wait for Back to Options, returning to gameplay, or a
   restart. Clicking the already-selected resolution makes no resize
   request. No confirmation dialog is used for this resolution change.
7. **Most device changes also apply immediately, traced.** An eligible
   device choice normally requests the selected fullscreen device, then
   refreshes the current palette and Graphics screen. The path refreshes
   presentation resources; no new-world load is present in that path.
   Exact camera, animation and panel-state preservation across real
   driver changes is **not tested**.
8. **Direct3D to DirectDraw is a distinct family, traced/data.** When the
   current renderer is Direct3D and the menu has a corresponding
   Direct3D device entry, selecting DirectDraw instead opens the
   `Switch to Direct Draw?` confirmation. The other Graphics controls
   are disabled during that modal choice. Acceptance stores the
   Software device name, writes Options, and exits the application;
   the choice takes effect on the next launch. Declining restores the
   Graphics screen without switching. Do not apply the ordinary live
   device-change rule to this transition.
9. **Persistence records names and a flag, traced.** `Options.cs` stores
   `assignGModeName` with the chosen device name and `set800mode` with
   the current resolution flag, alongside the other graphics options.
   Assigning a mode name through its console command stores the name;
   that command alone is not a live device switch. Similarly,
   `set800mode` participates in startup setup and interface-art selection;
   it is not equivalent to the Graphics screen's live resize action.
10. **Startup validates Glide separately, traced.** With an enumerated
    device list, a saved `Glide` selection is retained only if an
    available fullscreen Glide device is found. Otherwise it falls back
    to `Software` and clears the pending 800-mode request. A saved name
    other than `Glide` also clears the pending 800-mode request. This
    validation does not prove a general fallback for arbitrary invalid
    Direct3D names, an empty device list, or unavailable resolutions on
    an otherwise present Glide device.

### Shared interface behavior

11. **The interface uses the actual presentation dimensions, traced.**
    A successful resolution update gives the GUI the rendering device's
    actual width and height, updates its clipping bounds to those pixel
    dimensions, and sends a resize notification through the existing
    screens and their controls. Graphics and gameplay screens use the
    same general resize mechanism. The menu is not always a 640 × 480
    bitmap stretched as a finished frame.
12. **General rectangles scale, traced.** A control's stored normalized
    left/right values are multiplied by its containing width; its
    top/bottom values are multiplied by the containing height. Each
    resulting coordinate receives an addition of 0.1 and is then
    truncated toward zero. Child controls use the resized container's
    width and height. Thus ordinary 640-layout coordinates become
    approximately 1.25 times as large at 800 × 600, with this stated
    conversion rule rather than rounding every value to the nearest
    integer. Retain the original normalized values when changing back;
    do not repeatedly scale already-rounded pixel rectangles.
    **Parent and child conversion, clarified 2026-11-07:** when a
    control is bound to its GUI data, each integer edge is normalized
    against its own parent's width or height, not a fixed 640 or 480.
    A resize converts the parent's edges first and obtains the parent's
    new integer extent by subtracting those converted edges. Children
    use that integer extent for their separate local conversions.
    During drawing, each dialog adds its converted integer origin to
    its ancestors' integer origins; children add their converted local
    rectangles to that sum. There is no final scaling or rounding of a
    summed original absolute coordinate. For example, a parent originally
    at x=1 with width 14 becomes x=1 through x=18, width 17, at 800.
    Its child at local x=1 becomes local 1 and absolute 2. A child at
    local x=3 becomes local 3 and absolute 4; scaling the original summed
    absolute x=4 directly would incorrectly give 5.
13. **Fonts keep their pixel size, traced.** The GUI resolution update
    resets the text scale to 1 horizontally and vertically, and the
    shared text control uses that scale for the font. Rectangles and
    text alignment change with the larger boxes; the font glyphs do
    not become 1.25 times larger merely because the resolution changed.
    The disabled-row checker pattern is applied to the rendered text.
    **PFT scale identity, clarified 2026-11-07:** the shared GUI text
    drawing path replaces the font's own horizontal and vertical scale
    values loaded from the PFT font-info record with 1.0. This is not an
    extra GUI multiplier applied on top of the intrinsic PFT scales.
    Glyph dimensions, baseline information, and character spacing remain
    supplied by the font; the two scale values are what this path
    overrides. This finding is scoped to shared GUI text drawing, not
    every independent font user elsewhere in the engine.
14. **Caption width follows its box, traced.** The gameplay caption
    dialog receives the general resize behavior. Its half-screen width
    is 320 pixels at 640 × 480 and 400 at 800 × 600. The existing
    `interface.md` wrapping rule therefore operates with those actual
    widths and the same unscaled `console.pft` glyph widths. Row height
    remains 15 pixels, with the previously documented 4-pixel strip
    padding. Existing panel-relative caption positioning still applies.
    This resolves the earlier inference about 400-pixel wrapping width;
    it does not change the previously documented greedy wrapping rule.
15. **Picture behavior is independent of font behavior, traced.** The
    general picture rule in `interface.md` still applies: a stretched
    picture follows its resulting box, an unstretched picture retains
    its own pixel size, and the caption's stone background is tiled.
    Increasing the display dimensions is not texture resampling or a
    change to the game world's texture files.
16. **Save/Load is an explicit layout exception, traced.** On activation,
    the Save and Load screens reposition their main control groups and
    ten slot controls into a centered 640 × 480 area. At 800 × 600,
    those controls retain their 640-layout size and move by 80 pixels
    right and 60 pixels down. Their specialized scroll/list placement
    also accounts for the 60-pixel vertical offset. This does not mean
    every screen or every background uses that rule.
    The exact controls explicitly centered on activation are the
    `IDCTL_SAVE_` or `IDCTL_LOAD_` controls ending in **NAME, OK, DELETE,
    CANCEL, UP, TAB, DOWN, HILITE, LIST0 through LIST9, and THUMBNAIL**.
    Save also centers **BAR** and **RED**. This list does **not** include
    either screen's TITLE or its POPUP_BKGND, POPUP_TEXT or POPUP_BTN
    controls. Do not translate every non-root control by 80/60: controls
    outside the stated list retain shared GUI handling and any separate
    popup-placement rules. The absence of a control from this list is
    not a new claim about every later popup-specific repositioning.
17. **Resolution-specific background art exists, traced/data.** The
    original selects 640 and 800 versions of the main-menu and Save/Load
    backgrounds. The loose files use the names `main18.pbm` and
    `saveload.pbm` for the selected versions, with `main4x6.pbm`,
    `main6x8.pbm`, `save4x6.pbm` and `save6x8.pbm` as alternate names.
    The original swaps those names and invalidates the loaded resource.
    The current read-only installation has 640-sized selected files and
    800-sized `main6x8.pbm`/`save6x8.pbm` alternatives. A recreation can
    select the corresponding source resource without renaming anything.
18. **GUI hit tests use the resulting pixel rectangles, traced.** For
    example, the Graphics row hit test compares the supplied pointer's
    vertical pixel coordinate with the row's current top and bottom;
    the boundaries are included. It then rejects disabled rows. Tests
    must use the resized or centered rectangles, not an independent
    unscaled 640-layout copy. The complete Windows-pointer-to-GUI and
    3D-picking conversion chain remains untraced here.

## Failure behavior and limits

The lower-level device switch rejects an unknown device name or a device
it cannot create. If an attempted active-device replacement fails to
initialize, it releases the failed replacement and attempts to re-enable
the preceding device. The menu-level path does not check all of those
returns before retaining the requested name or redrawing. Consequently,
this study does not establish that the original always leaves a consistent
saved preference after failure. Original driver failure handling is
**not tested**.

The resolution backend may select a supported size from its enumerated
sizes before creating the new surface. No general unsupported-resolution
fallback to exactly 640 × 480, timed rollback, or keep-this-mode dialog
has been established. Do not invent such a rule as original behavior.

## Data used

- `GAME/Kq/Graphics.kq`: `KQGraphicsScreen`, `Graphics.gui`, `Menus.ppl`,
  background color 0, `useWorldPal=0`, `IDBMA_CURSGUI`, `click.aud`,
  and selection sound `intrface.wav`.
- `GAME/Resource/Graphics.gui` and `kqGuiTag.TTAG`: fixed controls and
  localized caption tags, including the DirectDraw-switch modal.
- `GAME/Kq/Main.kq`: base window 640 × 480; initial high-resolution
  DirectDraw mode configuration.
- Installed `Options.cs`: currently `assignGModeName D3D-display` and
  `set800mode 0`. These are installed settings, not claimed factory
  defaults; older notes describe a different installed value.
- `GAME/8Gui` background alternatives and the gameplay caption layout;
  existing `interface.md` describes art, fonts and caption formatting.
- Local `directglide-kq8-main/src/glide2x.h` was consulted only to name
  the hardware-query board types and framebuffer-memory unit, not to
  infer the original game's behavior from that patch.

## Do not reproduce

Do not rename files in the reference installation to select interface
art. Do not copy the original's unchecked failures into a new renderer;
report a failed switch and retain a usable mode. Such defensive failure
handling should be recorded as a compatibility safeguard, not evidence
of the original's behavior. Do not claim a modern graphics API is an
original renderer merely by displaying one of these menu labels.

## How to check with representative families

These are proposed automated checks, **not tested in this study**. No
manual play or visual comparison is requested.

- A device-list fixture can exercise missing Glide, present Glide,
  `D3D-dd3dfx`, an ordinary Direct3D device, and multiple eligible
  Direct3D entries. Compare row eligibility, stored name and selection
  markers. A separate Glide-capability fixture checks the board/memory
  threshold; DirectDraw and Direct3D do not enable 800.
- One ordinary control rectangle and one nested control can establish
  the shared coordinate conversion at both resolutions and after a
  return to 640. For a positive normalized coordinate corresponding to
  102 layout pixels, the 800 coordinate is 127, not 128. A separate
  parent at x=1 with width 14 and child local x=3 establishes absolute
  x=4 at 800, which distinguishes nested conversion from global scaling.
  A separate Save/Load fixture establishes the centered-size exception.
- A representative text label, stretched picture, unstretched picture
  and caption establish the distinct rendering families. Check fixed
  glyph sizes, replacement of non-unit intrinsic PFT scales with 1.0
  for shared GUI text, picture-box dimensions and caption widths 320/400.
- One live-resolution transition checks immediate dimension updates and
  current-screen refresh. A separate Direct3D-to-DirectDraw transition
  checks the confirmation, decline and accepted restart paths.
- Pointer samples on and just outside a resized row establish inclusive
  row boundaries and disabled-row rejection. Native Windows input and
  3D picking need their own remaining conversion evidence.

## Open questions

- Exact 3D viewport, projection/FOV and world-picking changes during a
  resolution switch; the GUI dimensions do not establish these rules.
- Complete native pointer conversion, cursor hotspot treatment, and
  possible differences between windowed and fullscreen input.
- Original startup behavior with an empty device list, an unsupported
  saved Direct3D name, or a saved 800 request on a Glide device that does
  not advertise 800; real failed switches and recovery are not tested.
- Exact gameplay/camera/animation state survival across a successful
  backend replacement. No new-world load was found in that path, but
  a complete state-preservation audit has not been performed.

Q207 is therefore partially answered. Q108's general rectangle and font
scaling, caption width, and Save/Load layout exception are established;
this is not a claim that all original presentation behavior is complete.
