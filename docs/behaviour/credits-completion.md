# Credits and completion score

Study session A, 2026-11-07. This answers the shared presentation and
score rules in Q198. Sources are the read-only GOG original executable
(static tracing), `GAME/Kq/credits.KQ`, `GAME/Resource/credits.gui`,
`GAME/Resource/kqGuiTag.TTAG`, and `GAME/ENGLISH/400.MSG` (data). Executable
evidence stays in ignored local study files. No Rust was read or written
for this study. Original runtime behavior is **not tested**.

## Purpose

Credits are a sequence of stationary text cards ending with a team
photograph and Connor's score. The menu's Credits choice and the ending's
`doCredits` command use the same viewer, assets, timing, and score rules.

## Rules

### Entry, ordering, time, and input

1. **Every entry starts again at card 1, traced.** Showing Credits resets
   its sequence and immediately presents the first card. Cards are
   message cases **1 through 25**, in numeric order, from message file
   400. Use the case identifier, not physical record order: the installed
   file stores case 25 before cases 20–24.
2. **Cards advance automatically using real time, traced.** An ordinary
   card lasts **2 seconds plus 0.5 seconds for each displayed line**.
   Both font styles and intentionally blank spacer lines count. The
   screen checks the deadline during its regular updates. This is elapsed
   Windows time, independent of the paused gameplay clock and of frame
   count.
3. **The final card has 12 extra seconds, traced.** Its duration is
   therefore **14 seconds plus 0.5 seconds per line**. The installed final
   card has ten lines, including eight spacer lines, and lasts **19
   seconds**. The photograph appears with this card and remains visible
   for that interval. It is not a separate timed stage after the text.
4. **A late update advances one card, traced.** It does not race through
   several overdue cards. Each new deadline starts from the current real
   time when that card is selected. The installed 25 cards contain 132
   lines in total, so their nominal uninterrupted duration is **128
   seconds**. Update lateness can extend it.
5. **Escape exits to the main menu, traced.** An ordinary Escape press
   exits from any card, including the final photograph. It does not quit
   the application, resume gameplay directly, or return to the Options
   submenu. Expiry of card 25 also returns to the main menu. Both routes
   stop the retained menu-theme sound with its normal one-second fade
   before showing the main menu; subsequent menu setup follows
   `main-menu.md`.
6. **There is no user-driven next-card command, traced.** Enter, Space,
   arrow keys, and mouse clicks do not shorten a card's interval. Keyboard
   events do check whether the existing deadline has expired, so a key
   arriving at the deadline can coincide with the normal automatic
   advance. This must not become unconditional key-to-skip behavior.
   The credits-specific pointer handler has no selection action.
7. **`doCredits` accepts no arguments, traced.** It selects this same
   screen. It does not award points, take a score argument, select a
   different card list, or calculate a separate ending score on entry.
   The game's movie and script sequencing that leads to this command is
   outside this viewer's timing.

### Shared card layout

8. **Authored line breaks define the rows, traced and data.** The viewer
   separates each message at its line breaks. It does not redistribute a
   card into paragraphs according to available width. Leading and
   embedded spaces are retained. In particular, the final card's eight
   lines containing a space are intentional layout and timing input.
   A terminal line break does **not** add an extra empty row: parsing
   ends when the next segment starts at the string terminator. Interior
   empty rows and authored space-only rows still count.
   **Study correction, 2026-11-07:** the earlier 134-line/129-second
   total mistakenly counted empty segments after the final CRLF in
   cases 19 and 20. Rechecking the original parser against the installed
   message data gives 8 rows/6 seconds for case 19 and 12 rows/8 seconds
   for case 20, and the corrected 132-row/128-second total above. This
   corrects a study counting error; original runtime remains not tested.
9. **A leading asterisk selects the smaller style, traced and data.** A
   line beginning with `*` uses `20sl.pft`; the leading asterisk is
   removed. A line without it uses `27sl.pft`. Both are horizontally and
   vertically centered within their line rectangles. Preserve the
   original interleaving of styles. Do not collect all the small-font
   headings into one group above the larger names.
