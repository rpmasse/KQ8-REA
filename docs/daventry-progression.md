# Daventry campaign route

Daventry has two regional exits. Castle Daventry contains a required ingredient and leads back to Daventry. The gate in the churchyard tomb advances the story to the Dimension of Death, called `DeadCity` in the data. These prerequisites come from the original GOG game scripts. They describe the route to verify, independently of whether the Rust game has passed each part.

| Stage | What the player does | What establishes progress |
| --- | --- | --- |
| Start the story | Select New Game, choose Normal and let the cottage sequence finish. Walk to the wizard and speak to him. | In `wizard.cs`, the eighth line of the first conversation exchanges `Piece1A` for `Mask1` and `Piece1`. The thirteenth line starts the map sequence and marks the first meeting. Its later cue gives the Magic Map. |
| Learn what blocks the route | Reach the Shadow encounter in the churchyard tomb and return to the wizard. | `sdwtrap2.cs` and `shadow.cs` record the meeting. With the first wizard meeting and the Shadow encounter recorded, `wizard.cs` gives the ingredient conversation. Its fifth line changes the wizard's state to 2, where he will accept the ingredients. |
| Obtain the Candle | Reach and interact with the church candle. | `newworld.cs` places `candle.kq`. `candle.cs` grants one Candle, clears the full flag and removes the prop. |
| Obtain the dead hero’s ring | Enter the hero’s tomb, open the coffin and take the ring. | `tombdor2.cs` rejects fists and the dagger when breaking the tomb door. `coffnlid.cs` changes the coffin interaction when its fall ends. `worldinv.cs` places `RingOfDeadHero` inside. `ring.cs` supplies its pickup response. |
| Obtain the Ashes | Use the cave exit to Castle Daventry, take the Ashes and return. | Daventry’s `portaltp.cs` starts `portal1`. CastleD’s `worldinv.cs` places the Ashes. Its `portalt2.cs` and `portal2.kq` bring Connor back. |
| Make the Ring of Light | Present each ingredient to the wizard while he is in state 2 and finish the casting sequence. | `wizard.cs` consumes each offered item and remembers it. Its three flags are tested together. The final `castring4` callback grants the Ring of Light and changes the wizard’s state to 3. |
| Leave Daventry | Enter the churchyard tomb’s gate with the Ring of Light and accept the travel prompt. | On a first visit, `sdwtrap1.cs` checks Connor’s inventory and then records `Daventry::BeenThere::DoneThat`. `davgate4.kq` names Dead City and its arrival coordinates. Later visits use the recorded flag. |

### Other Daventry branches

These scripts are relevant to a complete recreation. Some can provide access or equipment for the main route. The ingredient test above names only the Candle, Ashes and dead hero’s ring.

| Branch | What to verify in play |
| --- | --- |
| Church and tomb access | Church roof and entry triggers, pushing the urn and its delayed opening of the tomb door, the archer encounter and crossbow reward. Sources include `churchr3.cs`, `urn1.cs`, `rooftrap.cs` and `sprgnbow.cs`. |
| Weapons and navigation | The dagger has a loose placement in `worldinv.cs`. The axe pickup gives the Small Axe and starts the henchman sequence. Hanging and ringing the bell three times brings out the lake lady, who can grant the Lake Sword. The mill tree checks the weapon and changes the crossing. The rope and hook has its own pickup and traversal interactions. Sources include `dagger.cs`, `ax.cs`, `henchman.cs`, `belstnd.cs`, `bell.cs`, `lady.cs`, `milltree.cs` and `ropenhok.cs`. |
| Wizard's book and the star globe | The book's animation gives a Spell Page. Offering it to the star globe leads to a Clarity potion. The page and the potion are separate inventory and object-state checks. Sources include `wizbook.cs` and `starball.cs`. |
| The beast, unicorn and pyramids | The beast handles the Unicorn Horn. The unicorn sequence can give the Dark Pyramid. The Crystal Pyramid has its own pickup. These branches need separate coverage when following the later regional story back through Daventry. Sources include `ugly.cs`, `unicorn.cs` and `pyramid.cs`. |
| Castle keep and travel | Using the Keep Key on the locked keep door earns teleport progress. The castle cave’s first discovery grants experience separately. Sources include `keepdoor.cs` and `portaltp.cs`. |
| Village, supplies and discoveries | Cottage ravens, baskets, fish, the inn, the girl, bows and barrels, the alchemist's obstruction, the waterfall, health and equipment need their own interaction coverage. Original files include `condoor.cs`, `ravencnr.cs`, `basket2.cs`, `girl.cs`, `alchdoor.cs`, `boardup.cs`, `bricks.cs` and `worldinv.cs`. |

