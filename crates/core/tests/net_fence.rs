//! The fence around the one declared off-origin feature (R7): the Supabase
//! project. It follows the kit `kits/online`'s fence, without its relays.
//!
//! - `web/net/config.js` is the one file that names another origin.
//! - `web/net/client.js` is the one file that calls the database.
//! - Each page's Content-Security-Policy lists exactly that origin in
//!   `connect-src` (and none while the config is empty), `media-src` is
//!   `blob:` and no other directive names an address.
//! - The address is `https`, and the key is present when the address is.

use std::fs;
use std::path::{Path, PathBuf};

fn web() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../web")
}

fn files_under(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(read) = fs::read_dir(&d) else {
            continue;
        };
        for entry in read {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                if path.file_name().map(|n| n != "vendor").unwrap_or(true) {
                    stack.push(path);
                }
            } else {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

fn relative(path: &Path) -> String {
    path.strip_prefix(web()).unwrap().display().to_string()
}

/// The value of `name: "..."` in the config file text.
fn config_value(text: &str, name: &str) -> Option<String> {
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix(&format!("{name}:")) {
            let rest = rest.trim().trim_end_matches(',').trim();
            return rest.strip_prefix('"')?.strip_suffix('"').map(str::to_string);
        }
    }
    None
}

fn config() -> (String, String) {
    let text = fs::read_to_string(web().join("net/config.js")).expect("web/net/config.js");
    (
        config_value(&text, "url").expect("config.js has no url"),
        config_value(&text, "key").expect("config.js has no key"),
    )
}

/// The content of the first Content-Security-Policy meta element.
fn csp_of(html: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let at = lower.find("http-equiv=\"content-security-policy\"")?;
    let tag_start = lower[..at].rfind('<')?;
    let tag_end = at + lower[at..].find('>')?;
    let tag = &html[tag_start..tag_end];
    let c = tag.find("content=\"")? + "content=\"".len();
    let rest = &tag[c..];
    Some(rest[..rest.find('"')?].to_string())
}

/// The sources a directive lists, or `None` when the policy has no such directive.
fn directive<'a>(csp: &'a str, name: &str) -> Option<Vec<&'a str>> {
    csp.split(';').map(str::trim).find_map(|d| {
        let mut words = d.split_whitespace();
        (words.next() == Some(name)).then(|| words.collect())
    })
}

/// What is wrong with a policy, given the configured address (empty for none).
fn csp_problems(csp: &str, url: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut want = vec!["'self'"];
    if !url.is_empty() {
        want.push(url);
    }
    match directive(csp, "connect-src") {
        Some(mut got) => {
            got.sort();
            let mut w = want.clone();
            w.sort();
            if got != w {
                out.push(format!("connect-src is {got:?}, and config.js says {w:?}"));
            }
        }
        None => out.push("no connect-src".into()),
    }
    if directive(csp, "media-src") != Some(vec!["blob:"]) {
        out.push("media-src is not exactly blob:".into());
    }
    if directive(csp, "manifest-src") != Some(vec!["'self'"]) {
        out.push("manifest-src is not exactly 'self'".into());
    }
    for d in csp.split(';').map(str::trim) {
        let name = d.split_whitespace().next().unwrap_or("");
        if name != "connect-src" && d.contains("://") {
            out.push(format!("{name} names an address: {d}"));
        }
    }
    out
}

#[test]
fn web_net_holds_the_config_and_the_client_and_nothing_else() {
    let net: Vec<String> = files_under(&web().join("net")).iter().map(|p| relative(p)).collect();
    assert_eq!(net, vec!["net/client.js", "net/config.js"], "files in web/net/");
}

#[test]
fn only_the_config_names_an_address() {
    let mut found = Vec::new();
    for path in files_under(&web()) {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        if relative(&path) == "net/config.js" {
            continue;
        }
        for (n, line) in text.lines().enumerate() {
            // A page's policy line is checked against the config by the test below.
            if line.contains("http-equiv=\"Content-Security-Policy\"") {
                continue;
            }
            if line.contains("://") && !line.contains("http://www.w3.org/2000/svg") {
                found.push(format!("{}:{}: {}", relative(&path), n + 1, line.trim()));
            }
        }
    }
    assert!(found.is_empty(), "an address outside web/net/config.js:\n{}", found.join("\n"));
}

