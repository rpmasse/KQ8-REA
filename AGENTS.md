# AGENTS.md

## Project
This fork recreates **King's Quest: Mask of Eternity** in Rust for Windows. Use the original GOG release as the reference for gameplay, controls, timing and presentation. Follow the M0–M7 milestones in the [completion plan](docs/DESIGN.md#kq8-rea-completion-plan-2026-11-05). Use automated checks during implementation. Manual testing starts after the recreation is feature-complete.

## Hard rules, never break these

1. **Never write game assets, decompiled code, or extracted game files into this
   repository.** They stay on this machine, untracked. If you need to read game
   data, read it from the install path at runtime or extract it to a gitignored
   folder.
2. **`.gitignore` is a whitelist.** It ignores everything and includes only
   source files. Do not switch it to a normal ignore list.
3. **Do not run `git commit` unless I asked.** Stage nothing beyond what the
   current task requires.
4. **Do not touch anything outside this project folder** unless I explicitly name
   the path. This includes my game installs: read them, never write to them.
5. **Single-player and offline only.** If this touches online play, anti-cheat, or
   DRM, stop and tell me. Offline play with anti-cheat switched off through the
   game's own option is fine. Bypassing anti-cheat is never fine.
6. **Never put credentials in the repo or in any file you can read:** no API keys,
   no tokens, no passwords.

## How to work

- **Plan before code.** For anything more than a small fix, write the plan to
  `docs/DESIGN.md` first, then implement one step at a time.
- **One thing at a time.** Do not bundle unrelated changes. I want to be able to
  revert a single step.
- **Log, don't look.** You cannot see the game. Instrument instead: write
  positions, counts, timings, and state transitions to a log file so you can
  verify from the numbers. Do not try to visually inspect the game.
- **Tell me how to test it.** After every change, say the exact command to run and
  what I should see. "Done" without a test procedure is not done.
- **Ask before large refactors.** If you think the architecture is wrong, say so
  and explain, then wait for me.
- **Explain in plain language.** I am not the programmer here. If you use a term,
  explain it the first time.

## Honesty

- If something is not tested, write **"not tested"**. Never imply you verified
  something you didn't.
- If you are not sure, say you are not sure. A confident wrong answer costs me
  hours.
- If you hit something you cannot solve after two real attempts, stop and write
  up `STATUS.md` (see `STATUS-handoff.md`) instead of trying variations at random.
- Record failures alongside successes. A dead end I can see is worth more than a
  dead end I have to watch you repeat.

## Keep these files updated

- `MODLOG.md`: add an entry after every change. Template in
  `MODLOG-template.md`.
- `docs/DESIGN.md`: how the project works, in plain language. Update when the
  architecture changes, not on every commit.
- `README.md`: the "what works / what doesn't work" list, any new keybindings,
  "Current State" (very brief), any new files/directories. Test before you claim
  something works. New entries should be kept very brief, one sentence max.
  Keep the features list generic, don't add overly specific things.

## Environment

- OS: Windows 11
- Original game reference: King's Quest: Mask of Eternity, version 1.03 from GOG.com, in the sibling `../King's Quest 8 GOG/` directory. Set `KQ8_GAME_DIR` to its absolute path for automated checks. Read this installation without modifying it.
- Language and version: Rust stable with MSVC
- Agent: Claude Code