Physical access can add prerequisites that an inventory check alone cannot show. A full Daventry-to-Dead City run must start with an empty save directory and use ordinary player controls across all three ingredient pickups, wizard exchanges and both travel prompts.

### Verified through the first wizard meeting

The unattended Windows run started at New Game → Normal with a fresh save directory. Connor completed the opening sequence and walked from Sarah's cottage to the wizard. The replay pressed his configured forward and turn keys, leaving movement, collision, health and enemy AI active. A first route hit the farmhouse wall. The final route walked around its south side. On reaching the wizard the replay submitted a named world click, which ran his normal distance and script handling.

The eighth spoken response exchanged the initial fragment for `Mask1` and `Piece1`. The thirteenth started the five-part map sequence. Its fourth animation cue awarded `MagicMap`. A slot written after the final line contained all three items, recorded the first meeting and confirmed removal of `Piece1A`. The replay acknowledges the original introduction, movement, camera and map help. It checks that the map tip displays the bound Tab key. The ingredient and portal stages above await their own continuous run.

With `KQ8_GAME_DIR` set to the original GOG folder, run:

```sh
cargo test --locked -p kq8-viewer --test daventry_wizard -- --ignored
```

The test opens and closes the Rust game automatically. Its input route is in `crates/kq8-viewer/tests/routes/daventry_wizard.json`, with logs and the resulting slot under `kq8-artifacts/wizard-route/`.

### Verified from the wizard to the windmill axe

Connor loaded his earned Magic Map slot and walked east of the goblin patrol. He picked up the original stacks of two and three mushrooms, climbed the hill and opened the windmill door. He drew his fists, defeated the spriggan archer, put his fists away and tried the axe from the floor. Its script refused him at a distance of 2,415 units. After he pushed the hay bale under the axe and jumped onto it, the same check measured 1,528 units, within the original 1,700-unit reach. The script gave one Small Axe, awarded 25 experience and placed the trigger for the henchman encounter. Connor was alive with the axe in the saved game.

The replay presses the bound movement, draw, away and jump keys. Like the wizard route, it uses named targets for original world clicks and melee attacks. Normal combat, healing, attack range, collision and the axe's distance test all run. The resulting slot marks the beginning of the henchman branch. A fresh-process reload now preserves the moved bale and axe positions. Completing the encounter and reaching the water mill are subsequent checks.

The lake bell requires the Small Axe in hand. `bell.cs` checks the equipped short weapon against item 1020 before counting a strike. The bell comes from the wizard's hidden chest. The tower book gives the Spell Page, the star globe exchanges it for Clarity, and revealing the chest makes the Lady Bell available. Three counted strikes summon the Lady. These dependencies have been identified in the game data. Their complete sequence still needs a continuous campaign run.

```sh
cargo test --locked -p kq8-viewer --test daventry_windmill -- --ignored
```

The default test earns the Magic Map through New Game in its first process, then loads it from disk in the second. To repeat only the windmill leg, set `KQ8_WIZARD_SLOT` to the Magic Map save produced by the wizard route. Each test creates a fresh working directory under `kq8-artifacts/windmill-route/`. Original game assets and save files stay local.

After the windmill save, another process loads Connor on the bale. It checks that his feet stay on the raised surface, that the axe is still at the position set by its pickup script, and that trying it again leaves his inventory and experience intact. It saves again to check both positions survive another save. The original persistence manager determines which props belong in the save. For those props, the Rust game records the position and orientation after a push or a script move. A prop missing from an older Rust save starts at its original location.
