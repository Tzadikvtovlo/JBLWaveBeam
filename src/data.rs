//! Runtime data (`devices/`, `locales/`): the first folder that has it, from the program's
//! folder upward — next to the exe in a release, the project root in a dev build.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Single-file build: when no `devices/` folder is found next to the program, these copies, which
/// are compiled into it, are used (the generic JBL profile and the Wave Beam; no photos).
pub const EMBEDDED_PROFILES: [&str; 2] =
    [include_str!("../devices/jbl-generic/profile.toml"), include_str!("../devices/jbl-wave-beam/profile.toml")];
pub const EMBEDDED_LOCALES: [(&str, &str); 2] =
    [("en", include_str!("../locales/en.toml")), ("pt-BR", include_str!("../locales/pt-BR.toml"))];

pub fn root() -> Option<&'static Path> {
    static ROOT: OnceLock<Option<PathBuf>> = OnceLock::new();
    ROOT.get_or_init(|| {
        let exe = std::env::current_exe().ok()?;
        exe.ancestors().skip(1).find(|d| d.join("devices").is_dir()).map(Path::to_path_buf)
    })
    .as_deref()
}