10. **All rows share one vertical spacing rule, traced.** For a card
    containing N lines, the top-to-top gap is the smaller of 36 pixels
    and the integer part of **(screen height − 50) / N**. The first row's
    top is half the screen height minus the integer part of **N × gap /
    2**. Subsequent row tops add one gap each. Every line rectangle is
    **30 pixels high**, regardless of font style, and spans the GUI's
    full width. Text glyphs follow the shared unscaled GUI font rule in
    `graphics-modes.md`; they do not grow by 25% at 800 × 600.
11. **The spacing is recalculated in output pixels, traced.** It is not
    the entire 640 × 480 text layout stretched to 800 × 600. Representative
    installed card families illustrate the rule:

    | Family | Lines | Duration | Gap / first top at 640 × 480 | Gap / first top at 800 × 600 |
    | --- | ---: | ---: | ---: | ---: |
    | One small heading and one large name | 2 | 3 s | 36 / 204 px | 36 / 264 px |
    | Dense mixed-font card (case 20) | 12 | 8 s | 35 / 30 px | 36 / 84 px |
    | Densest installed card | 16 | 10 s | 26 / 32 px | 34 / 28 px |
    | Photograph and score, including spacers | 10 | 19 s | 36 / 60 px | 36 / 120 px |

12. **The photograph is only on card 25, traced and data.** Its asset is
    `kqteam08.pbm`, identified by `IDBMP_KQTEAM08` in the tag data. The
    original GUI rectangle is (120,10)–(520,310): 400 × 300 pixels,
    horizontally centered. It uses stretched picture drawing and full
    opacity. The general GUI resize scales its width and height, but
    Credits explicitly places its top at **10 output pixels**. This
    gives (150,10)–(650,385), or 500 × 375 pixels, at 800 × 600.
13. **The final caption and score are ordinary rows below the spacers,
    traced and data.** The team caption is the ninth row and uses the
    larger font; the score is the tenth and uses the smaller font. Their
    row tops are 348 and 384 pixels at 640 × 480, or 408 and 444 pixels
    at 800 × 600. No additional space is automatically reserved by
    measuring the photograph: the eight authored spacer lines supply it.
14. **Every card replaces the previous text, traced.** Unused text rows
    are cleared and placed outside the visible area. The photo is hidden
    when Credits opens, so reopening Credits cannot show the preceding
    run's final photograph behind card 1.

### Score and presentation state

15. **Score uses Connor's level and current experience, traced.** Read
    them when preparing card 25, not when first entering Credits.
    Current experience means the experience still held toward the next
    level, after past level-up deductions; see `interface.md` rules
    27–30a. Let L be Connor's current level and E that remaining
    experience:
    - At levels 1–15, the displayed number is **E + 10 × (2^(L−1) − 1)**.
    - Above level 15, it is **E + 163,830 + 150,000 × (L−15)**.
    These expressions reconstruct the experience associated with completed
    levels and add the remaining amount. No quest-completion percentage,
    health, gold, inventory count, elapsed play time, or difficulty bonus
    is included by the credits calculation.
16. **A fresh title session shows zero, traced; eligibility correction
    2026-11-07.** The score tests whether a game has been started, not
    whether the player may currently return to gameplay. Before a game
    has started, the final card substitutes zero. Once one has started,
    it reads the retained Connor's level and current experience.
    Opening Credits through menus does not force zero. Back to Game
    requires both a started game and the absence of the separate
    unavailable/dead condition. The earlier version of this rule
    incorrectly equated these tests; that was a study error. In
    particular, `killGame` disables Back to Game without making the
    credits score zero. See the ending lifecycle below.
17. **The authored denominator is fixed, data and traced.** The English
    score message contains the fixed possible total **6,631,364** and a
    single signed decimal integer substitution for the player's score.
    The displayed digits have no grouping commas, padding, fractional
    digits, or percent sign. The denominator is not recomputed from
    current level or available content, and the credits calculation does
    not clamp the player's number to it.
18. **Score is not necessarily a separately accumulated lifetime total,
    traced.** Script changes such as `KQConner::setLevel` can change the
    level without changing current experience. Credits follows the
    resulting level and experience even when they no longer agree with
    an independent total-earned-experience counter. A recreation must
    preserve that distinction.
