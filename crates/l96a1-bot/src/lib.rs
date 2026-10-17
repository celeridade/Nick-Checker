//! l96a1-bot — bot Discord da Fase 3 (setup de servidor, captcha, comandos snipe).
//!
//! O motor de check continua no `l9 snipe` (evidências + proxies + throttle);
//! o bot cuida de servidor, verificação e watchlist. Sem dependência
//! circular: o CLI monta os params e chama estas funções.

// poise 0.7 (serenity 0.12): comandos híbridos prefixo+slash, sem StandardFramework.
pub mod captcha;
pub mod run;
pub mod setup;

pub use run::{run_bot, BotData, BotRunConfig, Ctx, Error, all_commands};
pub use setup::{setup_guild, RoleScheme, SetupResult};
