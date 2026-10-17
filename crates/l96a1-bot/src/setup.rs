//! Setup automático do servidor (Fase 3 v2): exatamente 4 cargos +
//! categorias por grupo, 1 canal texto por plataforma, 1 voz por grupo,
//! #verify + #hits + #log, e 1 webhook por canal de plataforma.
//!
//! Cargos: `owner` (admin), `amigos` (manual p/ amigos + granted pelo
//! captcha), `og` (manual p/ contas com username raro) e o role do próprio
//! bot (managed, já existe). Retorna o mapa plataforma → webhook p/ o CLI
//! salvar em `[snipe.channels]` (+ webhook do #hits como principal).

use serenity::all::*;
use std::collections::HashMap;

pub struct RoleScheme {
    pub owner: String,
    pub member: String,
    pub og: String,
}

pub struct SetupResult {
    /// plataforma (chave, ex: "youtube") -> webhook url do canal
    pub channels: HashMap<String, String>,
    /// webhook do canal #hits (vira `webhook.url` principal)
    pub hits_url: String,
    /// id do canal #verify (vira `bot.verify_channel_id`)
    pub verify_channel_id: u64,
    pub log: Vec<String>,
}

pub fn chan_name(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

/// Overwrites padrão de canal privado: @everyone sem ver; roles com acesso.
fn private_overwrites(
    everyone: RoleId,
    allowed: &[RoleId],
) -> Vec<PermissionOverwrite> {
    let mut v = vec![PermissionOverwrite {
        allow: Permissions::empty(),
        deny: Permissions::VIEW_CHANNEL,
        kind: PermissionOverwriteType::Role(everyone),
    }];
    for r in allowed {
        v.push(PermissionOverwrite {
            allow: Permissions::VIEW_CHANNEL
                | Permissions::SEND_MESSAGES
                | Permissions::READ_MESSAGE_HISTORY
                | Permissions::CONNECT
                | Permissions::SPEAK,
            deny: Permissions::empty(),
            kind: PermissionOverwriteType::Role(*r),
        });
    }
    v
}

async fn get_or_create_role(
    http: &Http,
    guild: GuildId,
    name: &str,
    admin: bool,
) -> Result<RoleId, String> {
    let roles = guild.roles(http).await.map_err(|e| format!("roles: {e}"))?;
    if let Some((id, _)) = roles.iter().find(|(_, r)| r.name == name) {
        return Ok(*id);
    }
    let mut b = EditRole::new().name(name);
    if admin {
        b = b.permissions(Permissions::ADMINISTRATOR);
    }
    let role = guild.create_role(http, b).await.map_err(|e| format!("create role {name}: {e}"))?;
    Ok(role.id)
}

async fn get_or_create_category(
    http: &Http,
    guild: GuildId,
    name: &str,
    overwrites: Vec<PermissionOverwrite>,
) -> Result<ChannelId, String> {
    let channels = guild.channels(http).await.map_err(|e| format!("channels: {e}"))?;
    if let Some((id, _)) = channels
        .iter()
        .find(|(_, c)| c.name == name && c.kind == ChannelType::Category)
    {
        return Ok(*id);
    }
    let c = guild
        .create_channel(http, CreateChannel::new(name).kind(ChannelType::Category).permissions(overwrites))
        .await
        .map_err(|e| format!("category {name}: {e}"))?;
    Ok(c.id)
}

async fn get_or_create_channel(
    http: &Http,
    guild: GuildId,
    name: &str,
    kind: ChannelType,
    category: ChannelId,
    overwrites: Vec<PermissionOverwrite>,
) -> Result<ChannelId, String> {
    let channels = guild.channels(http).await.map_err(|e| format!("channels: {e}"))?;
    if let Some((id, _)) = channels.iter().find(|(_, c)| {
        c.name == name && c.kind == kind && c.parent_id == Some(category)
    }) {
        return Ok(*id);
    }
    let c = guild
        .create_channel(
            http,
            CreateChannel::new(name).kind(kind).category(category).permissions(overwrites),
        )
        .await
        .map_err(|e| format!("channel {name}: {e}"))?;
    Ok(c.id)
}

/// Reusa webhook existente com o nome (setup idempotente de verdade) ou cria.
async fn get_or_create_webhook(http: &Http, channel: ChannelId, name: &str) -> Result<String, String> {
    let existing = channel.webhooks(http).await.map_err(|e| format!("list webhooks: {e}"))?;
    if let Some(wh) = existing.iter().find(|w| w.name.as_deref() == Some(name)) {
        if let Ok(url) = wh.url() {
            return Ok(url);
        }
    }
    channel
        .create_webhook(http, CreateWebhook::new(name))
        .await
        .map_err(|e| format!("create webhook: {e}"))?
        .url()
        .map_err(|e| format!("webhook url: {e}"))
}

/// Configura o servidor inteiro. `groups` = (grupo, [plataformas]).
pub async fn setup_guild(
    token: &str,
    guild_id: u64,
    groups: &[(String, Vec<String>)],
    roles: &RoleScheme,
) -> Result<SetupResult, String> {
    let http = Http::new(token);
    let guild = GuildId::new(guild_id);
    guild
        .to_partial_guild(&http)
        .await
        .map_err(|e| format!("guild {guild_id} inacessível (bot convidado? token certo?): {e}"))?;
    let mut log = Vec::new();
    let everyone = RoleId::new(guild_id); // @everyone tem o id da guild

    // 4 cargos (o role do bot já existe como managed)
    let owner = get_or_create_role(&http, guild, &roles.owner, true).await?;
    let member = get_or_create_role(&http, guild, &roles.member, false).await?;
    let og = get_or_create_role(&http, guild, &roles.og, false).await?;
    log.push(format!(
        "cargos: '{}' (admin), '{}' (manual+captcha), '{}' (manual) ok",
        roles.owner, roles.member, roles.og
    ));
    let audience = vec![owner, member, og];

    let mut channels = HashMap::new();
    for (group, plats) in groups {
        let cat = get_or_create_category(
            &http,
            guild,
            &format!("⌖ L96A1 // {}", group.to_uppercase()),
            private_overwrites(everyone, &audience),
        )
        .await?;
        for p in plats {
            let ch = get_or_create_channel(
                &http,
                guild,
                &chan_name(p),
                ChannelType::Text,
                cat,
                private_overwrites(everyone, &audience),
            )
            .await?;
            let url = get_or_create_webhook(&http, ch, "l96a1")
                .await
                .map_err(|e| format!("webhook {p}: {e}"))?;
            channels.insert(p.clone(), url);
        }
        // 1 voz por grupo (lounge)
        get_or_create_channel(
            &http,
            guild,
            &format!("voz-{}", chan_name(group)),
            ChannelType::Voice,
            cat,
            private_overwrites(everyone, &audience),
        )
        .await?;
        log.push(format!("categoria {group}: {} texto + 1 voz", plats.len()));
    }

    // sistema: #verify aberto; #hits (audiência) + #log (só owner)
    let sys = get_or_create_category(&http, guild, "⌖ L96A1 // SISTEMA", vec![]).await?;
    let channels_all = guild.channels(&http).await.map_err(|e| format!("channels: {e}"))?;
    let find = |n: &str| {
        channels_all
            .iter()
            .find(|(_, c)| c.name == n && c.kind == ChannelType::Text && c.parent_id == Some(sys))
            .map(|(id, _)| *id)
    };
    let verify = match find("verify") {
        Some(id) => id,
        None => {
            guild
                .create_channel(
                    &http,
                    CreateChannel::new("verify").kind(ChannelType::Text).category(sys),
                )
                .await
                .map_err(|e| format!("channel verify: {e}"))?
                .id
        }
    };
    let hits = match find("hits") {
        Some(id) => id,
        None => {
            guild
                .create_channel(
                    &http,
                    CreateChannel::new("hits")
                        .kind(ChannelType::Text)
                        .category(sys)
                        .permissions(private_overwrites(everyone, &audience)),
                )
                .await
                .map_err(|e| format!("channel hits: {e}"))?
                .id
        }
    };
    match find("log") {
        Some(_) => {}
        None => {
            guild
                .create_channel(
                    &http,
                    CreateChannel::new("log")
                        .kind(ChannelType::Text)
                        .category(sys)
                        .permissions(private_overwrites(everyone, &[owner])),
                )
                .await
                .map_err(|e| format!("channel log: {e}"))?;
        }
    };
    let hits_url = get_or_create_webhook(&http, hits, "l96a1-hits")
        .await
        .map_err(|e| format!("webhook hits: {e}"))?;
    log.push("sistema: #verify (aberto), #hits, #log ok".to_string());

    Ok(SetupResult {
        channels,
        hits_url,
        verify_channel_id: verify.get(),
        log,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chan_name_sanitizes() {
        assert_eq!(chan_name("guns_lol"), "guns-lol");
        assert_eq!(chan_name("YouTube"), "youtube");
        assert_eq!(chan_name(" voz-social "), "voz-social");
    }
}
