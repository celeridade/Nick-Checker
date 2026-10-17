//! Forebears (Fase 1): TSV canonico + manifest + build/stats/validate/diff.
//!
//! Layout (sem quebrar o resto do dataset):
//! ```text
//! dataset/names/forebears/<pais>-forenames.tsv
//! dataset/names/forebears/<pais>-forenames.txt   (usernames em ordem de rank)
//! dataset/names/forebears/<pais>-surnames.tsv
//! dataset/names/forebears/<pais>-surnames.txt
//! dataset/names/forebears/manifest.json
//! ```
//! Colunas do TSV: rank, gender, gender_pct, name, slug, username,
//! incidence, frequency_1x, source_url (igual ao site + username ascii).

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

pub const TSV_HEADER: &str = "rank\tgender\tgender_pct\tname\tslug\tusername\tincidence\tfrequency_1x\tsource_url";

/// Uma linha do TSV canonico.
#[derive(Debug, Clone)]
pub struct Record {
    pub rank: u32,
    pub gender: char,
    pub gender_pct: u8,
    pub name: String,
    pub slug: String,
    pub username: String,
    pub incidence: u64,
    pub frequency_1x: String,
    pub source_url: String,
}

/// Entrada do manifest.json por artefato.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub country: String,
    pub kind: String,
    pub source_url: String,
    pub tsv: String,
    pub txt: String,
    pub rows: usize,
    pub duplicates_skipped: usize,
    pub complete: bool,
    pub unique_total_site: Option<u64>,
    pub sha256_tsv: String,
    pub fetched_at: String,
    pub columns: Vec<String>,
}

/// manifest.json da pasta forebears/.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub normalization: String,
    #[serde(default)]
    pub artifacts: Vec<ManifestEntry>,
}

fn default_version() -> u32 {
    1
}

/// Normaliza nome p/ username ascii (mesma regra do importador python):
/// NFKD manual p/ latin-1, lowercase, mantem [a-z0-9].
pub fn normalize_username(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.to_lowercase().chars() {
        let base = match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' | 'å' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            'ñ' => 'n',
            'ý' | 'ÿ' => 'y',
            _ => c,
        };
        if base.is_ascii_alphanumeric() {
            out.push(base);
        }
        // espacos, hifens, apóstrofos e resto: descarta (nomes colam: "De Souza" -> "desouza")
    }
    out.chars()
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        .collect()
}

/// SHA-256 compacto (p/ atualizar manifest sem nova dependencia).
pub fn sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1,
        0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
        0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786,
        0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147,
        0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
        0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a,
        0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
        0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c,
        0x1f83d9ab, 0x5be0cd19,
    ];
    let mut msg = data.to_vec();
    let bitlen = (data.len() as u64).wrapping_mul(8);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bitlen.to_be_bytes());
    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[4 * i], chunk[4 * i + 1], chunk[4 * i + 2], chunk[4 * i + 3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }
    let mut s = String::with_capacity(64);
    for v in h {
        s.push_str(&format!("{v:08x}"));
    }
    s
}

pub fn forebears_dir(dataset_dir: &str) -> PathBuf {
    Path::new(dataset_dir).join("names").join("forebears")
}

pub fn load_manifest(dataset_dir: &str) -> Result<Manifest, String> {
    let p = forebears_dir(dataset_dir).join("manifest.json");
    let t = std::fs::read_to_string(&p).map_err(|e| format!("manifest nao encontrado ({}): {e}", p.display()))?;
    serde_json::from_str(&t).map_err(|e| format!("manifest invalido: {e}"))
}

/// Carrega um TSV canonico (pula header, tolera linhas ruins contando-as).
pub fn load_tsv(path: &Path) -> (Vec<Record>, usize) {
    let mut out = Vec::new();
    let mut bad = 0;
    let Ok(text) = std::fs::read_to_string(path) else {
        return (out, 0);
    };
    for (i, line) in text.lines().enumerate() {
        if i == 0 || line.trim().is_empty() {
            continue;
        }
        let c: Vec<&str> = line.split('\t').collect();
        if c.len() != 9 {
            bad += 1;
            continue;
        }
        out.push(Record {
            rank: c[0].parse().unwrap_or(0),
            gender: c[1].chars().next().unwrap_or('u'),
            gender_pct: c[2].parse().unwrap_or(0),
            name: c[3].to_string(),
            slug: c[4].to_string(),
            username: c[5].to_string(),
            incidence: c[6].replace(',', "").parse().unwrap_or(0),
            frequency_1x: c[7].to_string(),
            source_url: c[8].to_string(),
        });
    }
    (out, bad)
}

