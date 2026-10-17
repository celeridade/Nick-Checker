//! l96a1-dataset — loader + scorer de candidatos.
//! A pasta de dados e read-only; progresso fica em `results/data_progress.json`.

pub mod forebears;

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Candidato com score (maior = checar primeiro).
#[derive(Debug, Clone)]
pub struct Candidate {
    pub name: String,
    pub score: u32,
    /// raridade 0..100 (Fase 4, definição do operador: escassez = cobiça).
    pub rarity: u32,
    pub bucket: String,
}

/// Raridade 0..100 de um username.
/// Curto (1–4) e padrões de espaço combinatório pequeno (repeater, `aabb`)
/// ficam no topo; `coveted` (nome popular Forebears/Trends = cobiçado)
/// soma bônus. Exemplos raros: `abc`, `0d_`, `aabb`, `dddd`, `bbbbbbbbbb`,
/// `0000000000000000`, `vazio`, `fogo`, `gamer`.
pub fn rarity_score(name: &str, coveted: bool) -> u32 {
    let len = name.chars().count();
    let base = match len {
        0 => 0,
        1 => 100,
        2 => 95,
        3 => 90,
        4 => 80,
        5 => 60,
        6 => 40,
        _ => 20,
    };
    let lower = name.to_lowercase();
    let chars: Vec<char> = lower.chars().collect();
    let uniq: HashSet<char> = chars.iter().copied().collect();
    let pattern = if len > 0 && uniq.len() == 1 {
        15 // single repeater: espaço mínimo
    } else if (2..=3).contains(&uniq.len()) && len <= 6 {
        10 // aabb, ccaa, abab e afins
    } else if uniq.len() * 2 <= len && len <= 6 {
        5
    } else {
        0
    };
    (base + pattern + if coveted { 15 } else { 0 }).min(100)
}

/// Popularidade cobiçada: nomes Forebears (top-1k por país) + Trends.
/// `trends.tsv` é opcional: `term\tscore0-100[\tupdated]` (ignora `#`).
#[derive(Debug, Default)]
pub struct Popularity {
    pub forebears: HashSet<String>,
    pub trends: HashMap<String, u8>,
}

impl Popularity {
    /// Cobiçado = no Forebears OU trends score >= 50.
    pub fn coveted(&self, name: &str) -> bool {
        self.forebears.contains(name)
            || self.trends.get(name).map(|s| *s >= 50).unwrap_or(false)
    }
}

/// Carrega popularidade de `dataset_dir` (barato: ~8k linhas + trends).
pub fn load_popularity(dataset_dir: &str) -> Popularity {
    let mut pop = Popularity::default();
    let fb = Path::new(dataset_dir).join("names").join("forebears");
    if let Ok(rd) = std::fs::read_dir(&fb) {
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().map(|x| x == "txt").unwrap_or(false) {
                if let Ok(t) = std::fs::read_to_string(&p) {
                    pop.forebears.extend(
                        t.lines().map(|l| l.trim().to_lowercase()).filter(|l| !l.is_empty()),
                    );
                }
            }
        }
    }
    let tp = Path::new(dataset_dir).join("trends.tsv");
    if let Ok(t) = std::fs::read_to_string(&tp) {
        for line in t.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
            let mut c = line.split('\t');
            if let (Some(term), Some(score)) = (c.next(), c.next()) {
                if let Ok(s) = score.trim().parse::<u8>() {
                    pop.trends.insert(term.trim().to_lowercase(), s.min(100));
                }
            }
        }
    }
    pop
}

fn score_name(name: &str) -> (u32, &'static str) {
    let len = name.chars().count();
    let lower = name.to_lowercase();
    let chars: Vec<char> = lower.chars().collect();
    // single repeater: aaa, 000
    if !chars.is_empty() && chars.iter().all(|&c| c == chars[0]) {
        return (100, "repeater");
    }
    // OG curtos
    if len <= 3 {
        return (100, "3C");
    }
    if len == 4 {
        return (80, "4C");
    }
    // repeticao parcial (ex: aabb, abab)
    let uniq: HashSet<char> = chars.iter().copied().collect();
    if uniq.len() * 2 <= len && len <= 6 {
        return (70, "pattern");
    }
    // dicionario comum curto
    if len <= 6 {
        return (50, "short");
    }
    (10, "long")
}

/// Caminha `dir` coletando `.txt` (lazy: lista arquivos primeiro).
pub fn list_files(dir: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_string()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p.to_string_lossy().to_string());
            } else if p.extension().map(|x| x == "txt").unwrap_or(false) {
                out.push(p.to_string_lossy().to_string());
            }
        }
    }
    out.sort();
    out
}

