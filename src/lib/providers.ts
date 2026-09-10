const PLUGIN_PROVIDERS = new Set(["paper", "spigot", "purpur", "folia"]);
const MOD_PROVIDERS = new Set(["fabric", "forge", "neoforge"]);

export function supportsPlugins(provider?: string | null) {
  return PLUGIN_PROVIDERS.has((provider ?? "").toLowerCase());
}

export function supportsMods(provider?: string | null) {
  return MOD_PROVIDERS.has((provider ?? "").toLowerCase());
}

export function providerLabel(provider?: string | null) {
  const id = (provider ?? "").toLowerCase();
  return id ? id.charAt(0).toUpperCase() + id.slice(1) : "Server";
}
