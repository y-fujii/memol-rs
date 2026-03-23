pub mod compile_thread;
pub mod jack;
pub mod notify;
pub mod player;
pub mod player_dummy;
pub mod player_jack;
#[cfg(target_family = "windows")]
pub mod player_winmm;
use std::*;

pub fn new_player(name: &str) -> Box<dyn player::Player> {
    if let Ok(player) = player_jack::Player::new(name) {
        return Box::new(player);
    }
    #[cfg(target_family = "windows")]
    if let Ok(player) = player_winmm::Player::new() {
        return Box::new(player);
    }
    Box::new(player_dummy::Player::new())
}