/// Carrega candidatos de arquivos e/ou lista direta, com dedup + score.
/// `coveted` = nomes cobiçados (Forebears/Trends); vazio = sem boost.
pub fn load_candidates(
    files: &[String],
    direct: &[String],
    smart: bool,
    coveted: &dyn Fn(&str) -> bool,
) -> Vec<Candidate> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    let mut push = |raw: &str, bucket: &str| {
        let name = raw.trim().to_lowercase();
        if name.is_empty() || !seen.insert(name.clone()) {
            return;
        }
        let (score, auto_bucket) = score_name(&name);
        let rarity = rarity_score(&name, coveted(&name));
        out.push(Candidate {
            name,
            score: if smart { score.saturating_add(rarity) } else { 50 },
            rarity,
            bucket: if bucket.is_empty() { auto_bucket.to_string() } else { bucket.to_string() },
        });
    };
    for name in direct {
        push(name, "direct");
    }
    for f in files {
        let bucket = Path::new(f)
            .parent()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        if let Ok(text) = std::fs::read_to_string(f) {
            for line in text.lines() {
                push(line, &bucket);
            }
        }
    }
    if smart {
        out.sort_by(|a, b| b.score.cmp(&a.score).then(a.name.cmp(&b.name)));
    } else {
        out.sort_by(|a, b| a.name.cmp(&b.name));
    }
    out
}

/// Conta nicks por arquivo (para o browser, sem carregar tudo).
pub fn count_file(path: &str) -> usize {
    let Ok(text) = std::fs::read_to_string(path) else { return 0 };
    text.lines().filter(|l| !l.trim().is_empty()).count()
}

/// Duração humana p/ TTL: "30m", "12h", "3d", "90s", "0"/"off".
/// Retorna segundos; "off"/"0"/vazio = 0 (nunca expira).
pub fn parse_duration(s: &str) -> u64 {
    let t = s.trim().to_lowercase();
    if t.is_empty() || t == "off" || t == "0" {
        return 0;
    }
    let (num, mult) = if let Some(n) = t.strip_suffix('s') {
        (n, 1)
    } else if let Some(n) = t.strip_suffix('m') {
        (n, 60)
    } else if let Some(n) = t.strip_suffix('h') {
        (n, 3600)
    } else if let Some(n) = t.strip_suffix('d') {
        (n, 86_400)
    } else {
        (t.as_str(), 1)
    };
    num.parse::<u64>().unwrap_or(0).saturating_mul(mult)
}

/// Epoch atual (s). Usado como carimbo de cache.
pub fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Progress {
    /// chave do scan -> nome -> epoch em que foi checado (TTL configuravel).
    #[serde(default)]
    pub done: HashMap<String, HashMap<String, u64>>,
}

