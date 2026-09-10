use crate::models::{Ecosystem, ProviderInfo};
use crate::providers::paper::PaperFamilyProvider;

pub fn info() -> ProviderInfo {
    ProviderInfo {
        id: "folia".into(),
        name: "Folia".into(),
        description: "Multithreaded Paper fork designed for large player counts.".into(),
        recommended_use: "High-population servers that need regionized multithreading.".into(),
        ecosystem: Ecosystem::Bukkit,
        supports_plugins: true,
        supports_mods: false,
        performance: "Excellent at scale".into(),
    }
}

pub struct FoliaProvider;

impl FoliaProvider {
    pub fn new(client: reqwest::Client) -> PaperFamilyProvider {
        PaperFamilyProvider::folia(client, info())
    }
}
