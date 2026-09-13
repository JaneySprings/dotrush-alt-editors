// ref: https://github.com/qwadrox/zed-netcoredbg

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zed_extension_api::{
    self as zed, serde_json, DebugAdapterBinary, DebugConfig, DebugRequest, DebugScenario,
    DebugTaskDefinition, StartDebuggingRequestArguments, Worktree,
};

use crate::{utils, ROOT_DIR};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct NetCoreDebugConfig {
    pub request: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub program: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_at_entry: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub just_my_code: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_step_filtering: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_id: Option<super::ProcessId>,
}

const REPO_SOURCE: &str = "JaneySprings/clrdbg";
const MODULE_DIR: &str = "Debugger";
const BINARY_NAME: &str = "clrdbg";

pub fn get_dap_binary(
    config: DebugTaskDefinition,
    _user_provided_debug_adapter_path: Option<String>,
    worktree: &Worktree,
) -> zed::Result<DebugAdapterBinary, String> {
    let configuration = config.config.to_string();
    let dbg_config: NetCoreDebugConfig =
        serde_json::from_str(&configuration).map_err(|e| e.to_string())?;

    let request = match dbg_config.request.as_str() {
        "launch" => zed::StartDebuggingRequestArgumentsRequest::Launch,
        "attach" => zed::StartDebuggingRequestArgumentsRequest::Attach,
        unknown => {
            return Err(format!(
                "Invalid 'request' value: '{}'. Expected 'launch' or 'attach'",
                unknown
            ));
        }
    };

    let binary_path = ensure_installed()?;

    Ok(zed::DebugAdapterBinary {
        command: Some(binary_path),
        arguments: vec![],
        envs: dbg_config.env.into_iter().collect(),
        cwd: Some(dbg_config.cwd.unwrap_or_else(|| worktree.root_path())),
        connection: None,
        request_args: StartDebuggingRequestArguments {
            configuration,
            request,
        },
    })
}

pub fn dap_request_kind(
    config: zed_extension_api::serde_json::Value,
) -> zed::Result<zed::StartDebuggingRequestArgumentsRequest, String> {
    match config.get("request").and_then(|v| v.as_str()) {
        Some("launch") => Ok(zed::StartDebuggingRequestArgumentsRequest::Launch),
        Some("attach") => Ok(zed::StartDebuggingRequestArgumentsRequest::Attach),
        _ => Err("Invalid request".to_string()),
    }
}

pub fn dap_config_to_scenario(
    config: DebugConfig,
) -> zed_extension_api::Result<zed_extension_api::DebugScenario, String> {
    let dap_config = match config.request {
        DebugRequest::Launch(launch) => NetCoreDebugConfig {
            request: "launch".to_string(),
            program: Some(launch.program),
            args: if launch.args.is_empty() {
                None
            } else {
                Some(launch.args)
            },
            cwd: launch.cwd,
            env: launch.envs.into_iter().collect(),
            stop_at_entry: None,
            just_my_code: None,
            enable_step_filtering: None,
            process_id: None,
        },
        DebugRequest::Attach(attach) => {
            let pid = attach.process_id.ok_or("process_id not provided")?;

            NetCoreDebugConfig {
                request: "attach".to_string(),
                program: None,
                args: None,
                cwd: None,
                env: HashMap::new(),
                stop_at_entry: config.stop_on_entry,
                just_my_code: None,
                enable_step_filtering: None,
                process_id: Some(crate::debugger::ProcessId::Int(pid as i32)),
            }
        }
    };

    let json = serde_json::to_string(&dap_config)
        .map_err(|e| format!("Failed to serialize NetCoreDebugConfig: {}", e))?;

    Ok(DebugScenario {
        label: config.label,
        adapter: config.adapter,
        build: None,
        config: json,
        tcp_connection: None,
    })
}

/// Returns the absolute path to the `clrdbg` executable, downloading it when missing.
///
/// clrdbg ships as a self-contained build per platform (`clrdbg_<os>-<arch>.zip`)
/// with the executable at the archive root, so it runs without a `dotnet` host.
fn ensure_installed() -> Result<String, String> {
    let install_dir = format!("{ROOT_DIR}/{MODULE_DIR}");
    let executable = executable_name();
    let binary_path = format!("{install_dir}/{executable}");

    if !utils::is_file(&binary_path) {
        download_clrdbg(&install_dir)?;

        if !utils::is_file(&binary_path) {
            return Err(format!("Cannot find {executable} after download"));
        }
        zed::make_file_executable(&binary_path)?;
    }

    utils::get_absolute_path(&binary_path)
        .map(|path| path.to_string_lossy().to_string())
        .map_err(|e| format!("Cannot resolve {executable} path: {e}"))
}

fn executable_name() -> String {
    let (platform, _) = zed::current_platform();
    match platform {
        zed::Os::Windows => format!("{BINARY_NAME}.exe"),
        _ => BINARY_NAME.to_string(),
    }
}

fn asset_name() -> Result<String, String> {
    let (platform, arch) = zed::current_platform();

    let os = match platform {
        zed::Os::Mac => "osx",
        zed::Os::Linux => "linux",
        zed::Os::Windows => "win",
    };
    let arch = match arch {
        zed::Architecture::Aarch64 => "arm64",
        zed::Architecture::X8664 => "x64",
        zed::Architecture::X86 => {
            return Err(format!("{BINARY_NAME} is not available for 32-bit x86"));
        }
    };

    Ok(format!("{BINARY_NAME}_{os}-{arch}.zip"))
}

fn download_clrdbg(dest_dir: &str) -> Result<(), String> {
    let asset_name = asset_name()?;

    let zed::GithubRelease { version, assets } = zed::latest_github_release(
        REPO_SOURCE,
        zed::GithubReleaseOptions {
            require_assets: true,
            pre_release: false,
        },
    )?;

    let asset = assets
        .into_iter()
        .find(|asset| asset.name == asset_name)
        .ok_or_else(|| format!("Asset '{asset_name}' not found in {BINARY_NAME} release {version}"))?;

    zed::download_file(&asset.download_url, dest_dir, zed::DownloadedFileType::Zip)
        .map_err(|e| format!("Failed to download '{asset_name}': {e}"))
}