impl Progress {
    pub fn load(path: &str) -> Self {
        let Ok(text) = std::fs::read_to_string(path) else {
            return Self::default();
        };
        // formato novo (com timestamps)
        if let Ok(p) = serde_json::from_str::<Progress>(&text) {
            return p;
        }
        // migracao do legado {"done":{"chave":["a","b"]}} (carimba agora)
        #[derive(Deserialize)]
        struct Legacy {
            #[serde(default)]
            done: HashMap<String, Vec<String>>,
        }
        if let Ok(old) = serde_json::from_str::<Legacy>(&text) {
            let now = now_secs();
            let mut p = Progress::default();
            for (k, names) in old.done {
                for n in names {
                    p.done.entry(k.clone()).or_default().insert(n, now);
                }
            }
            return p;
        }
        Self::default()
    }
    pub fn save(&self, path: &str) {
        if let Some(parent) = Path::new(path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(t) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, t);
        }
    }
    /// Filtra candidatos ja concluidos nesta chave.
    /// `ttl_secs` 0 = nunca expira; expirados voltam como novos.
    /// Retorna (novos, pulados_frescos, expirados_reaproveitados).
    pub fn split_new(
        &self,
        key: &str,
        cands: Vec<Candidate>,
        ttl_secs: u64,
    ) -> (Vec<Candidate>, usize, usize) {
        let now = now_secs();
        let empty = HashMap::new();
        let done = self.done.get(key).unwrap_or(&empty);
        let mut skipped = 0;
        let mut expired = 0;
        let fresh: Vec<Candidate> = cands
            .into_iter()
            .filter(|c| match done.get(&c.name) {
                None => true,
                Some(ts) if ttl_secs > 0 && now.saturating_sub(*ts) >= ttl_secs => {
                    expired += 1;
                    true
                }
                _ => {
                    skipped += 1;
                    false
                }
            })
            .collect();
        (fresh, skipped, expired)
    }
    /// Marca nomes como concluidos nesta chave (carimba agora).
    pub fn mark(&mut self, key: &str, names: &[String]) {
        let now = now_secs();
        let entry = self.done.entry(key.to_string()).or_default();
        for n in names {
            entry.insert(n.clone(), now);
        }
    }
    /// Limpeza manual: remove entradas cuja chave casa com `match_key`
    /// (substring; None = todas) e, se `older_than_secs` dado, só as mais
    /// velhas que isso. Retorna (chaves_removidas, nomes_removidos).
    pub fn clear(
        &mut self,
        match_key: Option<&str>,
        older_than_secs: Option<u64>,
    ) -> (usize, usize) {
        let now = now_secs();
        let mut keys_out = 0;
        let mut names_out = 0;
        let keys: Vec<String> = self
            .done
            .keys()
            .filter(|k| match_key.map(|m| k.contains(m)).unwrap_or(true))
            .cloned()
            .collect();
        for k in keys {
            if let Some(entry) = self.done.get_mut(&k) {
                let before = entry.len();
                entry.retain(|_, ts| match older_than_secs {
                    Some(cut) => now.saturating_sub(*ts) < cut,
                    None => false,
                });
                names_out += before - entry.len();
            }
            if self.done.get(&k).map(|e| e.is_empty()).unwrap_or(false) {
                self.done.remove(&k);
                keys_out += 1;
            }
        }
        (keys_out, names_out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smart_scoring_prioritizes_short_and_repeaters() {
        let input = vec![
            "longusername123".to_string(),
            "abcd".to_string(),
            "xyz".to_string(),
            "aaaa".to_string(),
            "XYZ".to_string(), // duplicata case-insensitive
        ];
        let cands = load_candidates(&[], &input, true, &|_| false);
        let names: Vec<&str> = cands.iter().map(|c| c.name.as_str()).collect();
        // score+raridade: "xyz" (100+100) e "aaaa" (100+95) antes de
        // "abcd" (80+80) e "longusername123" (10+20); mais curto = mais raro
        assert_eq!(names, vec!["xyz", "aaaa", "abcd", "longusername123"]);
    }

    #[test]
    fn progress_resume_splits_already_checked() {
        let mut prog = Progress::default();
        prog.mark("k1", &["alpha".to_string(), "beta".to_string()]);
        let cands = load_candidates(
            &[],
            &["alpha".to_string(), "gamma".to_string(), "beta".to_string()],
            false,
            &|_| false,
        );
        let (fresh, skipped, expired) = prog.split_new("k1", cands, 0);
        assert_eq!(skipped, 2);
        assert_eq!(expired, 0);
        assert_eq!(fresh.len(), 1);
        assert_eq!(fresh[0].name, "gamma");
    }

    #[test]
    fn progress_ttl_requeues_expired() {
        let mut prog = Progress::default();
        prog.mark("k1", &["alpha".to_string()]);
        // ttl 1s: força expiração ajustando o carimbo para o passado
        if let Some(m) = prog.done.get_mut("k1") {
            m.insert("alpha".to_string(), now_secs().saturating_sub(3600));
        }
        let cands = load_candidates(&[], &["alpha".to_string()], false, &|_| false);
        let (fresh, skipped, expired) = prog.split_new("k1", cands, 60);
        assert_eq!(fresh.len(), 1);
        assert_eq!(expired, 1);
        assert_eq!(skipped, 0);
    }

    #[test]
    fn progress_clear_by_key_and_age() {
        let mut prog = Progress::default();
        prog.mark("forebears|instagram|x", &["a".to_string()]);
        prog.mark("3C|tiktok|x", &["b".to_string()]);
        let (kk, nn) = prog.clear(Some("instagram"), None);
        assert_eq!((kk, nn), (1, 1));
        assert!(prog.done.contains_key("3C|tiktok|x"));
    }

    #[test]
    fn progress_migrates_legacy_format() {
        let dir = std::env::temp_dir().join("l96a1_test_mig");
        let _ = std::fs::create_dir_all(&dir);
        let p = dir.join("prog.json");
        std::fs::write(&p, r#"{"done":{"k1":["alpha","beta"]}}"#).unwrap();
        let prog = Progress::load(p.to_str().unwrap());
        assert!(prog.done.get("k1").map(|m| m.contains_key("alpha")).unwrap_or(false));
        let cands = load_candidates(&[], &["alpha".to_string(), "zeta".to_string()], false, &|_| false);
        let (fresh, skipped, _) = prog.split_new("k1", cands, 0);
        assert_eq!(skipped, 1);
        assert_eq!(fresh.len(), 1);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn rarity_scarcity_order() {
        // curto + repeater no topo; cobiçado ganha bônus
        assert!(rarity_score("abc", false) > rarity_score("paranoico", false));
        assert_eq!(rarity_score("dddd", false), 95); // base 80 + 15 repeater
        assert_eq!(rarity_score("aabb", false), 90); // base 80 + 10 padrão
        assert!(rarity_score("maria", true) > rarity_score("maria", false));
        assert!(rarity_score("x7q2m", false) < rarity_score("aabb", false));
    }

    #[test]
    fn coveted_boost_reorders() {
        let input = vec!["zxqw".to_string(), "luiz".to_string()];
        let cands = load_candidates(&[], &input, true, &|n| n == "luiz");
        assert_eq!(cands[0].name, "luiz"); // mesmo tamanho, cobiça vence
    }

    #[test]
    fn parse_duration_units() {
        assert_eq!(parse_duration("off"), 0);
        assert_eq!(parse_duration("30m"), 1800);
        assert_eq!(parse_duration("12h"), 43200);
        assert_eq!(parse_duration("3d"), 259200);
        assert_eq!(parse_duration("90s"), 90);
    }
}
