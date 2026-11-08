# Brightness, palette upload and display gamma (Q186)

Study session A, 2026-11-06. Sources: the installed GOG 1.03
`Mask.exe`, its `Options.cs`, and the bundled nGlide 1.04 `glide2x.dll`
and readme, all read only. This is static executable/data study in our
own words. Original runtime output, graphics-driver state and native
reload were **not tested**. This note corrects `object-lighting.md`
rules 17 and 32. It answers the engine-side palette, update and matching
questions; Q186 remains partly open for final output and artwork paths.

## Purpose

Describe the original brightness setting without treating its palette
gain as an additional display-gamma exponent. The chosen renderer affects
the result, even when the same palette and slider value are used.

## Rules

### Preference and live changes

1. **The preference is an integer.** `setBrightness <value>` reads the
   value as an integer and stores it. That command alone does **not**
   rebuild the active palette or call a display-gamma API. Wrong argument
   count reports usage. The command has no 0–100 clamp; the useful
   authored/menu range and the examples here are 0–100. Behavior for
   deliberately extreme or malformed values is not established.
2. **The palette gain is nominally `0.3 + 1.4 × value / 100`.** More
   precisely, the endpoints are the promoted single-precision values
   **0.30000001192092896** and **1.7000000476837158**. The interpolation
   uses a divisor of 100 divided by their difference, then rounds the
   result to a single-precision gain. Values 0, 50 and 100 give the two
   endpoints and **1.0** respectively. This gain multiplies stored RGB
   values; it is not a power applied to normalized RGB.
3. **Moving the original brightness control applies immediately.** Its
   change notification stores the preference, computes the gain and,
   when an active window surface exists and the gain differs, rebuilds
   that surface's palette. It also requests the surface's drawing-cache
   flush. No Graphics close, new screen or restart is needed for this
   palette update. With no surface, the preference is still retained.
4. **The preference is also applied at later setup points.** Options
   setup, screen/display restoration and restoration after a film apply
   it to the active surface. A new surface begins with gain 1.0 before
   the preference is applied. A gain change starts from source palette
   RGB; ordinary repeated changes do not multiply the previous adjusted
   bytes and grow progressively brighter or darker.
5. **The supplied installation is not evidence for a universal default
   of 50.** Its current `Options.cs` says `setBrightness 49` and
   `assignGModeName D3D-display`. The preference can be saved by the
   original. Use 50 as the identity-gain example, without asserting that
   every installation is configured to it.

5a. **Mode names, bounded Q186 clarification.** The exact enumeration
    names are **`Glide`** and **`Software`**. Direct3D names are
    **`D3D-` followed by the enumerated driver name**; the installed
    preference's **`D3D-display`** is also a literal used by the engine.
    It additionally contains the device-specific name `D3D-dd3dfx`.
    Thus the concrete preference commands are `assignGModeName Glide`,
    `assignGModeName D3D-display` and `assignGModeName Software`, without
    claiming those are all possible Direct3D device names or that every
    mode is available on this machine. The assignment preserves its
    supplied name; its software/hardware distinction tests the exact,
    case-sensitive name `Software`. This check does not establish any
    alternate spelling or alias.

### Glide and Direct3D palette colours

6. **Both hardware palette paths scale all 256 entries.** For ordinary
   0–100 values, multiply the three RGB bytes by the gain. If the largest
   scaled channel would exceed 255, use a shared reduced gain of
   **255 divided by the largest original channel**. This preserves the
   colour's proportions instead of clipping individual channels. The
   reduced gain itself is rounded to single precision. There is no
   brightness exemption for the first or last palette entries.
7. **Byte conversion cuts toward zero**, rather than rounding to the
   nearest integer. Values above 255 are explicitly capped to 255.
   Palette metadata/flags are not RGB channels and are not gain-scaled.
   Some channel products are rounded to single precision before integer
   conversion, and this differs between the two hardware paths. Near
   exact integer boundaries, the phrase “multiply and truncate” alone
   is therefore insufficient to promise identical bytes for every RGB
   triple. Exhaustive byte tables under each renderer's actual floating
   point environment remain open; no such table was captured here.
7a. **Explicit float storage, bounded Q186 clarification.** Both
    hardware paths receive a stored **32-bit float gain**. Their first
    scaled-RGB probe, used to find the largest channel and decide
    whether to reduce the gain, keeps its products in **x87 registers**;
    it does not explicitly store those products as 32-bit floats.
    The selected per-entry gain, including a reduced **255 / largest
    source channel**, is stored as a 32-bit float before final RGB
    conversion. Source-byte copies in that format are exact integers.

    | Final product before integer conversion | Glide | Direct3D |
    | --- | --- | --- |
    | Red | retained x87 register value | stored/reloaded as 32-bit float |
    | Green | stored/reloaded as 32-bit float | retained x87 register value |
    | Blue | retained x87 register value | stored/reloaded as 32-bit float |

    Each final **above-255 comparison precedes** the explicit product
    store in this table. Neither palette loop establishes its own x87
    precision/rounding control state. A stored 32-bit float is an
    explicit format boundary, not evidence that its rounding mode is
    always round-to-nearest. Conversely, a retained register product
    is not necessarily full extended precision: the incoming x87
    precision setting can make its arithmetic round sooner. Do not
    force every channel product to 32 bits merely because the gain is
    32 bits. The incoming control state and later driver/texture-format
    quantization remain unresolved here; no further floating-point or
    driver audit was attempted in this bounded check. Original runtime
    byte tables are **not tested**.
