//! Gateway do bot em **poise 0.7** (serenity 0.12): 1 definição = prefixo + slash.
//!
//! Substitui o `StandardFramework` depreciado (removido). Cada comando abaixo
//! é híbrido (`prefix_command + slash_command`), então `!purge` e `/purge`
//! passam pelo MESMO código — a classe de bug "slash ok, prefixo mudo"
//! deixa de existir por construção.
//!
//! A watchlist é o mesmo arquivo do sniper (`watchlist.txt`): o bot edita,
//! o sniper observa. Hits saem pelo sniper via webhooks (sem gateway).
//!
//! Comandos (prefixo `!` por padrão + `/slash` automáticos):
//! - `help` · `ping` · `verify` · `check <nick> [plats]` · `status`
//! - `snipe [add|remove|list]` (+ `/snipe_add|/snipe_remove|/snipe_list`)
//! - moderação: `ban|kick|mute|unmute|purge|slowmode|lock|unlock|userinfo`
//! - `nuke CONFIRM` (só prefixo, destrutivo, sem slash por segurança)

use poise::serenity_prelude as serenity;
use serenity::{
    ChannelId, Colour, CreateAttachment, CreateEmbed, CreateMessage, EditChannel, EditMember,
    GetMessages, GuildId, Message, MessageId, Mentionable, PermissionOverwrite,
    PermissionOverwriteType, Permissions, RoleId, Timestamp, User, UserId,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct BotRunConfig {
    pub token: String,
    pub guild_id: u64,
    pub prefix: String,
    pub verified_role: String,
    pub captcha: bool,
    pub verify_channel_id: u64,
    pub watchlist_path: String,
}

/// Estado compartilhado (poise `Data`): config + captchas pendentes + boot.
pub struct BotData {
    pub cfg: BotRunConfig,
    pub pending: Arc<Mutex<HashMap<UserId, Pending>>>,
    pub started: Instant,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Ctx<'a> = poise::Context<'a, BotData, Error>;

pub struct Pending {
    solution: String,
    attempts: u8,
}

// --- watchlist (mesmo formato do sniper: 1 nick por linha) ---

fn read_watchlist(path: &str) -> Vec<String> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(|l| l.trim().to_lowercase())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn write_watchlist(path: &str, users: &[String]) -> Result<(), String> {
    if let Some(p) = std::path::Path::new(path).parent() {
        let _ = std::fs::create_dir_all(p);
    }
    let mut t = String::new();
    for u in users {
        t.push_str(u);
        t.push('\n');
    }
    std::fs::write(path, t).map_err(|e| format!("watchlist: {e}"))
}

async fn verified_role_id(http: &serenity::Http, guild: GuildId, name: &str) -> Option<RoleId> {
    let roles = guild.roles(http).await.ok()?;
    roles.iter().find(|(_, r)| r.name == name).map(|(id, _)| *id)
}

/// Permissões efetivas no nível da guild (dono + @everyone + cargos).
/// Não usa `Member::permissions` (depreciado no 0.12) e sim a dobra direta
/// dos cargos — overwrites de canal são irrelevantes p/ admin/moderação.
async fn guild_perms(sctx: &serenity::Context, guild_id: GuildId, user_id: UserId) -> Permissions {
    let Ok(member) = guild_id.member(&sctx.http, user_id).await else {
        return Permissions::empty();
    };
    let Some(guild) = sctx.cache.guild(guild_id) else {
        return Permissions::empty();
    };
    if guild.owner_id == user_id {
        return Permissions::ADMINISTRATOR;
    }
    // @everyone tem o id da guild.
    let mut perms = guild
        .roles
        .get(&RoleId::new(guild_id.get()))
        .map(|r| r.permissions)
        .unwrap_or_else(Permissions::empty);
    for rid in &member.roles {
        if let Some(r) = guild.roles.get(rid) {
            perms |= r.permissions;
        }
    }
    perms
}

/// Admin do servidor (Manage Server ou Administrator) — p/ watchlist do sniper.
async fn is_admin(sctx: &serenity::Context, guild_id: GuildId, user_id: UserId) -> bool {
    let p = guild_perms(sctx, guild_id, user_id).await;
    p.administrator() || p.manage_guild()
}

/// Dono ou Administrator — p/ moderação destrutiva.
async fn is_root(sctx: &serenity::Context, guild_id: GuildId, user_id: UserId) -> bool {
    guild_perms(sctx, guild_id, user_id).await.administrator()
}

// --- checks do poise (falha vira "⛔ só admin." via on_error, nunca silêncio) ---

/// Check poise: Manage Server ou Administrator.
async fn check_admin(ctx: Ctx<'_>) -> Result<bool, Error> {
    let Some(gid) = ctx.guild_id() else { return Ok(false) };
    Ok(is_admin(ctx.serenity_context(), gid, ctx.author().id).await)
}

/// Check poise: dono do servidor ou Administrator.
async fn check_root(ctx: Ctx<'_>) -> Result<bool, Error> {
    let Some(gid) = ctx.guild_id() else { return Ok(false) };
    Ok(is_root(ctx.serenity_context(), gid, ctx.author().id).await)
}

async fn issue_captcha(
    http: &serenity::Http,
    cfg: &BotRunConfig,
    pending: &Arc<Mutex<HashMap<UserId, Pending>>>,
    user: User,
) {
    let (solution, png) = crate::captcha::generate(5);
    pending.lock().await.insert(user.id, Pending { solution: solution.clone(), attempts: 0 });
    let text = format!(
        "{} complete o captcha para liberar os canais (5 caracteres — digite aqui mesmo):",
        user.mention()
    );
    if cfg.verify_channel_id != 0 {
        let ch = ChannelId::new(cfg.verify_channel_id);
        let _ = ch
            .send_files(
                http,
                [CreateAttachment::bytes(png, "captcha.png")],
                CreateMessage::new().content(text),
            )
            .await;
    } else if let Ok(dm) = user.create_dm_channel(http).await {
        let _ = dm
            .send_files(
                http,
                [CreateAttachment::bytes(png, "captcha.png")],
                CreateMessage::new().content("Complete o captcha para liberar os canais:"),
            )
            .await;
    }
    let _ = solution;
}

// --- texto de ajuda (única fonte p/ !help e /help) ---

fn help_text(prefix: &str) -> String {
    format!(
        "**⌖ L96A1 // BOT** — prefixo `{p}` + `/slash` (poise híbrido: mesmo código)\n\
        \n**Consulta**\n\
        • `{p}check <nick> [plats]` — disponibilidade ao vivo (`/check`)\n\
        • `{p}status` — watchlist, uptime, latência (`/status`)\n\
        • `{p}ping` — HUD online (`/ping`)\n\
        • `{p}userinfo [@user]` — ficha (`/userinfo`)\n\
        \n**Watchlist do sniper** (admin p/ add/remove)\n\
        • `{p}snipe add <nick>` (`/snipe_add`)\n\
        • `{p}snipe remove <nick>` (`/snipe_remove`)\n\
        • `{p}snipe` / `{p}snipe list` (`/snipe_list`)\n\
        \n**Verificação**\n\
        • `{p}verify` — reemite seu captcha (`/verify`)\n\
        \n**Moderação** (só admin)\n\
        • `{p}ban <@user> [motivo]` (`/ban`)\n\
        • `{p}kick <@user>` (`/kick`)\n\
        • `{p}mute <@user> [10m|2h|1d]` (`/mute`)\n\
        • `{p}unmute <@user>` (`/unmute`)\n\
        • `{p}purge <n|all> [@user]` — lote 100 + 1s (rate-limit) (`/purge`)\n\
        • `{p}slowmode <s|0>` (`/slowmode`)\n\
        • `{p}lock` / `{p}unlock` (`/lock` `/unlock`)\n\
        • `{p}nuke CONFIRM` — apaga TODOS os canais (só prefixo, sem slash)\n\
        \nEx: `{p}check dark github,instagram` · `{p}snipe add dark` · `{p}purge all`\n\
        Se `{p}` não responde e `/` sim: ative **Message Content Intent** no portal do bot.",
        p = prefix
    )
}

// --- motor de check ao vivo (mesmas evidências dos checkers nativos) ---

struct CheckLine {
    platform: String,
    tag: String,
    symbol: String,
    link: String,
    url: String,
    thumb: Option<String>,
    available: bool,
}

fn avatar_for(platform_key: &str, nick: &str) -> Option<String> {
    match platform_key {
        "minecraft" | "namemc" => Some(format!("https://mc-heads.net/avatar/{nick}")),
        _ => None,
    }
}

async fn run_check(nick: &str, platforms_q: &str) -> Vec<CheckLine> {
    let nick = nick.trim().to_lowercase();
    if nick.is_empty() || nick.len() > 64 {
        return Vec::new();
    }
    let platforms = l96a1_checkers::resolve_with(platforms_q, &[]);
    if platforms.is_empty() {
        return Vec::new();
    }
    let client = Arc::new(l96a1_core::build_client(Duration::from_secs(10), None));
    let futs: Vec<_> = platforms
        .into_iter()
        .map(|plat| {
            let client = Arc::clone(&client);
            let nick = nick.clone();
            async move {
                let norm = match plat.validator().normalize(&nick) {
                    Some(n) => n,
                    None => {
                        return CheckLine {
                            platform: plat.key().to_string(),
                            tag: "INVALID".to_string(),
                            symbol: "–".to_string(),
                            link: String::new(),
                            url: String::new(),
                            thumb: None,
                            available: false,
                        };
                    }
                };
                let url = plat.profile_url(&norm);
                let key = plat.key().to_string();
                let thumb = avatar_for(&key, &norm);
                let ctx = l96a1_core::CheckCtx {
                    client,
                    proxy_url: None,
                    discord_tokens: Vec::new(),
                    tiktok_sessions: Vec::new(),
                    github_token: None,
                    timeout: Duration::from_secs(10),
                };
                let o = plat.check(&norm, &ctx).await;
                CheckLine {
                    platform: key,
                    tag: o.result.tag().to_string(),
                    symbol: o.result.symbol().to_string(),
                    link: l96a1_core::short_link(&url),
                    url,
                    thumb,
                    available: o.result == l96a1_core::CheckResult::Available,
                }
            }
        })
        .collect();
    let mut out: Vec<CheckLine> = futures::future::join_all(futs).await;
    out.sort_by(|a, b| b.available.cmp(&a.available).then(a.platform.cmp(&b.platform)));
    out
}

fn check_embed(nick: &str, lines: &[CheckLine]) -> CreateEmbed {
    let hits: Vec<&CheckLine> = lines.iter().filter(|l| l.available).collect();
    let title = if hits.is_empty() {
        format!("🔒 {nick} — ocupado")
    } else {
        format!("🎯 {nick} — {} disponível(is)", hits.len())
    };
    let mut desc = String::new();
    for l in lines.iter().take(25) {
        if l.available {
            desc.push_str(&format!("✅ `{}` — {}\n", l.platform, l.link));
        } else {
            desc.push_str(&format!("{} `{}` {}\n", l.symbol, l.platform, l.tag));
        }
    }
    if lines.len() > 25 {
        desc.push_str(&format!("… +{} omitidas", lines.len() - 25));
    }
    if desc.len() > 3500 {
        desc.truncate(3500);
        desc.push('…');
    }
    let color = if hits.is_empty() { Colour::DARK_GREY } else { Colour::from_rgb(0, 200, 120) };
    let mut e = CreateEmbed::new().title(title).description(desc).colour(color);
    if let Some(first) = hits.first() {
        if !first.url.is_empty() {
            e = e.url(&first.url);
        }
        if let Some(t) = &first.thumb {
            e = e.thumbnail(t);
        }
        if first.thumb.is_none() && first.platform == "discord" {
            e = e.thumbnail("https://discord.com/assets/favicon.ico");
        }
    }
    e
}

/// Duração humana p/ mute: "10m", "2h", "1d" (default minutos).
fn parse_mins(s: &str) -> Option<u64> {
    let t = s.trim().to_lowercase();
    let (n, m) = if let Some(x) = t.strip_suffix('d') {
        (x, 1440)
    } else if let Some(x) = t.strip_suffix('h') {
        (x, 60)
    } else if let Some(x) = t.strip_suffix('m') {
        (x, 1)
    } else {
        (t.as_str(), 1)
    };
    n.parse::<u64>().ok().map(|v| v.saturating_mul(m).clamp(1, 40320))
}

/// Ficha curta do usuário — portado de CorwinDev `userinfo`.
async fn userinfo_text(sctx: &serenity::Context, guild_id: Option<GuildId>, uid: UserId) -> String {
    let user = sctx.http.get_user(uid).await.ok();
    let name = user.as_ref().map(|u| u.name.clone()).unwrap_or_else(|| format!("{uid}"));
    let created = user
        .as_ref()
        .map(|u| format!("<t:{}:D>", u.id.created_at().unix_timestamp()))
        .unwrap_or_else(|| "—".to_string());
    if let Some(gid) = guild_id {
        if let Ok(m) = gid.member(&sctx.http, uid).await {
            let joined = m
                .joined_at
                .map(|t| format!("<t:{}:D>", t.unix_timestamp()))
                .unwrap_or_else(|| "—".to_string());
            let roles = if m.roles.is_empty() {
                "—".to_string()
            } else {
                m.roles.iter().take(8).map(|r| format!("<@&{r}>")).collect::<Vec<_>>().join(" ")
            };
            return format!("👤 **{name}** (`{uid}`)\nConta criada: {created}\nEntrou: {joined}\nCargos: {roles}");
        }
    }
    format!("👤 **{name}** (`{uid}`)\nConta criada: {created}")
}

// --- motor único de purge (Red `cleanup` + discord.py `TextChannel.purge`) ---
//
// - bulk_delete em lotes de 100 (só <14 dias), fallback 1-a-1 p/ antigas;
// - 1s entre lotes (rate-limit do Discord);
// - `want=None` = `all` (varre até o canal esvaziar, com paginação `before`);
// - filtro opcional por autor.

async fn purge_channel(
    http: &serenity::Http,
    channel: ChannelId,
    want: Option<usize>,
    filter_user: Option<UserId>,
    bots_only: bool,
) -> usize {
    let mut gone = 0usize;
    let mut before: Option<MessageId> = None;
    loop {
        if let Some(w) = want {
            if gone >= w {
                break;
            }
        }
        let mut q = GetMessages::new().limit(100);
        if let Some(b) = before {
            q = q.before(b);
        }
        let batch = match channel.messages(http, q).await {
            Ok(b) => b,
            Err(_) => break,
        };
        if batch.is_empty() {
            break;
        }
        before = batch.last().map(|m| m.id);
        let targets: Vec<&Message> = batch
            .iter()
            .filter(|m| {
                if bots_only {
                    m.author.bot
                } else {
                    filter_user.map(|u| m.author.id == u).unwrap_or(true)
                }
            })
            .collect();
        if targets.is_empty() {
            if batch.len() < 100 {
                break;
            }
            continue;
        }
        let take = match want {
            Some(w) => (w - gone).min(targets.len()),
            None => targets.len(),
        };
        let ids: Vec<MessageId> = targets.iter().take(take).map(|m| m.id).collect();
        if ids.is_empty() {
            break;
        }
        // Bulk falha se QUALQUER msg tem >14 dias → fallback individual.
        if channel.delete_messages(http, &ids).await.is_err() {
            let mut ok = 0;
            for mid in &ids {
                if channel.delete_message(http, *mid).await.is_ok() {
                    ok += 1;
                } else {
                    tokio::time::sleep(Duration::from_millis(800)).await;
                }
                if ok % 5 == 0 {
                    tokio::time::sleep(Duration::from_millis(600)).await;
                }
            }
            gone += ok;
            if ok < ids.len() {
                break;
            }
        } else {
            gone += ids.len();
        }
        // Rate-limit: 1s entre bulks (~5/5s por canal).
        tokio::time::sleep(Duration::from_secs(1)).await;
        if batch.len() < 100 {
            break;
        }
    }
    gone
}

/// Interpreta o alvo do purge: `None|""`→lote padrão, `all`→tudo, `N`→N msgs.
fn parse_purge_target(raw: Option<String>) -> Result<Option<usize>, String> {
    match raw.map(|s| s.trim().to_lowercase()).as_deref() {
        None | Some("") => Ok(Some(50)),
        Some("all") => Ok(None),
        Some(s) => s
            .parse::<usize>()
            .map(|n| Some(n.clamp(1, 5000)))
            .map_err(|_| "uso: `!purge <n|all> [@user|bots]` · ex: `!purge 100`, `!purge all`".to_string()),
    }
}

// ---------------------------------------------------------------- comandos

/// HUD online.
#[poise::command(prefix_command, slash_command)]
pub async fn ping(ctx: Ctx<'_>) -> Result<(), Error> {
    ctx.say("⌖ HUD online.").await?;
    Ok(())
}

/// Lista todos os comandos (prefixo + slash).
#[poise::command(prefix_command, slash_command)]
pub async fn help(ctx: Ctx<'_>) -> Result<(), Error> {
    let prefix = &ctx.data().cfg.prefix;
    ctx.say(help_text(prefix)).await?;
    Ok(())
}

/// Reemite seu captcha de verificação.
#[poise::command(prefix_command, slash_command)]
pub async fn verify(ctx: Ctx<'_>) -> Result<(), Error> {
    let data = ctx.data();
    issue_captcha(&ctx.serenity_context().http, &data.cfg, &data.pending, ctx.author().clone()).await;
    ctx.send(poise::CreateReply::default().content("📩 captcha enviado — responda no canal de verificação (ou DM).").ephemeral(true)).await?;
    Ok(())
}

/// Checa disponibilidade de um username ao vivo.
#[poise::command(prefix_command, slash_command, user_cooldown = 10)]
pub async fn check(
    ctx: Ctx<'_>,
    #[description = "username a checar"] nick: String,
    #[description = "filtro: all, social, github,instagram..."] plats: Option<String>,
) -> Result<(), Error> {
    let plats = plats.unwrap_or_else(|| "all".to_string());
    ctx.defer().await?;
    let lines = run_check(&nick, &plats).await;
    if lines.is_empty() {
        ctx.say(format!("⚠️ nada bate com esse filtro de plataformas para `{nick}`.")).await?;
        return Ok(());
    }
    let embed = check_embed(&nick.to_lowercase(), &lines);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Watchlist, uptime e latência.
#[poise::command(prefix_command, slash_command, user_cooldown = 5)]
pub async fn status(ctx: Ctx<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let n = read_watchlist(&data.cfg.watchlist_path).len();
    let up = data.started.elapsed().as_secs();
    ctx.say(format!(
        "🎯 **{n} alvos** na watchlist · ⏱️ uptime {:02}:{:02}:{:02} · prefixo `{}` + `/slash` (poise)",
        up / 3600,
        (up % 3600) / 60,
        up % 60,
        data.cfg.prefix,
    ))
    .await?;
    Ok(())
}

/// Gerencia a watchlist do sniper: `!snipe add|remove|list` (só prefixo).
/// No slash use `/snipe_add`, `/snipe_remove`, `/snipe_list`.
#[poise::command(prefix_command)]
pub async fn snipe(ctx: Ctx<'_>, #[rest] args: Option<String>) -> Result<(), Error> {
    let data = ctx.data();
    let mut parts = args.unwrap_or_default().split_whitespace().map(str::to_string).collect::<Vec<_>>().into_iter();
    let sub = parts.next().unwrap_or_default().to_lowercase();
    match sub.as_str() {
        "add" | "remove" => {
            let sctx = ctx.serenity_context();
            let Some(gid) = ctx.guild_id() else {
                ctx.say("só em servidor.").await?;
                return Ok(());
            };
            if !is_admin(sctx, gid, ctx.author().id).await {
                ctx.send(poise::CreateReply::default().content("⛔ só admin (Manage Server).").ephemeral(true)).await?;
                return Ok(());
            }
            let nick = parts.next().unwrap_or_default().trim().to_lowercase();
            if nick.is_empty() || nick.len() > 32 {
                ctx.say("uso: `!snipe add <nick>`").await?;
                return Ok(());
            }
            let mut users = read_watchlist(&data.cfg.watchlist_path);
            if sub == "add" {
                if users.contains(&nick) {
                    ctx.say(format!("`{nick}` já está na watchlist.")).await?;
                } else {
                    users.push(nick.clone());
                    users.sort();
                    match write_watchlist(&data.cfg.watchlist_path, &users) {
                        Ok(_) => {
                            ctx.say(format!("👁️ `{nick}` na mira ({} alvos).", users.len())).await?;
                        }
                        Err(e) => {
                            ctx.say(format!("⚠️ falhou ao salvar: {e}")).await?;
                        }
                    }
                }
            } else if let Some(i) = users.iter().position(|u| u == &nick) {
                users.remove(i);
                match write_watchlist(&data.cfg.watchlist_path, &users) {
                    Ok(_) => {
                        ctx.say(format!("`{nick}` removido ({} alvos).", users.len())).await?;
                    }
                    Err(e) => {
                        ctx.say(format!("⚠️ falhou ao salvar: {e}")).await?;
                    }
                }
            } else {
                ctx.say(format!("`{nick}` não está na watchlist.")).await?;
            }
        }
        _ => {
            let users = read_watchlist(&data.cfg.watchlist_path);
            let body = if users.is_empty() {
                format!("(vazia — `{}snipe add <nick>`)", data.cfg.prefix)
            } else {
                users.iter().take(40).map(|u| format!("`{u}`")).collect::<Vec<_>>().join(" ")
            };
            ctx.say(format!("🎯 **{} alvos**: {body}", users.len())).await?;
        }
    }
    Ok(())
}

/// (admin) Adiciona nick à watchlist do sniper.
#[poise::command(prefix_command, slash_command, check = "check_admin")]
pub async fn snipe_add(
    ctx: Ctx<'_>,
    #[description = "username alvo"] nick: String,
) -> Result<(), Error> {
    let nick = nick.trim().to_lowercase();
    if nick.is_empty() || nick.len() > 32 {
        ctx.send(poise::CreateReply::default().content("uso: `/snipe_add nick:<nick>`").ephemeral(true)).await?;
        return Ok(());
    }
    let data = ctx.data();
    let mut users = read_watchlist(&data.cfg.watchlist_path);
    if users.contains(&nick) {
        ctx.say(format!("`{nick}` já está na watchlist.")).await?;
    } else {
        users.push(nick.clone());
        users.sort();
        match write_watchlist(&data.cfg.watchlist_path, &users) {
            Ok(_) => {
                ctx.say(format!("👁️ `{nick}` na mira ({} alvos).", users.len())).await?;
            }
            Err(e) => {
                ctx.say(format!("⚠️ falhou ao salvar: {e}")).await?;
            }
        }
    }
    Ok(())
}

/// (admin) Remove nick da watchlist.
#[poise::command(prefix_command, slash_command, check = "check_admin")]
pub async fn snipe_remove(
    ctx: Ctx<'_>,
    #[description = "username alvo"] nick: String,
) -> Result<(), Error> {
    let nick = nick.trim().to_lowercase();
    let data = ctx.data();
    let mut users = read_watchlist(&data.cfg.watchlist_path);
    if let Some(i) = users.iter().position(|u| u == &nick) {
        users.remove(i);
        match write_watchlist(&data.cfg.watchlist_path, &users) {
            Ok(_) => {
                ctx.say(format!("`{nick}` removido ({} alvos).", users.len())).await?;
            }
            Err(e) => {
                ctx.say(format!("⚠️ falhou ao salvar: {e}")).await?;
            }
        }
    } else {
        ctx.say(format!("`{nick}` não está na watchlist.")).await?;
    }
    Ok(())
}

/// Lista os alvos da watchlist.
#[poise::command(prefix_command, slash_command)]
pub async fn snipe_list(ctx: Ctx<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let users = read_watchlist(&data.cfg.watchlist_path);
    let body = if users.is_empty() {
        format!("(vazia — `/snipe_add nick:<nick>` ou `{}snipe add <nick>`)", data.cfg.prefix)
    } else {
        users.iter().take(40).map(|u| format!("`{u}`")).collect::<Vec<_>>().join(" ")
    };
    ctx.say(format!("🎯 **{} alvos**: {body}", users.len())).await?;
    Ok(())
}

/// (admin) Bane um usuário.
#[poise::command(prefix_command, slash_command, check = "check_root")]
pub async fn ban(
    ctx: Ctx<'_>,
    #[description = "alvo"] user: User,
    #[description = "motivo"] #[rest] reason: Option<String>,
) -> Result<(), Error> {
    let Some(gid) = ctx.guild_id() else {
        ctx.say("só em servidor.").await?;
        return Ok(());
    };
    let reason = reason.unwrap_or_default();
    let r = if reason.trim().is_empty() {
        gid.ban(&ctx.serenity_context().http, user.id, 0).await.map_err(|e| e.to_string())
    } else {
        gid.ban_with_reason(&ctx.serenity_context().http, user.id, 0, &reason)
            .await
            .map_err(|e| e.to_string())
    };
    match r {
        Ok(_) => {
            ctx.say(format!("🔨 <@{}> banido.", user.id)).await?;
        }
        Err(e) => {
            ctx.say(format!("⚠️ ban falhou: {e}")).await?;
        }
    }
    Ok(())
}

/// (admin) Expulsa um usuário.
#[poise::command(prefix_command, slash_command, check = "check_root")]
pub async fn kick(ctx: Ctx<'_>, #[description = "alvo"] user: User) -> Result<(), Error> {
    let Some(gid) = ctx.guild_id() else {
        ctx.say("só em servidor.").await?;
        return Ok(());
    };
    match gid.kick(&ctx.serenity_context().http, user.id).await {
        Ok(_) => {
            ctx.say(format!("👢 <@{}> expulso.", user.id)).await?;
        }
        Err(e) => {
            ctx.say(format!("⚠️ kick falhou: {e}")).await?;
        }
    }
    Ok(())
}

/// (admin) Silencia com timeout.
#[poise::command(prefix_command, slash_command, check = "check_root")]
pub async fn mute(
    ctx: Ctx<'_>,
    #[description = "alvo"] user: User,
    #[description = "10m, 2h, 1d (padrão 10m)"] duracao: Option<String>,
) -> Result<(), Error> {
    let Some(gid) = ctx.guild_id() else {
        ctx.say("só em servidor.").await?;
        return Ok(());
    };
    let mins = duracao.map(|d| parse_mins(&d).unwrap_or(10)).unwrap_or(10);
    let until = Timestamp::from_unix_timestamp(chrono::Utc::now().timestamp() + (mins * 60) as i64)
        .map_err(|e| format!("{e}"))?;
    match gid
        .edit_member(
            &ctx.serenity_context().http,
            user.id,
            EditMember::new().disable_communication_until(until.to_string()),
        )
        .await
    {
        Ok(_) => {
            ctx.say(format!("🔇 <@{}> mutado por {mins}min.", user.id)).await?;
        }
        Err(e) => {
            ctx.say(format!("⚠️ mute falhou: {e}")).await?;
        }
    }
    Ok(())
}

/// (admin) Remove o silêncio.
#[poise::command(prefix_command, slash_command, check = "check_root")]
pub async fn unmute(ctx: Ctx<'_>, #[description = "alvo"] user: User) -> Result<(), Error> {
    let Some(gid) = ctx.guild_id() else {
        ctx.say("só em servidor.").await?;
        return Ok(());
    };
    match gid
        .edit_member(&ctx.serenity_context().http, user.id, EditMember::new().enable_communication())
        .await
    {
        Ok(_) => {
            ctx.say(format!("🔊 <@{}> liberado.", user.id)).await?;
        }
        Err(e) => {
            ctx.say(format!("⚠️ unmute falhou: {e}")).await?;
        }
    }
    Ok(())
}

/// (admin) Apaga N mensagens, de um user/bots, ou tudo (`all`).
#[poise::command(prefix_command, slash_command, check = "check_root")]
pub async fn purge(
    ctx: Ctx<'_>,
    #[description = "N ou all (padrão 50)"] target: Option<String>,
    #[description = "só deste usuário"] user: Option<User>,
    #[description = "só bots"] bots: Option<bool>,
) -> Result<(), Error> {
    let want = match parse_purge_target(target) {
        Ok(w) => w,
        Err(usage) => {
            ctx.send(poise::CreateReply::default().content(usage).ephemeral(true)).await?;
            return Ok(());
        }
    };
    ctx.defer().await?;
    let gone = purge_channel(
        &ctx.serenity_context().http,
        ctx.channel_id(),
        want,
        user.as_ref().map(|u| u.id),
        bots.unwrap_or(false),
    )
    .await;
    let label = match (&want, &user) {
        (None, _) => format!("🧹 {gone} mensagem(ns) apagadas (all)."),
        (Some(_), Some(u)) => format!("🧹 {gone} mensagem(ns) de <@{}> apagadas.", u.id),
        _ => format!("🧹 {gone} mensagem(ns) apagadas."),
    };
    ctx.say(label).await?;
    Ok(())
}

/// (admin) Define slowmode do canal em segundos (0=off).
#[poise::command(prefix_command, slash_command, check = "check_root")]
pub async fn slowmode(
    ctx: Ctx<'_>,
    #[description = "0-21600 (padrão 5)"] segundos: Option<i64>,
) -> Result<(), Error> {
    let secs = segundos.unwrap_or(5).clamp(0, 21600) as u16;
    match ctx.channel_id().edit(&ctx.serenity_context().http, EditChannel::new().rate_limit_per_user(secs)).await {
        Ok(_) => {
            ctx.say(if secs == 0 { "🐢 slowmode off.".to_string() } else { format!("🐢 slowmode: {secs}s.") }).await?;
        }
        Err(e) => {
            ctx.say(format!("⚠️ slowmode falhou: {e}")).await?;
        }
    }
    Ok(())
}

/// (admin) Tranca o canal (@everyone sem enviar).
#[poise::command(prefix_command, slash_command, check = "check_root")]
pub async fn lock(ctx: Ctx<'_>) -> Result<(), Error> {
    let Some(gid) = ctx.guild_id() else {
        ctx.say("só em servidor.").await?;
        return Ok(());
    };
    let ow = PermissionOverwrite {
        allow: Permissions::empty(),
        deny: Permissions::SEND_MESSAGES,
        kind: PermissionOverwriteType::Role(RoleId::new(gid.get())),
    };
    match ctx.channel_id().create_permission(&ctx.serenity_context().http, ow).await {
        Ok(_) => {
            ctx.say("🔒 canal trancado.").await?;
        }
        Err(e) => {
            ctx.say(format!("⚠️ lock falhou: {e}")).await?;
        }
    }
    Ok(())
}

/// (admin) Destranca o canal.
#[poise::command(prefix_command, slash_command, check = "check_root")]
pub async fn unlock(ctx: Ctx<'_>) -> Result<(), Error> {
    let Some(gid) = ctx.guild_id() else {
        ctx.say("só em servidor.").await?;
        return Ok(());
    };
    let ow = PermissionOverwrite {
        allow: Permissions::SEND_MESSAGES,
        deny: Permissions::empty(),
        kind: PermissionOverwriteType::Role(RoleId::new(gid.get())),
    };
    match ctx.channel_id().create_permission(&ctx.serenity_context().http, ow).await {
        Ok(_) => {
            ctx.say("🔓 canal destrancado.").await?;
        }
        Err(e) => {
            ctx.say(format!("⚠️ unlock falhou: {e}")).await?;
        }
    }
    Ok(())
}

/// Ficha de um usuário (id, conta, cargos).
#[poise::command(prefix_command, slash_command)]
pub async fn userinfo(ctx: Ctx<'_>, #[description = "alvo (padrão: você)"] user: Option<User>) -> Result<(), Error> {
    let uid = user.map(|u| u.id).unwrap_or_else(|| ctx.author().id);
    let text = userinfo_text(ctx.serenity_context(), ctx.guild_id(), uid).await;
    ctx.say(text).await?;
    Ok(())
}

/// PERIGO (só prefixo, sem slash): apaga TODOS os canais e categorias.
#[poise::command(prefix_command)]
pub async fn nuke(ctx: Ctx<'_>, #[rest] confirm: Option<String>) -> Result<(), Error> {
    // Check manual (sem `check =` p/ mensagem de confirmação customizada primeiro).
    let sctx = ctx.serenity_context();
    let Some(gid) = ctx.guild_id() else { return Ok(()) };
    if !is_root(sctx, gid, ctx.author().id).await {
        ctx.say("⛔ só admin.").await?;
        return Ok(());
    }
    if confirm.as_deref() != Some("CONFIRM") {
        ctx.say("⚠️ isso apaga TUDO. Confirme com `!nuke CONFIRM`.").await?;
        return Ok(());
    }
    let channels = gid.channels(&sctx.http).await?;
    let mut n = 0;
    let mut cats = Vec::new();
    for (id, ch) in &channels {
        if ch.kind == serenity::ChannelType::Category {
            cats.push(*id);
        } else if id.delete(&sctx.http).await.is_ok() {
            n += 1;
        }
    }
    for id in cats {
        if id.delete(&sctx.http).await.is_ok() {
            n += 1;
        }
    }
    println!("⌖ nuke: {n} canais/categorias apagados na guild {gid}");
    Ok(())
}

/// Todos os comandos (registro slash + prefixo).
pub fn all_commands() -> Vec<poise::Command<BotData, Error>> {
    vec![
        ping(),
        help(),
        verify(),
        check(),
        status(),
        snipe(),
        snipe_add(),
        snipe_remove(),
        snipe_list(),
        ban(),
        kick(),
        mute(),
        unmute(),
        purge(),
        slowmode(),
        lock(),
        unlock(),
        userinfo(),
        nuke(),
    ]
}

// --- eventos (captcha) + erro (nunca silêncio) ---

async fn event_handler(
    framework: poise::FrameworkContext<'_, BotData, Error>,
    event: &serenity::FullEvent,
) -> Result<(), Error> {
    let data = framework.user_data;
    let sctx = framework.serenity_context;
    match event {
        serenity::FullEvent::Ready { data_about_bot } => {
            println!("⌖ bot online como {} (poise híbrido, prefixo '{}')", data_about_bot.user.name, data.cfg.prefix);
        }
        serenity::FullEvent::GuildMemberAddition { new_member } => {
            if data.cfg.captcha {
                issue_captcha(&sctx.http, &data.cfg, &data.pending, new_member.user.clone()).await;
            }
        }
        serenity::FullEvent::Message { new_message: msg } => {
            if msg.author.bot {
                return Ok(());
            }
            // Comando de prefixo vai p/ o poise, não p/ o captcha.
            if msg.content.trim_start().starts_with(&data.cfg.prefix) {
                return Ok(());
            }
            let in_verify = data.cfg.verify_channel_id != 0
                && msg.channel_id == ChannelId::new(data.cfg.verify_channel_id);
            let is_dm = msg.guild_id.is_none();
            if !(in_verify || is_dm) {
                return Ok(());
            }
            let mut pend = data.pending.lock().await;
            let Some(p) = pend.get_mut(&msg.author.id) else { return Ok(()) };
            if msg.content.trim().eq_ignore_ascii_case(&p.solution) {
                pend.remove(&msg.author.id);
                drop(pend);
                let gid = GuildId::new(data.cfg.guild_id);
                match verified_role_id(&sctx.http, gid, &data.cfg.verified_role).await {
                    Some(rid) => {
                        let granted = match gid.member(&sctx.http, msg.author.id).await {
                            Ok(m) => m.add_role(&sctx.http, rid).await.is_ok(),
                            Err(_) => false,
                        };
                        if granted {
                            let _ = msg.reply(&sctx.http, "✅ verificado — canais liberados.").await;
                        } else {
                            let _ = msg.reply(&sctx.http, "⚠️ acerto, mas falhei ao dar o cargo (fale com um admin).").await;
                        }
                    }
                    None => {
                        let _ = msg.reply(&sctx.http, "⚠️ cargo de verificação não existe (rode o setup).").await;
                    }
                }
            } else {
                p.attempts += 1;
                if p.attempts >= 5 {
                    pend.remove(&msg.author.id);
                    let _ = msg
                        .reply(&sctx.http, format!("❌ muitas tentativas — digite `{}verify` para novo captcha.", data.cfg.prefix))
                        .await;
                } else {
                    let _ = msg.reply(&sctx.http, format!("❌ errado ({}/5). Tente de novo.", p.attempts)).await;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

/// Erro de comando vira resposta (nunca silêncio) — substitui o `dispatch_error`.
async fn on_error(error: poise::FrameworkError<'_, BotData, Error>) {
    match &error {
        poise::FrameworkError::Command { ctx, error, .. } => {
            let _ = ctx.say(format!("⚠️ falhou: {error}")).await;
        }
        poise::FrameworkError::CommandCheckFailed { ctx, .. } => {
            let _ = ctx
                .send(poise::CreateReply::default().content("⛔ só admin.").ephemeral(true))
                .await;
        }
        poise::FrameworkError::CooldownHit { ctx, remaining_cooldown, .. } => {
            let _ = ctx
                .send(
                    poise::CreateReply::default()
                        .content(format!("⏳ em cooldown, tente em {}s.", remaining_cooldown.as_secs().max(1)))
                        .ephemeral(true),
                )
                .await;
        }
        poise::FrameworkError::ArgumentParse { ctx, error, .. } => {
            let _ = ctx
                .send(poise::CreateReply::default().content(format!("⚠️ argumento inválido: {error}")).ephemeral(true))
                .await;
        }
        _ => {
            let _ = poise::builtins::on_error(error).await;
        }
    }
}

// --- captcha pendente ouvido também fora do verify? não: igual ao legado ---

/// Sobe o gateway e fica rodando (Ctrl+C encerra).
/// Prefixo exige **Message Content Intent** no portal + no código (pedido
/// abaixo). Sem ele no portal, `!x` é ignorado e só `/x` funciona.
pub async fn run_bot(cfg: BotRunConfig) -> Result<(), String> {
    if cfg.token.is_empty() {
        return Err("sem token: defina BOT_TOKEN/DISCORD_BOT_TOKEN".to_string());
    }
    let intents = serenity::GatewayIntents::GUILDS
        | serenity::GatewayIntents::GUILD_MEMBERS
        | serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::DIRECT_MESSAGES;
    let prefix = cfg.prefix.clone();
    let guild_id = cfg.guild_id;
    // O setup é dono do `cfg` (move); o token do builder sai daqui (clone).
    let token = cfg.token.clone();
    let pending: Arc<Mutex<HashMap<UserId, Pending>>> = Arc::new(Mutex::new(HashMap::new()));
    let started = Instant::now();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: all_commands(),
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(prefix.clone().into()),
                mention_as_prefix: false,
                case_insensitive_commands: true,
                ignore_bots: true,
                ..Default::default()
            },
            event_handler: |framework, event| Box::pin(event_handler(framework, event)),
            on_error: |error| Box::pin(on_error(error)),
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                // Slash: guild (instantâneo) quando guild_id conhecido, senão global (~1h).
                if guild_id != 0 {
                    match poise::builtins::register_in_guild(
                        ctx,
                        &framework.options().commands,
                        GuildId::new(guild_id),
                    )
                    .await
                    {
                        Ok(_) => println!("⌖ slash: comandos registrados na guild {guild_id}"),
                        Err(e) => println!("⚠️ slash (guild): {e}"),
                    }
                } else {
                    match poise::builtins::register_globally(ctx, &framework.options().commands).await {
                        Ok(_) => println!("⌖ slash: comandos globais registrados"),
                        Err(e) => println!("⚠️ slash (global): {e}"),
                    }
                }
                Ok(BotData { cfg, pending, started })
            })
        })
        .build();
    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .map_err(|e| format!("client: {e}"))?;
    let _ = prefix;
    client.start().await.map_err(|e| format!("gateway: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_mins_units() {
        assert_eq!(parse_mins("10m"), Some(10));
        assert_eq!(parse_mins("2h"), Some(120));
        assert_eq!(parse_mins("1d"), Some(1440));
        assert_eq!(parse_mins("45"), Some(45));
        assert_eq!(parse_mins("xx"), None);
    }
    #[test]
    fn purge_target_shapes() {
        assert_eq!(parse_purge_target(None), Ok(Some(50)));
        assert_eq!(parse_purge_target(Some("all".into())), Ok(None));
        assert_eq!(parse_purge_target(Some("200".into())), Ok(Some(200)));
        assert!(parse_purge_target(Some("lixo".into())).is_err());
    }
    #[test]
    fn hybrids_cover_prefix_and_slash() {
        // Todo comando (menos o nuke destrutivo) é híbrido: 1 definição,
        // prefixo + slash — a classe de bug "slash ok, prefixo mudo" morre aqui.
        for cmd in all_commands() {
            if cmd.name == "nuke" {
                assert!(cmd.prefix_action.is_some());
                assert!(cmd.slash_action.is_none());
            } else if cmd.name == "snipe" {
                assert!(cmd.prefix_action.is_some());
            } else {
                assert!(cmd.prefix_action.is_some(), "{} sem prefixo", cmd.name);
                assert!(cmd.slash_action.is_some(), "{} sem slash", cmd.name);
            }
        }
    }
    #[test]
    fn help_mentions_prefix_and_slash() {
        let h = help_text("!");
        assert!(h.contains("!check"));
        assert!(h.contains("/check"));
        assert!(h.contains("/snipe_add"));
        assert!(h.contains("!purge all"));
    }
}
