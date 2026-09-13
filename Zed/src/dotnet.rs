use std::collections::HashMap;
use zed_extension_api::{self as zed, Worktree};

/// Locates the `dotnet` host as an absolute path.
///
/// Zed resolves a relative command against the extension's working directory,
/// so a bare `dotnet` would never be found. Prefer the user's `PATH`, then fall
/// back to `DOTNET_ROOT` from the login shell environment.
pub(crate) fn find(worktree: &Worktree) -> Result<String, String> {
    if let Some(path) = worktree.which("dotnet") {
        return Ok(path);
    }

    let env: HashMap<String, String> = worktree.shell_env().into_iter().collect();
    if let Some(root) = env.get("DOTNET_ROOT").filter(|root| !root.is_empty()) {
        let (platform, _) = zed::current_platform();
        let executable = match platform {
            zed::Os::Windows => "dotnet.exe",
            _ => "dotnet",
        };
        return Ok(format!("{}/{}", root.trim_end_matches(['/', '\\']), executable));
    }

    Err(
        "Cannot find 'dotnet'. Install the .NET SDK 10 or higher and make sure 'dotnet' is available on PATH (or set DOTNET_ROOT)."
            .to_string(),
    )
}
