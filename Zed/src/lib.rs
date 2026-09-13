mod debugger;
mod dotnet;
mod github;
mod utils;

use zed::serde_json;
use zed_extension_api::settings::LspSettings;
use zed_extension_api::{
    self as zed, DebugAdapterBinary, DebugConfig, DebugTaskDefinition, LanguageServerId, Result,
    Worktree,
};

pub(crate) const ROOT_DIR: &str = "./bin";
pub(crate) const GITHUB_REPO: &str = "JaneySprings/DotRush";

const LANGUAGE_SERVER_MODULE: &str = "LanguageServer";
const LANGUAGE_SERVER_DLL: &str = "DotRush.dll";

struct DotRushExtension {}
impl DotRushExtension {
    /// Returns the absolute path to `DotRush.dll`.
    ///
    /// The language server is a framework-dependent build without an app host,
    /// so it is shipped as a single universal `DotRush.Bundle.LanguageServer.zip`
    /// asset and started with `dotnet DotRush.dll`.
    fn language_server_dll(&mut self, language_server_id: &LanguageServerId) -> Result<String> {
        let install_dir = format!("{ROOT_DIR}/{LANGUAGE_SERVER_MODULE}");
        let dll_path = format!("{install_dir}/{LANGUAGE_SERVER_DLL}");

        if !utils::is_file(&dll_path) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            let downloaded = github::download_bundle(LANGUAGE_SERVER_MODULE, &install_dir)
                .and_then(|_| {
                    if utils::is_file(&dll_path) {
                        Ok(())
                    } else {
                        Err(format!(
                            "{LANGUAGE_SERVER_DLL} not found after extracting the language server bundle"
                        ))
                    }
                });

            if let Err(error) = downloaded {
                zed::set_language_server_installation_status(
                    language_server_id,
                    &zed::LanguageServerInstallationStatus::Failed(error.clone()),
                );
                return Err(error);
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::None,
        );

        utils::get_absolute_path(&dll_path)
            .map(|path| path.to_string_lossy().to_string())
            .map_err(|e| format!("Cannot resolve {LANGUAGE_SERVER_DLL} path: {e}"))
    }
}

impl zed::Extension for DotRushExtension {
    fn new() -> Self {
        Self {}
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &Worktree,
    ) -> Result<zed::Command> {
        // Resolve `dotnet` first: without a runtime there is no point in downloading the server.
        let dotnet = dotnet::find(worktree)?;
        let dll_path = self.language_server_dll(language_server_id)?;

        Ok(zed::Command {
            command: dotnet,
            args: vec![dll_path],
            env: Default::default(),
        })
    }

    fn language_server_workspace_configuration(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        let settings = LspSettings::for_worktree("dotrush", worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.settings.clone())
            .unwrap_or_default();
        Ok(Some(settings))
    }

    fn get_dap_binary(
        &mut self,
        adapter_name: String,
        config: DebugTaskDefinition,
        user_provided_debug_adapter_path: Option<String>,
        worktree: &Worktree,
    ) -> zed::Result<DebugAdapterBinary, String> {
        match adapter_name.as_str() {
            "monodbg" => {
                debugger::mono::get_dap_binary(config, user_provided_debug_adapter_path, worktree)
            }
            "ncdbg" => {
                debugger::ncdbg::get_dap_binary(config, user_provided_debug_adapter_path, worktree)
            }
            _ => todo!(),
        }
    }

    fn dap_request_kind(
        &mut self,
        adapter_name: String,
        config: zed_extension_api::serde_json::Value,
    ) -> zed::Result<zed::StartDebuggingRequestArgumentsRequest, String> {
        match adapter_name.as_str() {
            "monodbg" => debugger::mono::dap_request_kind(config),
            "ncdbg" => debugger::ncdbg::dap_request_kind(config),
            _ => todo!(),
        }
    }

    fn dap_config_to_scenario(
        &mut self,
        config: DebugConfig,
    ) -> zed_extension_api::Result<zed_extension_api::DebugScenario, String> {
        match config.adapter.as_str() {
            "monodbg" => debugger::mono::dap_config_to_scenario(config),
            "ncdbg" => debugger::ncdbg::dap_config_to_scenario(config),
            _ => todo!(),
        }
    }
}

zed::register_extension!(DotRushExtension);
