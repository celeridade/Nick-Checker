//! Checkers nativos (29 hand-written + ~1176 gerados de manifest/WMN).
//! Gerados: ver tools/gen_sites.py. Um modulo por site, pastas por categoria.

pub mod archived;
pub mod art;
pub mod blog;
pub mod business;
pub mod coding;
pub mod community;
pub mod dating;
pub mod dev;
pub mod finance;
pub mod gaming;
pub mod health;
pub mod hobby;
pub mod images;
pub mod misc;
pub mod music;
pub mod news;
pub mod nsfw;
pub mod political;
pub mod shopping;
pub mod social;
pub mod streaming;
pub mod tech;
pub mod video;

// Re-exports das 29 nativas (API publica historica).
pub use social::tiktok;
pub use social::tiktok_session;
pub use social::instagram;
pub use social::snapchat;
pub use social::pinterest;
pub use social::letterboxd;
pub use social::bluesky;
pub use social::x;
pub use social::telegram;
pub use social::guns_lol;
pub use social::tumblr;
pub use social::threads;
pub use social::medium;
pub use dev::github;
pub use dev::gitlab;
pub use dev::reddit;
pub use gaming::discord;
pub use gaming::steam;
pub use gaming::lichess;
pub use gaming::chess_com;
pub use gaming::roblox;
pub use gaming::minecraft;
pub use streaming::youtube;
pub use streaming::spotify;
pub use streaming::twitch;
pub use streaming::kick;
pub use music::soundcloud;
pub use music::lastfm;
pub use music::myspace;
pub use community::imgur;