19. **Use the credits palette setup, traced and data.** `credits.KQ`
    specifies background color 0, no world palette, and `Menus.ppl`.
    The credits activation then attempts to replace the palette with
    **`saveload.ppl`**; it does so when that palette is available. Use
    this final active palette for the original credits art. On leaving,
    the general GUI text catalogue is restored to message file 500 and
    the next screen applies its own presentation rules.

## Ending lifecycle: killGame, movie and credits

20. **Installed ending sequence, data.** The original
    `Temple4/kq.vol:vortrap.cs`, on the completion callback for
    `c_imdone.kqs`, executes `killGame`, `playAvi w32ndg_1.dll`, then
    `doCredits`. This installation has no loose override for that
    script. The original command spelling is `killGame`; an internal
    recreation request named EndGame must still implement these
    original effects.
21. **killGame makes play unavailable; it does not destroy the game,
    traced.** The command immediately marks the session unavailable
    for returning to gameplay, selects the main menu and pauses world
    simulation. It retains the world, Connor and the fact that a game
    has started. In particular it does not reset Connor's experience
    or level, destroy him, replace him with a fresh Connor, or take a
    zero-score snapshot. Back to Game is therefore unavailable from
    this point even though credits can still read Connor. The same
    unavailable condition also prevents the ordinary Back to Game
    action from resuming the completed session.
