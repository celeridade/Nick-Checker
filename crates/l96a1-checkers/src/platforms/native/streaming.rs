//! Checkers — categoria `streaming` (nativas + geradas: ver tools/gen_sites.py).

#[path = "../streaming/youtube.rs"]
pub mod youtube;
#[path = "../streaming/spotify.rs"]
pub mod spotify;
#[path = "../streaming/twitch.rs"]
pub mod twitch;
#[path = "../streaming/kick.rs"]
pub mod kick;

// --- gerados (tools/gen_sites.py) ---
#[path = "../streaming/mf_kick.rs"]
pub mod mf_kick;
#[path = "../streaming/mf_twitch.rs"]
pub mod mf_twitch;