8. **Glide has a further converted-black exception.** After brightness
   conversion, a packed RGB value of zero becomes **(255, 0, 255)** for
   entries **1–9 and 246–254** only. This is 18 entries. Entries **0 and
   255 are outside this exception**, as are entries 10–245. It is the
   converted result that is tested: a sufficiently dark nonblack colour
   in one of those 18 entries can become magenta. This is separate from
   earlier palette loading/system-colour behavior in
   `models-and-textures.md`; it does not by itself explain that note's
   observed entry-0 change.
9. **Source palettes normally stay in stored RGB space.** Glide builds
   a separate adjusted upload table. Direct3D temporarily adjusts the
   source entries during its palette update, then restores their original
   bytes when that update succeeds. Subsequent normal palette matching
   therefore uses the source values. An exceptional Direct3D palette
   creation failure returns before that restoration; that failure path
   can leave adjusted source bytes. Driver-failure recovery was not run.
10. **Fog/background are not uniformly gain-scaled.** Glide's palette
    fog colour and indexed background clear use the **source** RGB entry,
    not the adjusted upload table. Direct3D selects its fog colour while
    the temporary adjusted entries are present, so its fog uses adjusted
    RGB. Direct3D's indexed background clear reads the restored source
    RGB to make its background material. Thus both hardware paths use
    source RGB for this background operation, but their palette-based
    fog differs away from gain 1.0. Final card/wrapper rounding is a
    separate stage.

### Software palette colours

11. **The software output palette clips channels independently.** Its
    8-bit DirectDraw and window-DIB paths multiply each RGB channel by
    the gain, cap it at 255 and convert toward zero. They do not reduce
    all three channels together to preserve hue. These palette-update
    paths do no corresponding conversion for a non-8-bit surface; this
    is not a claim about every possible software drawing destination.
12. **The window-DIB path also gets Windows' system colours.** Before
    constructing its adjusted 8-bit colour table it replaces source
    entries **0–9 and 246–255** with the current Windows system-palette
    entries. These 20 entries then go through the same brightness
    multiplication as the others. It has no Glide magenta substitution.
    The resulting edge colours depend on Windows' palette, and are not
    established solely by reading a `.ppl` file.

12a. **Which Software surfaces the game selects, traced 2026-11-08.**
     The ordinary Software device explicitly selects **8 bits per
     pixel** in both of its output paths:

     | Software presentation | Original requested surface | Palette behavior |
     | --- | --- | --- |
     | Fullscreen DirectDraw | 8-bit indexed display mode and DirectDraw drawing surfaces | Rule 11's independent channel clipping |
     | Window output | 8-bit indexed Windows DIB, including the retry at the previous size | Rules 11–12, including the Windows system-color replacement |

     The Software fullscreen mode list also admits only 8-bit display
     modes. Changing its size or rebuilding its surfaces does not select
     16, 24 or 32 bits per pixel. The normal Windows DIB creation stores
     the requested 8-bit depth explicitly; it does not substitute the
     desktop's pixel depth. A modern 32-bit desktop therefore does not
     establish a 32-bit game drawing surface or bypass rule 12.

     The shared DIB constructor contains support for a caller-supplied
     non-8-bit format, but the studied Software creation and retry paths
     supply 8. No ordinary startup, Graphics selection or Software live
     resize path studied here selects a 16-, 24- or 32-bit Software
     surface. This is a bounded call-path result, not a claim that every
     unrelated bitmap, film or hardware-renderer destination is indexed.
     An unsupported 8-bit display request does not establish a fallback
     to one of those higher depths.

     These are original surface choices, independent of the host API
     used to present a recreation. A modern backend's texture or
     swapchain depth does not identify which original palette rule to
     apply. Device selection and the separate fullscreen/window slots
     are described in `graphics-modes.md`. Original runtime and actual
     Windows/driver conversion to the desktop: **not tested**.

### Display gamma and other drawing paths

13. **Correction: the brightness gain is not sent to Glide display
    gamma.** The engine's sole call to `grGammaCorrectionValue` is during
    construction of a fresh Glide display surface, whose gain is still
    **1.0**. The later preference application and live slider updates
    rebuild its palette and do not call that API. Consequently slider
    values 0/50/100 do **not** request display-gamma values 0.3/1.0/1.7.
    The Direct3D/software palette updates studied here likewise do not
    set a display gamma ramp. The separately named console `setGamma`
    changes a canvas's palette gain through the same palette-update
    mechanism; its name is not evidence for an additional display ramp.
