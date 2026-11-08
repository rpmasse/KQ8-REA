# Existing-file movie failure completion

The real bounded decoder reader is now shared with a controlled-child
fixture. Production FFmpeg arguments, AVI metadata/audio ownership and
frame-clock behavior are unchanged. The test uses original ending AVI
metadata and synthetic RGBA output from hidden child processes; it never
writes or extracts original game media.

One grouped headless matrix exercises the production reader, movie update,
error-page rendering, acknowledgement and return handling for:

- No first picture after decoder failure.
- A complete picture followed by truncated RGBA data.
- A complete picture followed by premature end of stream.
- A nonzero exit after the declared final picture. This case uses a
  one-interval metadata fixture; full original timing was already checked.

The failure page matches the original menu renderer's error output. Space
does not acknowledge it. Physical and queued Enter/Escape do. Title, Main
and Gameplay returns restore the cursor and preserve retained world state.
An ending failure waits for acknowledgement, then starts the queued Credits
without a gameplay update. Ended sessions go directly to New's difficulty
choice; playable sessions first show the restart confirmation with Cancel
selected. The check observes these through public menu input actions.

The movie and top-bar matrices pass first attempt, **2/2 in 1.53 s**, after
the shared 2m56s core/viewer build. Prior successful full-movie decoding,
Credits timing and native presentation checks were retained, not replayed.

```powershell
cargo test --locked --release -p kq8-viewer --bin kq8-viewer movie_failures_acknowledge_and_preserve_return_ownership -- --ignored --nocapture
```

Set `KQ8_GAME_DIR` to the absolute read-only GOG path. Evidence is in
`kq8-artifacts/logs/m4-input-and-movie-failures.log` and the
`movie-failure-matrix.log` / `movie-failure-active.log` files alongside it.

Scope: this verifies reader/playback failures after metadata parsing and
child creation. It does not retest decoder-spawn or malformed-AVI errors,
audible synchronization, exact Windows MCI dialog wording or original
runtime failures. Safe acknowledgement and continuation are the explicitly
allowed safeguard in `behaviour/credits-completion.md` rule 22 and “Do not
reproduce”; they are not a claim that every original MCI error returns.
No native game window or manual testing was used by this matrix.
