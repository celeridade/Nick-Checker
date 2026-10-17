//! Isolamento por key: `discord` roda SÓ a nativa, `mf:discord` SÓ a mf.
//! (Regra de produto: filtro exato, sem dedupe entre hosts no scan.)

use l96a1_core::Platform;

#[test]
fn discord_filters_are_exact() {
    let nat: Vec<String> = l96a1_checkers::resolve("discord").iter().map(|p| p.key().to_string()).collect();
    assert_eq!(nat, vec!["discord".to_string()]);

    let mf: Vec<String> =
        l96a1_checkers::resolve("mf:discord").iter().map(|p| p.key().to_string()).collect();
    assert_eq!(mf, vec!["mf:discord".to_string()]);
}

#[test]
fn groups_cover_new_cats() {
    assert!(l96a1_checkers::GROUPS.contains(&"tech"));
    assert!(l96a1_checkers::GROUPS.contains(&"xx NSFW xx"));
    assert!(!l96a1_checkers::GROUPS.contains(&"manifest"));
    assert!(!l96a1_checkers::GROUPS.contains(&"wmn"));
}
