#[path = "support/daventry.rs"]
mod daventry;
use daventry::{earn_magic_map, game, test_root};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn new_game_walks_to_the_wizard_and_earns_the_map() {
    earn_magic_map(&game(), &test_root("wizard-route"));
}
