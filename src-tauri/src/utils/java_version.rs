pub fn recommended_java_major(minecraft_version: &str) -> u32 {
    let (major, minor, patch) = parse_mc(minecraft_version);
    match (major, minor, patch) {
        (1, m, p) if m > 20 || (m == 20 && p >= 5) => 21,
        (1, m, _) if m >= 18 => 17,
        (1, 17, _) => 17,
        (1, m, _) if m >= 12 => 8,
        (v, _, _) if v >= 26 || v > 1 => 21,
        _ => 8,
    }
}

pub fn parse_mc(version: &str) -> (u32, u32, u32) {
    let cleaned = version.trim().trim_start_matches('v');
    let mut parts = cleaned.split(|c: char| c == '.' || c == '-');
    let major = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    (major, minor, patch)
}

pub fn is_release_version(version: &str) -> bool {
    let lower = version.to_ascii_lowercase();
    !(lower.contains("snapshot")
        || lower.contains("pre")
        || lower.contains("rc")
        || lower.contains("alpha")
        || lower.contains("beta")
        || version.contains('w'))
}

pub fn channel_for_version(version: &str) -> &'static str {
    let lower = version.to_ascii_lowercase();
    if lower.contains("snapshot") || version.contains('w') {
        "snapshot"
    } else if lower.contains("pre") || lower.contains("rc") {
        "preview"
    } else {
        let (major, minor, _) = parse_mc(version);
        if major == 1 && minor < 16 {
            "legacy"
        } else {
            "stable"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn java_recommendations() {
        assert_eq!(recommended_java_major("1.21.8"), 21);
        assert_eq!(recommended_java_major("1.20.4"), 17);
        assert_eq!(recommended_java_major("1.16.5"), 8);
        assert_eq!(recommended_java_major("1.12.2"), 8);
    }

    #[test]
    fn parses_versions() {
        assert_eq!(parse_mc("1.21.10"), (1, 21, 10));
        assert_eq!(parse_mc("1.20"), (1, 20, 0));
    }
}
