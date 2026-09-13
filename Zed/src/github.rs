use zed_extension_api::{self as zed};

use crate::GITHUB_REPO;

const BUNDLE_PREFIX: &str = "DotRush.Bundle";

/// Downloads `DotRush.Bundle.<module>.zip` from the latest DotRush GitHub release
/// and extracts it into `dest_dir` (relative to the extension's working directory).
///
/// Bundles are universal (framework-dependent) builds, so there is a single asset
/// per module and no platform suffix.
pub(crate) fn download_bundle(module: &str, dest_dir: &str) -> Result<(), String> {
    let asset_name = format!("{BUNDLE_PREFIX}.{module}.zip");

    let zed::GithubRelease { version, assets } = zed::latest_github_release(
        GITHUB_REPO,
        zed::GithubReleaseOptions {
            require_assets: true,
            pre_release: false,
        },
    )?;

    let asset = assets
        .into_iter()
        .find(|asset| asset.name == asset_name)
        .ok_or_else(|| format!("Asset '{asset_name}' not found in DotRush release {version}"))?;

    zed::download_file(&asset.download_url, dest_dir, zed::DownloadedFileType::Zip)
        .map_err(|e| format!("Failed to download '{asset_name}': {e}"))
}
