mod types;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use types::DebuggerOptions;
use zed_extension_api::{
    self as zed, serde_json, AttachRequest, DebugAdapterBinary, DebugConfig, DebugRequest,
    DebugScenario, DebugTaskDefinition, StartDebuggingRequestArguments, Worktree,
};

use crate::{dotnet, github, utils, ROOT_DIR};

const MODULE_DIR: &str = "DebuggerMono";
const DLL_NAME: &str = "monodbg.dll";

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct MonoDebugConfig {
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
    pub process_id: Option<super::ProcessId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub debugger_options: Option<DebuggerOptions>,
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub ttype: Option<String>,
}

impl MonoDebugConfig {
    fn default_attach(config: DebugConfig, attach: AttachRequest) -> Result<DebugScenario, String> {
        if attach.process_id.is_none() {
            return Err("process_id is required".to_string());
        }

        // process id select from zed
        // let process_id = ProcessId::Int(attach.process_id.unwrap() as i32);

        let mono_debug_config = Self {
            request: "attach".to_string(),
            program: None,
            args: None,
            cwd: None,
            env: HashMap::new(),
            process_id: None,
            debugger_options: Some(DebuggerOptions::default()),
            ttype: Some("unity".to_string()),
        };
        let json = serde_json::to_string(&mono_debug_config)
            .map_err(|e| format!("Failed to serialized attach config: {}", e))?;

        Ok(DebugScenario {
            label: config.label,
            adapter: config.adapter,
            build: None,
            config: json,
            tcp_connection: None,
        })
    }
}

/// Returns the absolute path to `monodbg.dll`, downloading the
/// `DotRush.Bundle.DebuggerMono.zip` release asset when it is missing.
///
/// The debugger is a framework-dependent build started with `dotnet monodbg.dll`.
fn ensure_installed() -> Result<String, String> {
    let install_dir = format!("{ROOT_DIR}/{MODULE_DIR}");
    let dll_path = format!("{install_dir}/{DLL_NAME}");

    if !utils::is_file(&dll_path) {
        github::download_bundle(MODULE_DIR, &install_dir)?;

        if !utils::is_file(&dll_path) {
            return Err(format!("Cannot find {DLL_NAME} after download"));
        }
    }

    utils::get_absolute_path(&dll_path)
        .map(|path| path.to_string_lossy().to_string())
        .map_err(|e| format!("Cannot resolve {DLL_NAME} path: {e}"))
}

pub fn get_dap_binary(
    config: DebugTaskDefinition,
    _user_provided_debug_adapter_path: Option<String>,
    worktree: &Worktree,
) -> zed::Result<DebugAdapterBinary, String> {
    let dotnet = dotnet::find(worktree)?;
    let dll_path = ensure_installed()?;

    let configuration = config.config.to_string();
    let dbg_config: MonoDebugConfig =
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

    Ok(zed::DebugAdapterBinary {
        command: Some(dotnet),
        arguments: vec![dll_path],
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
    match config.request {
        DebugRequest::Launch(_launch) => Err("Launch not implemented".to_string()),
        DebugRequest::Attach(attach) => MonoDebugConfig::default_attach(config, attach),
    }
}