#[test]
fn only_the_client_reads_the_config_and_only_it_sends_requests_off_the_page() {
    let mut found = Vec::new();
    for path in files_under(&web()) {
        let name = relative(&path);
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        if !name.ends_with(".js") || name == "net/client.js" || name == "net/config.js" {
            continue;
        }
        if text.contains("config.js") {
            found.push(format!("{name} imports the config"));
        }
        // A fetch in the page names a file of its own, as a string, never a computed address.
        let mut from = 0;
        while let Some(at) = text[from..].find("fetch(") {
            let start = from + at + "fetch(".len();
            if !text[start..].starts_with('"') {
                found.push(format!("{name} calls fetch with something other than a file name"));
            }
            from = start;
        }
    }
    assert!(found.is_empty(), "fence:\n{}", found.join("\n"));
}

#[test]
fn every_page_connects_only_to_itself_and_the_configured_address() {
    let (url, _) = config();
    let mut pages = 0;
    for path in files_under(&web()) {
        if path.extension().map(|x| x == "html").unwrap_or(false) {
            pages += 1;
            let text = fs::read_to_string(&path).unwrap();
            let csp = csp_of(&text).unwrap_or_else(|| panic!("{}: no policy", relative(&path)));
            let problems = csp_problems(&csp, &url);
            assert!(problems.is_empty(), "{}: {}", relative(&path), problems.join("; "));
        }
    }
    assert!(pages > 0, "no page found");
}

#[test]
fn the_address_is_https_and_comes_with_its_key_or_both_are_empty() {
    let (url, key) = config();
    if url.is_empty() {
        assert!(key.is_empty(), "config.js has a key and no address");
    } else {
        assert!(url.starts_with("https://"), "the address must be https: {url}");
        assert!(!url.ends_with('/'), "no trailing slash on the address: {url}");
        assert!(!url.contains(' '), "the address has a space");
        assert!(!key.is_empty(), "config.js has an address and no key");
        assert!(!key.contains("service_role"), "the key must be the publishable one");
    }
}

// ---- the fence's own readers, on planted defects ----

const GOOD: &str = "default-src 'self'; connect-src 'self' https://abc.supabase.co; media-src blob:; manifest-src 'self'; img-src 'self' data:";

#[test]
fn the_policy_reader_accepts_the_configured_address_and_nothing_more() {
    assert!(csp_problems(GOOD, "https://abc.supabase.co").is_empty());
    let plain = "default-src 'self'; connect-src 'self'; media-src blob:; manifest-src 'self'";
    assert!(csp_problems(plain, "").is_empty());
}

#[test]
fn the_policy_reader_catches_an_extra_a_missing_or_a_misplaced_address() {
    let extra = GOOD.replace("https://abc.supabase.co", "https://abc.supabase.co https://evil.example");
    assert!(!csp_problems(&extra, "https://abc.supabase.co").is_empty());
    assert!(!csp_problems(GOOD, "https://other.supabase.co").is_empty());
    // An address in the config and none in the policy.
    assert!(!csp_problems("connect-src 'self'; media-src blob:; manifest-src 'self'", "https://abc.supabase.co").is_empty());
    // An address in the policy and none in the config.
    assert!(!csp_problems(GOOD, "").is_empty());
    let img = format!("{GOOD}; font-src https://fonts.example");
    assert!(!csp_problems(&img, "https://abc.supabase.co").is_empty());
    assert!(!csp_problems(&GOOD.replace("media-src blob:", "media-src blob: https://m.example"), "https://abc.supabase.co").is_empty());
    assert!(!csp_problems("default-src 'self'", "").is_empty());
}

#[test]
fn the_config_reader_reads_both_values() {
    let text = "export const config = {\n  url: \"https://x.supabase.co\",\n  key: \"abc\",\n};\n";
    assert_eq!(config_value(text, "url").as_deref(), Some("https://x.supabase.co"));
    assert_eq!(config_value(text, "key").as_deref(), Some("abc"));
    assert_eq!(config_value(text, "other"), None);
}