/// Estatisticas por artefato do manifest.
pub fn stats(dataset_dir: &str) -> String {
    let dir = forebears_dir(dataset_dir);
    let man = match load_manifest(dataset_dir) {
        Ok(m) => m,
        Err(e) => return format!("STATS FALHOU: {e}"),
    };
    let mut s = format!("FOREBEARS {} artefato(s) em {}\n", man.artifacts.len(), dir.display());
    for a in &man.artifacts {
        let tsv = dir.join(&a.tsv);
        let txt = dir.join(&a.txt);
        let (recs, bad) = load_tsv(&tsv);
        let txt_n = std::fs::read_to_string(&txt).map(|t| t.lines().filter(|l| !l.trim().is_empty()).count()).unwrap_or(0);
        let inc: u64 = recs.iter().map(|r| r.incidence).sum();
        s.push_str(&format!(
            "  {:<28} tsv={} txt={} manifest_rows={} bad={} incidencia_total={} complete={}\n",
            format!("{}/{}", a.country, a.kind),
            recs.len(),
            txt_n,
            a.rows,
            bad,
            inc,
            a.complete
        ));
    }
    s
}

/// Valida TSVs + TXT + duplicados globais. Retorna (ok, relatorio).
pub fn validate(dataset_dir: &str) -> (bool, String) {
    let dir = forebears_dir(dataset_dir);
    let mut rep = String::new();
    let mut ok = true;
    macro_rules! fail {
        ($msg:expr) => {{
            ok = false;
            rep.push_str(&format!("  FAIL {}\n", $msg));
        }};
    }
    let man = match load_manifest(dataset_dir) {
        Ok(m) => m,
        Err(e) => return (false, format!("VALIDATE FALHOU: {e}\n")),
    };
    rep.push_str(&format!("manifest v{} ({} artefatos)\n", man.version, man.artifacts.len()));
    let mut global: HashMap<String, String> = HashMap::new(); // username -> "pais/kind"
    for a in &man.artifacts {
        let tag = format!("{}/{}", a.country, a.kind);
        let tsv = dir.join(&a.tsv);
        let txt = dir.join(&a.txt);
        if !tsv.exists() {
            fail!(format!("{tag}: tsv ausente ({})", a.tsv));
            continue;
        }
        if !txt.exists() {
            fail!(format!("{tag}: txt ausente ({})", a.txt));
            continue;
        }
        let raw = std::fs::read_to_string(&tsv).unwrap_or_default();
        let first = raw.lines().next().unwrap_or_default();
        if first != TSV_HEADER {
            fail!(format!("{tag}: header divergente"));
        }
        let (recs, bad) = load_tsv(&tsv);
        if bad > 0 {
            fail!(format!("{tag}: {bad} linhas malformadas no TSV"));
        }
        if recs.len() != a.rows {
            fail!(format!("{tag}: tsv tem {} linhas, manifest diz {}", recs.len(), a.rows));
        }
        let txt_lines: Vec<String> = std::fs::read_to_string(&txt)
            .unwrap_or_default()
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();
        if txt_lines.len() != recs.len() {
            fail!(format!("{tag}: txt tem {} linhas, tsv tem {}", txt_lines.len(), recs.len()));
        }
        // ordem de popularidade: rank crescente
        let mut prev = 0u32;
        for r in &recs {
            if r.rank < prev {
                fail!(format!("{tag}: rank fora de ordem em {}", r.username));
                break;
            }
            prev = r.rank;
        }
        // username == normalize(name) e charset
        for r in &recs {
            let want = normalize_username(&r.name);
            let want = if want.is_empty() { normalize_username(&r.slug.replace('-', "")) } else { want };
            if r.username != want {
                fail!(format!("{tag}: username '{}' != normalizado '{}'", r.username, want));
                break;
            }
            if !r.username.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()) {
                fail!(format!("{tag}: charset invalido em '{}'", r.username));
                break;
            }
        }
        // txt espelha tsv na mesma ordem
        for (t, r) in txt_lines.iter().zip(recs.iter()) {
            if t != &r.username {
                fail!(format!("{tag}: txt/tsv divergem em '{t}' vs '{}'", r.username));
                break;
            }
        }
        // sha256
        let sha = sha256_hex(&raw.into_bytes());
        if sha != a.sha256_tsv {
            fail!(format!("{tag}: sha256 divergente (build desatualizado?)"));
        }
        // duplicados: intra-arquivo = FAIL; entre arquivos (ex.: Rosa e
        // nome e sobrenome) = WARN, pois o loader ja faz dedup por scan.
        let mut local: HashSet<&str> = HashSet::new();
        for r in &recs {
            if !local.insert(r.username.as_str()) {
                fail!(format!("{tag}: duplicado intra-arquivo '{}'", r.username));
                break;
            }
            if let Some(prev_tag) = global.get(&r.username) {
                if *prev_tag != tag {
                    rep.push_str(&format!("  WARN overlap '{}': {prev_tag} x {tag} (dedup no scan)\n", r.username));
                }
            } else {
                global.insert(r.username.clone(), tag.clone());
            }
        }
        rep.push_str(&format!("  ok {tag}: {} nomes\n", recs.len()));
    }
    (ok, rep)
}

