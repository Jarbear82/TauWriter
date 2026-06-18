use zed_extension_api::{self as zed, LanguageServerId, Result};

pub struct TauWriterExtension;

impl zed::Extension for TauWriterExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        // 1. Detect platform and architecture
        let (platform, arch) = zed::current_platform();

        // 2. Map to the local bin/ subdirectory
        let binary_name = match platform {
            zed::Os::Mac => match arch {
                zed::Architecture::Aarch64 => "tauwriter-lsp-macos-arm64",
                zed::Architecture::X8664 => "tauwriter-lsp-macos-x64",
                _ => return Err("Unsupported macOS architecture".into()),
            },
            zed::Os::Linux => match arch {
                zed::Architecture::X8664 => "tauwriter-lsp-linux-x64",
                _ => return Err("Unsupported Linux architecture".into()),
            },
            zed::Os::Windows => "tauwriter-lsp-windows.exe",
        };

        // This path is relative to the "work" directory
        let binary_path = format!("bin/{}", binary_name);

        if std::fs::metadata(&binary_path).is_err() {
            // Binary not found in work directory, attempt to download from GitHub
            // Note: Replace 'main' with a specific tag/version in production for stability
            let url = format!(
                "https://raw.githubusercontent.com/Jarbear82/TauWriter/main/extension/bin/{}",
                binary_name
            );

            zed::download_file(&url, &binary_path, zed::DownloadedFileType::Uncompressed)
                .map_err(|e| format!("Failed to download binary from {}: {}", url, e))?;

            zed::make_file_executable(&binary_path)?;
        }

        Ok(zed::Command {
            command: binary_path,
            args: vec!["--stdio".to_string()],
            env: Default::default(),
        })
    }
}

zed::register_extension!(TauWriterExtension);
