pub const PUBLIC_KEY: &str = include_str!("../updater-public-key.txt");
pub const ENDPOINT: &str =
    "https://github.com/yuhui0531/localclip-windows/releases/latest/download/latest.json";
pub fn installation_enabled() -> bool {
    !PUBLIC_KEY.trim().is_empty()
}
pub fn allowed_asset(url: &str, architecture: &str) -> bool {
    let Some(rest) =
        url.strip_prefix("https://github.com/yuhui0531/localclip-windows/releases/download/")
    else {
        return false;
    };
    let parts: Vec<_> = rest.split('/').collect();
    parts.len() == 2
        && !parts[0].is_empty()
        && !rest.contains(['?', '#', '%', '\\'])
        && parts[1].ends_with(&format!("_{architecture}.msi"))
}
