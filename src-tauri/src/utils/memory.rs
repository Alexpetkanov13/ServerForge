pub fn clamp_memory_mb(requested: u32, total_system_mb: u64) -> u32 {
    let usable = (total_system_mb as u32).saturating_sub(2048).max(1024);
    requested.clamp(512, usable.max(512))
}

pub fn memory_warning(allocated_mb: u32, total_system_mb: u64) -> Option<&'static str> {
    if total_system_mb == 0 {
        return None;
    }
    let ratio = allocated_mb as f64 / total_system_mb as f64;
    if ratio > 0.75 {
        Some("This allocation leaves very little memory for the operating system.")
    } else if allocated_mb < 1024 {
        Some("Less than 1 GB may be unstable for modern Minecraft versions.")
    } else {
        None
    }
}

pub fn preset_mb(preset: &str) -> u32 {
    match preset {
        "light" => 2048,
        "standard" => 4096,
        "large" => 6144,
        "heavy" => 8192,
        _ => 4096,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_to_available_ram() {
        assert_eq!(clamp_memory_mb(32000, 8192), 6144);
        assert!(clamp_memory_mb(4096, 32768) >= 4096);
    }

    #[test]
    fn warns_on_excessive_allocation() {
        assert!(memory_warning(28000, 32000).is_some());
        assert!(memory_warning(4096, 32000).is_none());
    }
}
