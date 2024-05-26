use std::fs;
use zed::LanguageServerId;
use zed_extension_api::{self as zed, Result};

struct HtmxExtension {
    cached_binary_path: Option<String>,
}

impl HtmxExtension {
    fn language_server_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
    ) -> Result<String> {
        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
                return Ok(path.clone());
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = zed::latest_github_release(
            "ThePrimeagen/htmx-lsp",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, arch) = zed::current_platform();
        let asset_name = format!(
            "htmx-lsp-{os}-{arch}{ext}",
            arch = match arch {
                zed::Architecture::Aarch64 => "arm64",
                zed::Architecture::X8664 => "x64",
                _ => return Err("Unsupported architecture".to_string()),
            },
            os = match platform {
                zed::Os::Mac => "darwin",
                zed::Os::Linux => "linux",
                zed::Os::Windows => "windows",
            },
            ext = match platform {
                zed::Os::Windows => ".exe",
                zed::Os::Mac | zed::Os::Linux => "",
            }
        );

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("No asset found matching {:?}", asset_name))?;

        let version_dir = format!("htmx-lsp-{}", release.version);
        fs::create_dir_all(&version_dir)
            .map_err(|err| format!("Failed to create directory '{version_dir}': {err}"))?;

        let binary_path = format!("{version_dir}/{asset_name}");

        if !fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(
                &asset.download_url,
                &binary_path,
                zed::DownloadedFileType::Uncompressed,
            )
            .map_err(|err| format!("Failed to download file: {err}"))?;

            zed::make_file_executable(&binary_path)
                .map_err(|err| format!("Failed to make file executable: {err}"))?;
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

impl zed::Extension for HtmxExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        Ok(zed::Command {
            command: self.language_server_binary_path(language_server_id)?,
            args: vec![],
            env: Default::default(),
        })
    }
}

zed::register_extension!(HtmxExtension);
