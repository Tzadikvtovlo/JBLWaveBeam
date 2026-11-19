//! Texts the user sees (UI and error messages) from `locales/<language>.toml`, picked by the
//! system language or `--lang`. A key missing there falls back to English, then to the key itself.

use std::collections::HashMap;
use std::fmt::Display;
use std::sync::OnceLock;

static TEXTS: OnceLock<HashMap<String, String>> = OnceLock::new();

pub fn init(forced: Option<&str>) {
    let wanted = forced.map(str::to_string).or_else(sys_locale::get_locale).unwrap_or_default();
    let mut texts = load("en");
    if let Some(file) = pick(&wanted) {
        texts.extend(load(&file));
    }
    let _ = TEXTS.set(texts);
}

pub fn t(key: &str) -> String {
    TEXTS.get().and_then(|m| m.get(key)).cloned().unwrap_or_else(|| key.to_string())
}

/// `t`, or nothing when the key is not in any language (optional texts, like hints).
pub fn maybe(key: &str) -> Option<String> {
    TEXTS.get().and_then(|m| m.get(key)).cloned()
}

/// `t` with its `{name}` placeholders filled in.
pub fn tf(key: &str, args: &[(&str, &dyn Display)]) -> String {
    args.iter().fold(t(key), |s, (name, value)| s.replace(&format!("{{{name}}}"), &value.to_string()))
}

/// `pt-BR` → pt-BR.toml; otherwise any file of the same language (pt.toml, pt-PT.toml).
fn pick(wanted: &str) -> Option<String> {
    let files: Vec<String> = match crate::data::root() {
        Some(root) => std::fs::read_dir(root.join("locales"))
            .ok()?
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "toml"))
            .filter_map(|p| Some(p.file_stem()?.to_str()?.to_string()))
            .collect(),
        // single-file build: the languages compiled into the program
        None => crate::data::EMBEDDED_LOCALES.iter().map(|(n, _)| n.to_string()).collect(),
    };
    // Linux hands out "pt_BR.UTF-8"
    let wanted = wanted.split('.').next().unwrap_or_default().replace('_', "-");
    let language = |s: &str| s.split('-').next().unwrap_or_default().to_ascii_lowercase();
    files
        .iter()
        .find(|f| f.eq_ignore_ascii_case(&wanted))
        .or_else(|| files.iter().find(|f| language(f) == language(&wanted)))
        .cloned()
}

/// Reads a locale file into flat keys: `[list] title` → `list.title`.
fn load(name: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let Some(root) = crate::data::root() else {
        // single-file build: the language compiled into the program
        if let Some((_, text)) = crate::data::EMBEDDED_LOCALES.iter().find(|(n, _)| *n == name) {
            if let Ok(table) = toml::from_str::<toml::Table>(text) {
                flatten("", &table, &mut out);
            }
        }
        return out;
    };
    let path = root.join("locales").join(format!("{name}.toml"));
    match std::fs::read_to_string(&path).map(|s| toml::from_str::<toml::Table>(&s)) {
        Ok(Ok(table)) => flatten("", &table, &mut out),
        Ok(Err(e)) => eprintln!("{}: {e}", path.display()),
        Err(_) => {}
    }
    out
}

fn flatten(prefix: &str, table: &toml::Table, out: &mut HashMap<String, String>) {
    for (k, v) in table {
        let key = if prefix.is_empty() { k.clone() } else { format!("{prefix}.{k}") };
        match v {
            toml::Value::String(s) => {
                out.insert(key, s.clone());
            }
            toml::Value::Table(t) => flatten(&key, t, out),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_the_language_and_falls_back() {
        assert_eq!(pick("pt_BR.UTF-8").as_deref(), Some("pt-BR"));
        assert_eq!(pick("pt-PT").as_deref(), Some("pt-BR"));
        assert_eq!(pick("en-US").as_deref(), Some("en"));
        assert_eq!(pick("ja-JP"), None);
        init(Some("pt-BR"));
        assert_eq!(t("list.title"), "Fones");
        assert_eq!(tf("list.many_connected", &[("n", &3)]), "3 conectados");
        assert_eq!(t("no.such.key"), "no.such.key");
    }

    #[test]
    fn every_language_has_every_key() {
        let keys = |name| {
            let mut k: Vec<String> = load(name).into_keys().collect();
            k.sort();
            k
        };
        assert_eq!(keys("pt-BR"), keys("en"));
    }
}
