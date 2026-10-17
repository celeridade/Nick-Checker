//! Checkers — categoria `dev` (nativas + geradas: ver tools/gen_sites.py).

#[path = "../dev/github.rs"]
pub mod github;
#[path = "../dev/gitlab.rs"]
pub mod gitlab;
#[path = "../dev/reddit.rs"]
pub mod reddit;

// --- gerados (tools/gen_sites.py) ---
#[path = "../dev/mf_github.rs"]
pub mod mf_github;
#[path = "../dev/mf_gitlab.rs"]
pub mod mf_gitlab;
