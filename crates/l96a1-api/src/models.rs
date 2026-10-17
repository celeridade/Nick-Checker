//! DTOs compatíveis com `website/src/lib/results.ts`.
//! O frontend espera exatamente estes nomes de campo no SSE `result`.
#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub platform: String,
    pub platformUrl: String,
    pub username: String,
    #[serde(default)]
    pub displayName: Option<String>,
    #[serde(default)]
    pub pfpUrl: Option<String>,
    #[serde(default)]
    pub bannerUrl: Option<String>,
    #[serde(default)]
    pub userId: Option<String>,
    #[serde(default)]
    pub bio: Option<String>,
    #[serde(default)]
    pub followers: Option<i64>,
    #[serde(default)]
    pub following: Option<i64>,
    #[serde(default)]
    pub posts: Option<i64>,
    #[serde(default)]
    pub likes: Option<i64>,
    #[serde(default)]
    pub isVerified: Option<bool>,
    #[serde(default)]
    pub emails: Vec<String>,
    #[serde(default)]
    pub website: Option<String>,
    #[serde(default)]
    pub createdAt: Option<String>,
    #[serde(default)]
    pub lastActive: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub isPrivate: bool,
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub extras: HashMap<String, String>,
    pub profileUrl: String,
    #[serde(default)]
    pub sourceQuery: String,
    #[serde(default)]
    pub duration: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub hasChecker: bool,
    #[serde(default)]
    pub displayOrder: usize,
    /// Veio do cache de já-checados (sem rede nesta passada).
    #[serde(default)]
    pub cached: bool,
}

/// Aviso por (nick x plataforma): rate/error/unknown com proxy mascarado.
/// Espelha o ClaimBox dos livres, mas para diagnóstico (Fase 4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarnInfo {
    pub nick: String,
    pub platform: String,
    pub key: String,
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub evidence: String,
    #[serde(default)]
    pub latency_ms: u64,
    #[serde(default)]
    pub proxy: String,
    #[serde(default)]
    pub attempt: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub key: String,
    pub name: String,
    pub group: String,
    pub hasChecker: bool,
    pub max_concurrent: usize,
    pub route: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Queued,
    Running,
    Done,
    Cancelled,
    Error,
}

/// Nick LIVRE (Available) — o que vai para os resultados/download.
/// Lógica sniper: só disponíveis entram; hits (taken) ficam no stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeNick {
    pub nick: String,
    pub platform: String,
    #[serde(default)]
    pub profileUrl: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanJob {
    pub id: String,
    pub status: JobStatus,
    pub source: String,
    pub usernames: Vec<String>,
    pub platforms: String,
    pub total: usize,
    pub checked: usize,
    pub hits: usize,
    pub created_at: String,
    #[serde(default)]
    pub results: Vec<SearchResult>,
    /// Disponíveis do job (para download txt/md/csv/pdf).
    #[serde(default)]
    pub available: Vec<FreeNick>,
    #[serde(default)]
    pub error: Option<String>,
    /// Diagnóstico por plataforma (rate/error/unknown, cap 500).
    #[serde(default)]
    pub warnings: Vec<WarnInfo>,
}

/// Uma linha de histórico por scan (bulk/dataset compactam em 1 linha).
/// `available` = só disponíveis (nicks livres ou plataformas livres).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub query: String,
    pub queryType: String,
    pub resultCount: usize,
    pub status: String,
    pub time: String,
    pub created_at: String,
    #[serde(default)]
    pub available: Vec<String>,
}