14. **The GOG Glide wrapper is another independent setting.** The
    bundled nGlide 1.04 combines its configured gamma with the incoming
    API gamma as **configured gamma + API value − 1**, with negative combined values
    falling back to the configured value and an upper limit of 20, and submits one equal red/green/blue
    256-entry, 16-bit gamma ramp to Direct3D. The nominal ramp is
    **65535 × (entry / 255) raised to (1 / combined gamma)**, converted
    toward zero; its arithmetic includes single-precision constants and
    exponent. With the engine's API value 1.0, the wrapper's configured
    gamma remains the relevant gamma setting, independently of the
    brightness slider. This does not establish the user's effective
    wrapper configuration, exact driver-accepted ramp, monitor transfer,
    or physical screen RGB. Those were not measured.
15. **Palette-based menus share the active surface rule.** The main
    menu's `Menus.ppl` replaces the world palette (`main-menu.md`), and
    the active surface's palette upload still uses its brightness gain.
    The update is not restricted to terrain or model textures. This does
    not prove equivalent brightness for every widget, true-colour BMP,
    texture format or film frame.
16. **Films have a separate output path.** `playAvi` creates a film
    playback window, and non-Direct3D game displays are temporarily
    switched to Software and restored afterward. Direct3D follows its
    own film-window/display handling. The palette-gain preference is
    reapplied to the game surface on return. Codec output, any film
    colour conversion, and final brightness during playback remain open;
    no all-screen postprocess can be inferred from the palette rules.

### Matching indices before display adjustment

17. **`resetHazeColor` matches the active source palette**, not its
    adjusted upload table. It searches usable entries **10–244** for the
    least squared RGB difference; an equal difference retains the earlier
    entry. It stores the selected index and reapplies the palette. An
    ordinary brightness change does not change that match's input RGB.
18. **Other-world magic maps match stored file RGB.** The matching path
    reads 256 RGB-plus-flags entries from the selected original `.ppl`
    files, beginning at file byte 28. With the normal full search range,
    each source entry picks the least squared RGB difference among all
    256 destination entries; ties keep the earlier destination. The
    resulting map indices are displayed through the current palette
    afterward (`magic-map.md` rule 34). Brightness is not applied before
    computing that map-index translation. This file-based matching is
    also independent of the window-DIB system-colour replacement.

## Numbers and units

- Slider/preference: integer; ordinary range 0–100. Gain: unitless,
  stored as single precision. RGB palette channels: unsigned bytes.
- Display API gamma at Glide surface creation: **1.0**, at every
  brightness preference. It is a separate quantity from palette gain.
- Suitable non-boundary numerical checks for a palette entry outside
  Glide's special 18 entries:

  | Source RGB | Value 0, both hardware/software | Value 50 | Value 100, hardware | Value 100, software |
  | --- | --- | --- | --- | --- |
  | (102, 153, 204) | (30, 45, 61) | (102, 153, 204) | (127, 191, 255) | (173, 255, 255) |
  | (3, 7, 11) | (0, 2, 3) | (3, 7, 11) | (5, 11, 18) | (5, 11, 18) |

These are values derived from the traced conversions, not captured
original-framebuffer samples or final monitor RGB.

## Data used

The installation's `Options.cs`; the world's selected `.ppl` palettes;
`GAME/Resource/*.ppl` for other-world map matching; `Menus.ppl` and
`Mainmenu.kq` for menu palette selection; `glide2x.dll` and
`nglide_readme.txt` for the bundled wrapper. No original files were
changed. No original/extracted implementation material accompanies this
note.

## Edge cases and do not reproduce

Preserve the traced renderer differences and Glide special-entry rule
when claiming parity for those modes. Do not add an extra
brightness-driven display-gamma curve based on the former rule 17.
Do not apply brightness to palette-matching inputs. Do not infer a
universal 50 default from the identity-gain example. Malformed preference
values and Direct3D device failures need a separate policy; normal output
claims here do not cover them.

## How to check

An automated original-engine capture can log the preference, active gain,
source palette RGB, uploaded table, fog/background RGB and gamma API
arguments at 0/50/100, then during a 0 → 100 → 0 slider sweep without
closing Graphics. Expect immediate palette changes, stable source RGB on
successful hardware updates and a gamma API argument of 1.0 only when a
new Glide surface opens. Record the floating point control state and
driver format alongside each upload before claiming exact byte tables.
Check magenta substitution at indices 0, 1, 9, 10, 245, 246, 254 and 255.
Log haze/map selected indices across brightness changes; normal matches
must remain unchanged. Repeat on menu/world palette switches and before,
during and after a film. These original runtime checks are **not tested**
in this study; manual play testing was not requested.

## Open questions

- Exact final output/texture-channel quantization in each graphics mode,
  including renderer/driver floating point state at integer boundaries.
- Effective nGlide/driver gamma-ramp bytes and physical display transfer;
  the slider does not select the three display gamma factors assumed in
  the original question.
- Complete 24-bit artwork, all menu drawing primitives, movie codec and
  film-window output equivalence. Palette behavior alone cannot settle
  these paths.
- Original runtime capture and failure/recovery behavior. No original
  render or native reload was executed during this study.