22. **Movie return preserves that retained game, traced.** The movie
    command is synchronous with respect to the calling script: after
    playback returns, execution proceeds to `doCredits`. Its playback
    setup and cleanup manage the movie window, display and sound
    resources; they do not remove Connor or clear the game-started
    condition. A missing movie file returns from the movie command,
    allowing the following `doCredits` to execute. Exact movie input
    and audio synchronization remain outside this bounded study.
    **Existing-file failure clarification, 2026-11-07:** the missing-file
    result must not be generalized to decoder or media-open failures.
    A readable file reaches the Windows MCI movie service with its
    standard error dialogs enabled. Thus an MCI-reported codec or
    media-open error can present an acknowledgement before any return
    to the script. This is a Windows media error, not a game-authored
    Credits or Main-menu popup. The executable's choice is traced;
    the dialog consequence follows the documented
    [MCI error behavior](https://learn.microsoft.com/en-us/windows/win32/multimedia/error-messages-and-notifications)
    and [movie-window creation contract](https://learn.microsoft.com/en-us/windows/win32/api/vfw/nf-vfw-mciwndcreatea).
    There is no single reliable original continuation rule for every
    setup failure: failure of the preliminary display-device switch
    returns before opening media, whereas failed movie-window creation
    is not routed through an equivalent early return. The subsequent
    wait expects the movie to report that it has stopped. Consequently,
    neither an immediate silent return nor a guaranteed return after
    acknowledging every decoder/setup error is established. These
    error paths are static findings, **not tested** in the original.
23. **Credits still reads the ending Connor, traced.** `doCredits`
    selects the same credits viewer used by the menu. It does not
    undo the completed-session restriction and does not dispose of
    Connor. When card 25 is constructed, use the retained level and
    current experience under rules 15–18. A successful ending must
    not acquire score zero merely because ordinary gameplay has ended.
    This is a late read at card construction, not a total copied when
    `killGame` or the movie began.
24. **Leaving ending credits, traced.** Escape or natural completion
    returns to Main under the normal credits exit rule. That screen
    change does not make the completed game playable again and does
    not discard the retained Connor. Reopening Credits in this same
    completed session still has access to his score. Destruction or
    replacement during a later new game, load, quit or application
    shutdown is a separate lifecycle operation, not an effect of
    `killGame`, movie completion or credits exit.
    **New-menu clarification, 2026-11-07:** New's already-running warning
    requires a started game whose gameplay remains available. An ended
    session therefore skips that warning even while Credits retains
    access to its score. The death menu uses the same warning predicate;
    see `main-menu.md` rule 10.

The lifecycle findings above are static evidence, **not tested** by
running the original ending.

## Data used

- `GAME/Kq/credits.KQ`: screen class, GUI, volume path, initial palette,
  background, cursor, and generic menu sounds.
- `GAME/Resource/credits.gui`: root 640 × 480 canvas, photograph control,
  and two text-control styles. It provides 13 larger-font rows and 16
  smaller-font rows; every installed card fits these capacities.
- `GAME/Resource/kqGuiTag.TTAG`: font and photograph asset-name mappings.
- `GAME/ENGLISH/400.MSG`: card cases, line breaks, style markers, final
  score format, and fixed possible total.
- `Temple4/kq.vol:vortrap.cs`: the ending's command sequence after
  `c_imdone.kqs` finishes.
- GUI resources `20sl.pft`, `27sl.pft`, `kqteam08.pbm`, and `saveload.ppl`
  through the original resource search. Keep these user-supplied assets
  outside the source repository.

## Edge cases and do not reproduce

- The viewer does not skip several cards to catch up after an update
  stall. Preserve that visible rule; do not tie card time to frames.
- Missing card text leaves the cleared screen and retries that case
  after the basic delay rather than skipping it. Empty or malformed
  messages, excessive line counts, invalid levels, and unbounded format
  strings are not supported content cases. Reject them safely in the
  recreation instead of reproducing memory errors or division by zero.
- The original compares absolute 32-bit system millisecond deadlines;
  long-system-uptime wraparound is an implementation defect, not a
  required credits feature. A monotonic duration with the stated normal
  behavior is appropriate.
- The original's incorrect usage text for malformed `doCredits`
  invocations is not a gameplay feature; report the actual command's
  no-argument usage clearly.
- Do not reproduce a movie failure that waits indefinitely for a stopped
  state or repeatedly reports a backend error. Preserve the distinction
  between an unreadable/missing file and a reported media failure. Safe
  cleanup and continuation after acknowledging an existing-file playback
  error may be used as a compatibility safeguard, but must not be
  described as a verified original continuation for every failure.

## How to check automatically

No manual or original-runtime test was performed. Use representative
families and numeric checks rather than a separate test for each card:

1. Read the installed message data: cases 1–25 must be available, case
   order must differ from file order, and all style counts must fit the
   two original text capacities. There are 132 lines in total. Cases 19
   and 20 check that a terminal CRLF does not create a final empty row.
2. Use a controlled clock for a two-line card, a dense mixed-font card,
   and the final card. Check the durations and coordinates in the table,
   unchanged output before each deadline, one advance at the deadline,
   and one advance after a long delay.
3. Exercise ordinary Enter, Space, arrows, and pointer clicks before a
   deadline: no immediate next card. Escape and final-card expiry return
   to the main menu. Reentering begins at card 1 with the photo hidden.
4. Compare score arithmetic at the level boundary: L=1/E=7 gives 7;
   L=2/E=3 gives 13; L=15/E=7 gives 163837; L=16/E=7 gives 313837;
   L=17/E=0 gives 463830. A session in which no game has started gives
   0 regardless of any leftover Connor object. A level-only change
   must affect the score.
5. At both original resolutions, check the two font selections,
   interleaved row order, unchanged glyph scale, photograph bounds,
   spacer rows, final score formatting, and cleanup when reopening.
6. Starting from a retained Connor with a nonzero score, process the
   ending's `killGame`, movie return and `doCredits` in order. Back to
   Game becomes unavailable at `killGame`; Connor remains available;
   card 25 uses his nonzero score. After credits returns to Main, Back
   to Game remains unavailable and reopening Credits retains that
   score. Also exercise a missing movie file without discarding the
   pending credits command.

## Open questions

- The full operating-system input path for modified Escape, key
  autorepeat, focus loss, and input-method events was not independently
  traced here. Ordinary Escape and the absence of a next-card action are
  established; do not infer additional shortcuts from this note.
- Audio synchronization with the ending movie or the ending script's
  separate music commands is not established by studying the credits
  viewer. Its card timing is not synchronized to an audio track.
- The exact text and number of Windows media-error acknowledgements,
  and whether each codec/setup failure eventually reaches the stopped
  state after acknowledgement, remain untested and depend on the
  external media service. No universal original failure-return contract
  is established by rule 22.
- Palette-load failure and malformed-data handling were read statically,
  not exercised. There has been **no manual testing**.
