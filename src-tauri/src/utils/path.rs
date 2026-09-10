use crate::error::AppError;

const INVALID_CHARS: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
const RESERVED: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

pub fn validate_server_name(name: &str) -> Result<(), AppError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::new(
            "Invalid server name",
            "Please enter a name for this server.",
        ));
    }
    if trimmed.chars().any(|c| INVALID_CHARS.contains(&c) || c.is_control()) {
        return Err(AppError::new(
            "Invalid server name",
            "The name contains characters that cannot be used in a folder name.",
        )
        .with_causes(vec![r#"Avoid < > : " / \ | ? *"#]));
    }
    if RESERVED
        .iter()
        .any(|reserved| trimmed.eq_ignore_ascii_case(reserved))
    {
        return Err(AppError::new(
            "Invalid server name",
            "That name is reserved by the operating system.",
        ));
    }
    if trimmed.len() > 80 {
        return Err(AppError::new(
            "Invalid server name",
            "Please use a shorter server name.",
        ));
    }
    Ok(())
}

pub fn folder_name_from_server(name: &str) -> String {
    name.trim()
        .chars()
        .map(|c| if INVALID_CHARS.contains(&c) { '-' } else { c })
        .collect::<String>()
        .trim()
        .to_string()
}

pub fn share_file_stem(name: &str) -> String {
    let stem = folder_name_from_server(name)
        .split_whitespace()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
        .to_lowercase();
    if stem.is_empty() {
        "server".into()
    } else {
        stem
    }
}

pub fn share_file_name(name: &str) -> String {
    format!("{}.server", share_file_stem(name))
}

pub fn share_pack_path(server_path: &std::path::Path, name: &str) -> std::path::PathBuf {
    server_path.join(share_file_name(name))
}

pub fn looks_like_server_dir(path: &std::path::Path) -> bool {
    if !path.is_dir() {
        return false;
    }
    let markers = [
        "server.properties",
        "eula.txt",
        "paper.yml",
        "spigot.yml",
        "purpur.yml",
        "fabric-server-launch.jar",
    ];
    if markers.iter().any(|m| path.join(m).exists()) {
        return true;
    }
    std::fs::read_dir(path)
        .ok()
        .map(|rd| {
            rd.flatten().any(|e| {
                e.file_name()
                    .to_string_lossy()
                    .to_ascii_lowercase()
                    .ends_with(".jar")
            })
        })
        .unwrap_or(false)
}

pub fn is_path_inside(parent: &std::path::Path, child: &std::path::Path) -> bool {
    let Ok(parent) = parent.canonicalize() else {
        return false;
    };
    let Ok(child) = child.canonicalize() else {
        return false;
    };
    child.starts_with(parent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_and_illegal_names() {
        assert!(validate_server_name("").is_err());
        assert!(validate_server_name("bad:name").is_err());
        assert!(validate_server_name("CON").is_err());
        assert!(validate_server_name("Survival SMP").is_ok());
    }

    #[test]
    fn sanitizes_folder_names() {
        assert_eq!(folder_name_from_server("My/Server"), "My-Server");
    }

    #[test]
    fn share_file_uses_kebab_case() {
        assert_eq!(share_file_name("Testing Server"), "testing-server.server");
        assert_eq!(share_file_name("Survival SMP"), "survival-smp.server");
        let dir = std::path::Path::new("servers").join("testing-server");
        assert_eq!(
            share_pack_path(&dir, "Testing Server"),
            dir.join("testing-server.server")
        );
    }
}
