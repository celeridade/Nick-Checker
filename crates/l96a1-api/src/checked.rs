//! Cache de (nick x plataforma) já checados.
//! Taken/Private/Banned usam TTL longo (não re-checar). Available nunca entra
//! aqui (livres vão para history/claim). Rate/Error/Unknown usam cache
//! transitório curto (10min): evita re-queimar o mesmo nick rateado dentro
//! do mesmo bulk/sniper sem esconder recuperação (expira sozinho).
//! TTL configurável (minutos/horas/dias) + limpeza manual. Apagar o history
//! libera re-check (o frontend chama DELETE junto). Persistido em JSON
//! (só não-livres permanentes; transitórios são só memória).

use l96a1_core::CheckResult;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

/// TTL do cache transitório (rate/error/unknown): curto, só memória.
const TRANSIENT_TTL: Duration = Duration::from_secs(600);

#[derive(Debug, Clone)]
pub struct CheckedHit {
    pub result: CheckResult,
    pub at: Instant,
    pub latency_ms: u64,
}

#[derive(Debug, Clone)]
pub struct CheckedCache {
    inner: Arc<Mutex<HashMap<(String, String), CheckedHit>>>,
    /// Transitórios (rate/error/unknown): (resultado, instante). Só memória.
    transient: Arc<Mutex<HashMap<(String, String), (CheckResult, Instant)>>>,
    ttl: Arc<Mutex<Duration>>,
    file: String,
}

fn now_unix() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn checked_path(configured: &str) -> String {
    if std::path::Path::new(configured).exists() {
        return configured.to_string();
    }
    for cand in ["/data/checked.json", "checked.json"] {
        if std::path::Path::new(cand).exists() {
            return cand.to_string();
        }
    }
    configured.to_string()
}