/// Regenera .txt a partir dos TSVs e atualiza rows/sha no manifest.
pub fn build(dataset_dir: &str) -> Result<String, String> {
    let dir = forebears_dir(dataset_dir);
    let mp = dir.join("manifest.json");
    let mut man: Manifest = serde_json::from_str(
        &std::fs::read_to_string(&mp).map_err(|e| format!("manifest: {e}"))?,
    )
    .map_err(|e| format!("manifest invalido: {e}"))?;
    let mut out = String::new();
    for a in man.artifacts.iter_mut() {
        let tsv = dir.join(&a.tsv);
        let txt = dir.join(&a.txt);
        let (mut recs, _) = load_tsv(&tsv);
        recs.sort_by_key(|r| r.rank);
        let mut body = String::new();
        for r in &recs {
            body.push_str(&r.username);
            body.push('\n');
        }
        std::fs::write(&txt, &body).map_err(|e| format!("escrever {}: {e}", txt.display()))?;
        let raw = std::fs::read(&tsv).map_err(|e| format!("ler {}: {e}", tsv.display()))?;
        a.rows = recs.len();
        a.sha256_tsv = sha256_hex(&raw);
        out.push_str(&format!("  build {}/{}: {} nomes\n", a.country, a.kind, recs.len()));
    }
    // updated_at fica intacto: a data de coleta (fetched_at) por artefato
    // e quem importa (não o build local). Build só regenera txt + rows/sha.
    let js = serde_json::to_string_pretty(&man).map_err(|e| format!("json: {e}"))?;
    std::fs::write(&mp, js).map_err(|e| format!("manifest: {e}"))?;
    Ok(out)
}

/// Diff entre dois .txt (ou dois manifests): adicionados/removidos + amostra.
pub fn diff_files(a: &str, b: &str) -> String {
    fn lines(p: &str) -> HashSet<String> {
        // manifest.json -> conjunto "pais/kind:rows"; .txt -> usernames
        if p.ends_with("manifest.json") {
            let Ok(t) = std::fs::read_to_string(p) else { return HashSet::new() };
            let Ok(m): Result<Manifest, _> = serde_json::from_str(&t) else { return HashSet::new() };
            return m.artifacts.iter().map(|e| format!("{}/{}:{}", e.country, e.kind, e.rows)).collect();
        }
        std::fs::read_to_string(p)
            .unwrap_or_default()
            .lines()
            .map(|l| l.trim().to_lowercase())
            .filter(|l| !l.is_empty())
            .collect()
    }
    let sa = lines(a);
    let sb = lines(b);
    let mut added: Vec<_> = sb.difference(&sa).cloned().collect();
    let mut removed: Vec<_> = sa.difference(&sb).cloned().collect();
    added.sort();
    removed.sort();
    let mut s = format!("DIFF {} -> {}\n  +{} -{} ={} comum\n", a, b, added.len(), removed.len(), sa.intersection(&sb).count());
    for x in added.iter().take(20) {
        s.push_str(&format!("  + {x}\n"));
    }
    if added.len() > 20 {
        s.push_str(&format!("  + ... ({} ocultos)\n", added.len() - 20));
    }
    for x in removed.iter().take(20) {
        s.push_str(&format!("  - {x}\n"));
    }
    if removed.len() > 20 {
        s.push_str(&format!("  - ... ({} ocultos)\n", removed.len() - 20));
    }
    s
}

/// Conta nomes por arquivo forebears (p/ browser lazy futuro).
pub fn count_file(path: &str) -> usize {
    std::fs::read_to_string(path)
        .map(|t| t.lines().filter(|l| !l.trim().is_empty()).count())
        .unwrap_or(0)
}

/// Mapa pais/kind -> total (p/ TUI rapida sem carregar tudo).
pub fn index_summary(dataset_dir: &str) -> BTreeMap<String, usize> {
    let mut m = BTreeMap::new();
    if let Ok(man) = load_manifest(dataset_dir) {
        for a in man.artifacts {
            m.insert(format!("{}/{}", a.country, a.kind), a.rows);
        }
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_pt() {
        assert_eq!(normalize_username("João"), "joao");
        assert_eq!(normalize_username("José"), "jose");
        assert_eq!(normalize_username("De Souza"), "desouza");
        assert_eq!(normalize_username("Açaí-123"), "acai123");
    }

    #[test]
    fn sha256_vazio_conhecido() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
