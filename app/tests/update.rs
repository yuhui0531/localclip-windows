use localclip_app::update_policy::{allowed_asset, installation_enabled};
#[test]
fn manual_build_cannot_install_and_assets_are_repo_and_architecture_bound() {
    assert!(!installation_enabled());
    assert!(allowed_asset(
        "https://github.com/yuhui0531/localclip-windows/releases/download/v1/LocalClip_1_x64.msi",
        "x64"
    ));
    for url in [
        "http://github.com/yuhui0531/localclip-windows/releases/download/v1/LocalClip_1_x64.msi",
        "https://github.com/other/project/releases/download/v1/LocalClip_1_x64.msi",
        "https://github.com/yuhui0531/localclip-windows/releases/download/v1/LocalClip_1_arm64.msi",
        "https://example.com/LocalClip_1_x64.msi",
    ] {
        assert!(!allowed_asset(url, "x64"));
    }
}