impl CheckedCache {
    pub fn new(ttl_secs: u64, file: String) -> Self {
        let ttl = if ttl_secs == 0 { Duration::ZERO } else { Duration::from_secs(ttl_secs) };
        let c = Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            transient: Arc::new(Mutex::new(HashMap::new())),
            ttl: Arc::new(Mutex::new(ttl)),
            file,
        };
        c.load();
        c
    }

    /// Chave normalizada: nick lowercase + platform_key.
    pub fn key(nick: &str, platform_key: &str) -> (String, String) {
        (nick.trim().to_lowercase(), platform_key.to_string())
    }

    pub async fn get(&self, nick: &str, platform_key: &str) -> Option<(CheckResult, u64)> {
        let ttl = *self.ttl.lock().await;
        let mut m = self.inner.lock().await;
        let k = Self::key(nick, platform_key);
        if let Some(hit) = m.get(&k).cloned() {
            if !ttl.is_zero() && hit.at.elapsed() > ttl {
                m.remove(&k);
            } else {
                return Some((hit.result, hit.latency_ms));
            }
        }
        drop(m);
        // Transitório curto: rate/error/unknown recentes evitam re-request
        // dentro do mesmo bulk (ex.: nick rateado não queima proxy de novo).
        let mut t = self.transient.lock().await;
        if let Some((r, at)) = t.get(&k).cloned() {
            if at.elapsed() <= TRANSIENT_TTL {
                return Some((r, 0));
            }
            t.remove(&k);
        }
        None
    }

    /// Armazena Taken/Private/Banned (TTL longo) + Rate/Error/Unknown
    /// (transitório 10min, só memória). Available nunca é cacheado.
    /// Retorna false só para Available.
    pub async fn put(&self, nick: &str, platform_key: &str, result: CheckResult, latency_ms: u64) -> bool {
        match result {
            CheckResult::Taken | CheckResult::Private | CheckResult::Banned => {
                let mut m = self.inner.lock().await;
                m.insert(Self::key(nick, platform_key), CheckedHit { result, at: Instant::now(), latency_ms });
                true
            }
            CheckResult::RateLimited | CheckResult::Error | CheckResult::Unknown => {
                let mut t = self.transient.lock().await;
                // Oportunista: expurga expirados ao inserir.
                t.retain(|_, (_, at)| at.elapsed() <= TRANSIENT_TTL);
                t.insert(Self::key(nick, platform_key), (result, Instant::now()));
                true
            }
            CheckResult::Available => false,
        }
    }

    /// Limpa tudo. Retorna quantos saíram.
    pub async fn clear(&self) -> usize {
        let mut m = self.inner.lock().await;
        let n = m.len();
        m.clear();
        drop(m);
        let mut t = self.transient.lock().await;
        let nt = t.len();
        t.clear();
        drop(t);
        self.save();
        n + nt
    }

    /// Remove todas as plataformas de 1 nick. Retorna quantos saíram.
    pub async fn clear_nick(&self, nick: &str) -> usize {
        let target = nick.trim().to_lowercase();
        let mut m = self.inner.lock().await;
        let before = m.len();
        m.retain(|(n, _), _| *n != target);
        let out = before - m.len();
        drop(m);
        let mut t = self.transient.lock().await;
        let tbefore = t.len();
        t.retain(|(n, _), _| *n != target);
        let tout = tbefore - t.len();
        drop(t);
        if out > 0 {
            self.save();
        }
        out + tout
    }

    pub async fn len(&self) -> usize {
        // Oportunista: expurga expirados na leitura de tamanho.
        let ttl = *self.ttl.lock().await;
        let mut m = self.inner.lock().await;
        if !ttl.is_zero() {
            m.retain(|_, h| h.at.elapsed() <= ttl);
        }
        let n = m.len();
        drop(m);
        let mut t = self.transient.lock().await;
        t.retain(|_, (_, at)| at.elapsed() <= TRANSIENT_TTL);
        n + t.len()
    }

    pub async fn ttl_secs(&self) -> u64 {
        self.ttl.lock().await.as_secs()
    }

    pub async fn set_ttl(&self, secs: u64) -> u64 {
        let d = if secs == 0 { Duration::ZERO } else { Duration::from_secs(secs) };
        *self.ttl.lock().await = d;
        secs
    }

    /// Persiste em JSON (chamado no fim de cada scan/job, nunca no quente).
    pub fn save(&self) {
        let path = checked_path(&self.file);
        if let Some(p) = std::path::Path::new(&path).parent() {
            if !p.as_os_str().is_empty() {
                let _ = std::fs::create_dir_all(p);
            }
        }
        // Snapshot síncrono via try_lock (nunca bloqueia o scan; se lock
        // ocupado, pula — o próximo fim de scan salva).
        let Ok(m) = self.inner.try_lock() else { return };
        let now = now_unix();
        let mut obj = serde_json::Map::new();
        for ((nick, key), h) in m.iter() {
            let age = h.at.elapsed().as_secs();
            let at = now.saturating_sub(age);
            obj.insert(
                format!("{nick}|{key}"),
                serde_json::json!({
                    "result": match h.result {
                        CheckResult::Taken => "taken",
                        CheckResult::Private => "private",
                        _ => "banned",
                    },
                    "at": at,
                    "latency": h.latency_ms,
                }),
            );
        }
        let _ = std::fs::write(&path, serde_json::to_string(&obj).unwrap_or_default());
    }

    fn load(&self) {
        let path = checked_path(&self.file);
        let Ok(text) = std::fs::read_to_string(&path) else { return };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else { return };
        let Some(obj) = v.as_object() else { return };
        let now = now_unix();
        // TTL lido depois via env; na carga usa 24h padrão se arquivo antigo.
        let mut m = self.inner.try_lock().expect("cache lock na init");
        for (k, item) in obj {
            let mut parts = k.splitn(2, '|');
            let (Some(nick), Some(key)) = (parts.next(), parts.next()) else { continue };
            let result = match item.get("result").and_then(|r| r.as_str()) {
                Some("taken") => CheckResult::Taken,
                Some("private") => CheckResult::Private,
                Some("banned") => CheckResult::Banned,
                _ => continue,
            };
            let at_unix = item.get("at").and_then(|a| a.as_u64()).unwrap_or(now);
            let age = now.saturating_sub(at_unix);
            let latency = item.get("latency").and_then(|l| l.as_u64()).unwrap_or(0);
            m.insert(
                (nick.to_string(), key.to_string()),
                CheckedHit { result, at: Instant::now() - Duration::from_secs(age), latency_ms: latency },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_path(tag: &str) -> String {
        let mut p = std::env::temp_dir();
        p.push(format!("l96a1-checked-test-{tag}-{}.json", now_unix()));
        p.to_string_lossy().to_string()
    }

    #[tokio::test]
    async fn caches_only_non_free() {
        let c = CheckedCache::new(3600, tmp_path("a"));
        assert!(c.put("nick", "github", CheckResult::Taken, 5).await);
        assert!(!c.put("nick", "x", CheckResult::Available, 5).await);
        // Rate/Error/Unknown: transitório curto (evita re-queimar no bulk).
        assert!(c.put("nick", "y", CheckResult::RateLimited, 5).await);
        assert!(c.put("nick2", "y", CheckResult::Unknown, 5).await);
        assert_eq!(c.get("NICK", "github").await.map(|(r, _)| r), Some(CheckResult::Taken));
        assert_eq!(c.get("nick", "y").await.map(|(r, _)| r), Some(CheckResult::RateLimited));
        assert_eq!(c.clear_nick("nick").await, 2);
        assert_eq!(c.get("nick2", "y").await.map(|(r, _)| r), Some(CheckResult::Unknown));
        assert_eq!(c.clear().await, 1);
        assert_eq!(c.len().await, 0);
    }

    #[tokio::test]
    async fn ttl_expires() {
        let c = CheckedCache::new(1, tmp_path("b"));
        c.put("n", "k", CheckResult::Banned, 1).await;
        tokio::time::sleep(Duration::from_secs(2)).await;
        assert_eq!(c.get("n", "k").await, None);
    }
}
