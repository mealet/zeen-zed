use zed_extension_api::{
    self as zed, current_platform, latest_github_release, Architecture, Command,
    DownloadedFileType, GithubReleaseOptions, LanguageServerId, Os, Result, Worktree,
};

struct ZeenExtension;

impl zed::Extension for ZeenExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &LanguageServerId,
        _worktree: &Worktree,
    ) -> Result<Command> {
        let (os, arch) = current_platform();
        let platform = match (os, arch) {
            (Os::Linux, Architecture::X8664) => "linux-x86_64-gnu",
            (Os::Linux, Architecture::Aarch64) => "linux-aarch64-gnu",
            (Os::Mac, Architecture::Aarch64) => "darwin-aarch64",
            (Os::Mac, Architecture::X8664) => "darwin-x86_64",
            (Os::Windows, Architecture::X8664) => "windows-x86_64",
            _ => return Err(format!("unsupported platform {os:?}/{arch:?}")),
        };

        let release = latest_github_release(
            "mealet/zeen",
            GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;
        let version = release.version.trim_start_matches('v');

        let file_name = format!("zeen-{version}-{platform}.tar.gz");
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == file_name)
            .ok_or_else(|| format!("no asset named {file_name} in release"))?;

        let exe = if os == Os::Windows {
            "zeen-lsp.exe"
        } else {
            "zeen-lsp"
        };
        let dir = format!("zeen-lsp-{version}");
        let binary = format!("{dir}/zeen-{version}-{platform}/bin/{exe}");

        if !std::fs::metadata(&binary).is_ok_and(|stat| stat.is_file()) {
            zed::download_file(&asset.download_url, &dir, DownloadedFileType::GzipTar)
                .map_err(|message| format!("failed to download zeen-lsp: {message}"))?;
            zed::make_file_executable(&binary)
                .map_err(|message| format!("failed to chmod zeen-lsp: {message}"))?;
        }

        Ok(Command {
            command: binary,
            args: vec![],
            env: vec![],
        })
    }
}

zed::register_extension!(ZeenExtension);
