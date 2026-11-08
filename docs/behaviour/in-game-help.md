# In-game help

## Three ways to open the panel

* **Scripted tutorial.** `doPopup N` schedules a request to Connor for **3 seconds of game time later**. He can deliver it in Wait or Move when `handsOff` is clear, his speech queue is empty and at least 3 seconds have passed since his previous delivery attempt reached the UI. Equality at 3 seconds is sufficient. While those conditions fail, a request is held and retried after **1 second**. Later held tips can be postponed a further **3 seconds** to let an earlier tip through. A busy popup panel also reschedules the arriving request for **3 seconds** later. Occupied panels are preserved.
* When an ordinary tutorial reaches the UI, a nonzero `notStartingHelp` suppresses it. The check happens at delivery. The popup plays `button1.aud` on opening, uses case **148** for **OK** and case **147** for the option to turn off starting help. Clicking the checkbox mark or its label toggles the preference immediately, displays **X** for off, and leaves the tip open. Clicking it again restores future tips. OK closes the panel and resumes play. It sends no tutorial answer back to the requesting script. In the installed `Options.cs`, `notStartingHelp 0` enables tutorials. The reserved value 2 suppresses them and cannot be toggled in Options. The executable bypasses the preference and omits the checkbox for ordinary case numbers 170 or higher. Missing message cases are outside this study.
* **F1 in gameplay.** With the popup closed, F1 opens the catalog immediately at its first page, even if automatic tips are disabled. It plays `intrface.aud`. Its buttons, left to right, are **Close** (case 155), **<<**, **>>**. There is no checkbox. Previous and Next replace only the text and wrap in both directions. Reopening F1 starts at the first page again.
* **Unlit Go on the Magic Map.** Clicking it displays case **151** immediately, even with automatic tips disabled. It has only OK, and omits the checkbox. A scripted `doPopup 151` takes the ordinary delayed tutorial path. The Wizard's `wizmap4` gives Connor the map. At `wizmap5`, `wizard.cs` releases camera/control, starts another conversation and requests case **153**. That request still waits for Connor's normal delivery conditions.

## Message catalog and text

Read `GAME/ENGLISH/500.MSG` with noun 0, verb 0 and sequence 1. F1 has **24 pages**. In order they are cases **127 through 146**, then **149, 150, 151, 153**. The first is the camera tutorial. The last is the map tutorial. Cases 147, 148, 152 and 155 are interface labels. Before wrapping, replace the caret and following binding code in the message with the player's current key label. Keep the surrounding square brackets. Case 153 uses the Map binding.

Use the seven center-aligned text lines of the gameplay popup with the 38-character wrapping and per-line positions described in [world-change.md, rule 26](world-change.md#the-yesno-popup-unknown-6). Extra empty lines are cleared. A long tutorial is wrapped within a page. Next and Previous change topics.

## Controls at 640 × 480

The `IDCTL_POPUP` dialog in `GAME/Resource/Gplayscr.gui` occupies (140,120) to (510,320). The following coordinates are **relative to its top-left**. Its background `buywin4.pbm` occupies (0,0) to (360,200). The font is `console.pft`. The checkbox uses `buybtndn.pbm` at (17,168) to (33,184), with X over that box and the clickable label from (40,168) to (360,184).

| Mode | Button positions at opening |
| --- | --- |
| Scripted tutorial | OK at (141,142) to (229,167) |
| F1 catalog | Close at (48,145) to (136,183), Previous at (140,146) to (228,182), Next at (232,146) to (320,182) |
| Unlit Go | OK at (141,145) to (229,183) |

The highlighted button uses `buyhibtn.pba`. Other buttons use `yesbtn.pba`. For a held click or keyboard press, see the pressed frame and label offset in [world-change.md, rule 26](world-change.md#the-yesno-popup-unknown-6). At 800 × 600 the button offsets for the two resolutions differ by 44 vertically. This study checked the 640 × 480 layout in the shipped file. The complete 800 × 600 composition is open.

## Pausing, input and sounds

Displaying text pauses world simulation while music and ongoing sounds continue. Changing a catalog page keeps it paused. A normal dismissal releases this pause. Activating a button or toggling the checkbox plays `button1.aud`. A successful keyboard highlight change also plays it.

On opening, the leftmost button has focus. Left and Right **move focus by one button** while a popup is open. At an end they leave focus there. An arrow press alone does not change the page. Clicking Previous or Next, or activating the focused paging button, changes it. With a popup already open, **Enter or F1 press** presses the focused button, and **release** activates it if that press began with the popup open. Releasing the F1 that opened the catalog does not close it. With the initial focus, a second F1 press and release closes the catalog. The mouse moves to the pressed button for keyboard activation and returns on release.

Escape opens Main with the usual handsOff exception. When accepted while help is visible, it closes that panel before opening Main. Choosing Return resumes gameplay with help closed: the displayed tutorial or catalog page is not restored or requested again by this menu round trip. A later F1 opens the catalog at its first page with Close focused, as on any fresh opening. Other pending tutorial requests remain separate from the dismissed panel.

Tutorial waits use game time. Time spent paused in Main does not consume the initial **3-second** delay, the **1-second** retry, or later **3-second** postponements. Return resumes the remaining game-time waits without adding the wall-clock time spent in the menu. A pending request therefore does not become due merely because the player leaves Main open.

## How to check and remaining limits

* Remap Map and trigger the Wizard's `wizmap5` from its shipped script. Check that case 153 appears after Connor is free and its text shows the new binding. Toggle the checkbox, dismiss, and request another scripted tip. It should be suppressed. F1 should still open the camera page, and an unlit map Go should still open the teleport tip.
* On F1's first page move focus to Previous and activate it. This should show case 153. Return to the first page with Next. Reopen F1 and check that both the first page and Close focus have reset. Track simulation ticks, playing sound and requests across each dismissal.
* With a tutorial visible, use Escape, then Main's Return. Check that gameplay resumes with the panel closed. Repeat from a later F1 page; after Return, F1 should open the first page with Close focused. Separately, pause in Main while a tutorial delay or retry still has game time remaining, wait longer than that interval in real time, and Return. The request should retain its remaining game-time wait.
* Do not reproduce accidental duplication of identical held requests. The behavior with several identical pending tips, persistence through save/load, and all gameplay inputs outside the panel were not verified.

Studied statically in the owned GOG executable. The installed message file, layout and Wizard script were read separately. The Escape/Main/Return result follows the help dismissal and gameplay return paths; the timing result follows tutorial scheduling and the paused game-clock update. This bounded follow-up does not cover loading a save, starting a new game, or changing display resolution while Main is open. Runtime comparison with the original: **not tested**.
