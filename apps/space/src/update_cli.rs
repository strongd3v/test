//! `space update` — check for and apply a newer release, natively (the same
//! flow the installer performs: download → verify → symlink swap → service
//! restart). macOS app bundles swap the bundle instead; source builds are
//! report-only.

use anyhow::bail;
use space_update::{InstallKind, current_version, version_newer};

/// `--check` prints the verdict and exits (nonzero when an update is available,
/// so scripts can gate on it).
pub async fn update(check_only: bool) -> anyhow::Result<()> {
    let manifest = space_update::fetch_latest().await?;
    let current = current_version();
    if !version_newer(&manifest.version, current) {
        println!(
            "space {current} is up to date (latest: {}).",
            manifest.version
        );
        return Ok(());
    }
    println!("space {current} → {} available", manifest.version);
    if check_only {
        std::process::exit(1);
    }

    let install = space_update::detect_install();
    if let Some(blocker) = install.desktop_update_blocker() {
        bail!("{blocker}");
    }
    match install {
        InstallKind::Managed { app_root } => {
            println!(
                "downloading {}…",
                space_update::headless_artifact(&manifest.version)
            );
            space_update::stage_headless(&manifest, &app_root).await?;
            space_update::apply_headless(&app_root, &manifest.version)?;
            println!(
                "installed {} (current → {})",
                app_root.join(&manifest.version).display(),
                manifest.version
            );
            match space_update::restart_service() {
                Ok(()) => println!("engine service restarted."),
                Err(err) => println!(
                    "note: service restart failed ({err:#}) — restart the engine manually to finish."
                ),
            }
            Ok(())
        }
        InstallKind::MacApp { bundle } => {
            println!(
                "downloading {}…",
                space_update::mac_app_artifact(&manifest.version)
            );
            let data_dir = super::paths::data_dir();
            let staged = space_update::stage_mac_app(&manifest, &data_dir).await?;
            space_update::apply_mac_app(&staged, &bundle)?;
            println!("updated {} — relaunch Space to finish.", bundle.display());
            Ok(())
        }
        #[cfg(windows)]
        InstallKind::WindowsPortable { directory } => {
            let staged = space_update::windows::stage(&manifest, &directory).await?;
            space_update::windows::apply(&staged, &directory, false)?;
            println!(
                "updated to {} — relaunch Space to finish.",
                manifest.version
            );
            Ok(())
        }
        InstallKind::Unmanaged => {
            bail!(
                "this binary is not update-managed (source build or hand-copied).\n\
                 Linux: curl -fsSL https://space.pentomake.com/install.sh | sh, or run install.sh from the release tarball\n\
                 macOS: download the new Space.app dmg, or rebuild from source.\n\
                 Windows: install with the Space setup .exe from {}, or rebuild from source.",
                space_update::LATEST_RELEASE_PAGE
            )
        }
    }
}
