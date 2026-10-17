//! Camada de plataformas.
//!
//! - [`native`] — checkers por site em `platforms/<categoria>/`: 29 hand-written
//!   + ~1176 gerados de manifest/WMN (ver `tools/gen_sites.py`).
//! - [`generated`] — registro dos gerados (`all_generated()`).
//! - [`site_probe`] — probe compartilhado dos gerados (lógica ex-motores Tier-2).
//!
//! Os módulos das 29 nativas são re-exportados no nível do crate
//! (`l96a1_checkers::<módulo>`) para preservar a API pública existente.

pub mod generated;
pub mod native;
pub mod site_probe;
