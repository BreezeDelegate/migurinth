use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use uuid::Uuid;

const ISOLATED_DIRS: &[&str] =
    &["saves", "logs", "crash-reports", "screenshots", "backups"];

fn mirror_files(
    source: &Path,
    destination: &Path,
    hardlink_jars: bool,
) -> io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            continue;
        }
        let target = destination.join(entry.file_name());
        if kind.is_dir() {
            mirror_files(&entry.path(), &target, hardlink_jars)?;
        } else if kind.is_file() && !target.exists() {
            let is_jar = entry
                .path()
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("jar"));
            if !hardlink_jars
                || !is_jar
                || fs::hard_link(entry.path(), &target).is_err()
            {
                fs::copy(entry.path(), &target)?;
            }
        }
    }
    Ok(())
}

fn sync_mods(source: &Path, destination: &Path) -> io::Result<()> {
    if !source.is_dir() {
        return Ok(());
    }
    // Mod jars are immutable downloads. A hard link avoids duplicating a large pack.
    // Only remove a derived mod directory when it is an ordinary directory, never a link.
    if destination.exists() {
        if destination.symlink_metadata()?.file_type().is_symlink() {
            return Err(io::Error::other(
                "Refusing to replace a linked session mods directory",
            ));
        }
        fs::remove_dir_all(destination)?;
    }
    mirror_files(source, destination, true)
}

fn prepare_account_directory(
    game_directory: &Path,
    config_directory: &Path,
    instance_id: &str,
    account_id: Uuid,
) -> io::Result<PathBuf> {
    // Instance identifiers are opaque and may contain Windows path separators.
    // Keep the legacy UUID directory name for existing users; hash all other IDs
    // into a deterministic, filesystem-safe component without exposing the ID.
    let safe_instance_id = Uuid::parse_str(instance_id)
        .map(|uuid| uuid.to_string())
        .unwrap_or_else(|_| {
            format!("{:x}", Sha256::digest(instance_id.as_bytes()))
        });
    let root = config_directory
        .join("account-sessions")
        .join(safe_instance_id);
    fs::create_dir_all(&root)?;

    let primary_file = root.join("primary-account");
    let primary = match fs::read_to_string(&primary_file) {
        Ok(value) => Uuid::parse_str(value.trim()).map_err(io::Error::other)?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&primary_file)
            {
                Ok(mut file) => {
                    file.write_all(account_id.to_string().as_bytes())?;
                    file.sync_all()?;
                    account_id
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                    let existing = fs::read_to_string(&primary_file)?;
                    Uuid::parse_str(existing.trim())
                        .map_err(io::Error::other)?
                }
                Err(error) => return Err(error),
            }
        }
        Err(error) => return Err(error),
    };

    if account_id == primary {
        return Ok(game_directory.to_path_buf());
    }

    let secondary_directory = root.join(account_id.to_string());
    fs::create_dir_all(&secondary_directory)?;

    for entry in fs::read_dir(game_directory)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if ISOLATED_DIRS.contains(&name_str.as_ref()) || name_str == "mods" {
            continue;
        }
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            continue;
        }
        let destination = secondary_directory.join(&name);
        if kind.is_dir() {
            mirror_files(&entry.path(), &destination, false)?;
        } else if kind.is_file() && !destination.exists() {
            fs::copy(entry.path(), destination)?;
        }
    }

    sync_mods(
        &game_directory.join("mods"),
        &secondary_directory.join("mods"),
    )?;
    Ok(secondary_directory)
}

pub async fn game_directory_for_account(
    game_directory: &Path,
    config_directory: &Path,
    instance_id: &str,
    account_id: Uuid,
) -> crate::Result<PathBuf> {
    let game_directory = game_directory.to_path_buf();
    let config_directory = config_directory.to_path_buf();
    let instance_id = instance_id.to_owned();
    tokio::task::spawn_blocking(move || {
        prepare_account_directory(
            &game_directory,
            &config_directory,
            &instance_id,
            account_id,
        )
    })
    .await
    .map_err(|error| {
        crate::ErrorKind::LauncherError(error.to_string()).as_error()
    })?
    .map_err(|error| {
        crate::ErrorKind::LauncherError(error.to_string()).as_error()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn non_uuid_windows_instance_paths_are_accepted() {
        let root = tempfile::tempdir().unwrap();
        let game = root.path().join("game");
        let meta = root.path().join("meta");
        fs::create_dir_all(game.join("mods")).unwrap();
        fs::write(game.join("mods/test.jar"), b"mod").unwrap();

        let primary = Uuid::new_v4();
        let secondary = Uuid::new_v4();
        let windows_path = r"\custom-minecraft\my-pack";

        let primary_dir =
            prepare_account_directory(&game, &meta, windows_path, primary)
                .unwrap();
        assert_eq!(primary_dir, game);

        let secondary_dir =
            prepare_account_directory(&game, &meta, windows_path, secondary)
                .unwrap();
        assert_ne!(secondary_dir, game);
        assert!(secondary_dir.starts_with(meta.join("account-sessions")));
        assert!(secondary_dir.join("mods/test.jar").exists());

        let secondary_again =
            prepare_account_directory(&game, &meta, windows_path, secondary)
                .unwrap();
        assert_eq!(secondary_dir, secondary_again);
        assert_eq!(
            prepare_account_directory(&game, &meta, windows_path, primary)
                .unwrap(),
            game
        );

        let unrelated = prepare_account_directory(
            &game,
            &meta,
            r"\custom-minecraft\other-pack",
            primary,
        )
        .unwrap();
        assert_eq!(unrelated, game);
        let unrelated_secondary = prepare_account_directory(
            &game,
            &meta,
            r"\custom-minecraft\other-pack",
            secondary,
        )
        .unwrap();
        assert_ne!(secondary_dir, unrelated_secondary);
    }

    #[test]
    fn sessions_keep_primary_and_secondary_files_separate() {
        let root = tempfile::tempdir().unwrap();
        let game = root.path().join("game");
        let meta = root.path().join("meta");
        fs::create_dir_all(game.join("mods")).unwrap();
        fs::create_dir_all(game.join("config")).unwrap();
        fs::create_dir_all(game.join("saves/world")).unwrap();
        fs::write(game.join("mods/test.jar"), b"mod").unwrap();
        fs::write(game.join("config/settings.toml"), b"enabled").unwrap();
        fs::write(game.join("saves/world/level.dat"), b"save").unwrap();
        let instance_id = Uuid::new_v4().to_string();
        let primary = Uuid::new_v4();
        let secondary = Uuid::new_v4();
        assert_eq!(
            prepare_account_directory(&game, &meta, &instance_id, primary)
                .unwrap(),
            game
        );
        let secondary_dir =
            prepare_account_directory(&game, &meta, &instance_id, secondary)
                .unwrap();
        assert!(secondary_dir.join("mods/test.jar").exists());
        assert!(secondary_dir.join("config/settings.toml").exists());
        assert!(!secondary_dir.join("saves/world/level.dat").exists());
        fs::write(secondary_dir.join("config/settings.toml"), b"secondary")
            .unwrap();
        fs::write(game.join("mods/new.jar"), b"new").unwrap();
        prepare_account_directory(&game, &meta, &instance_id, secondary)
            .unwrap();
        assert_eq!(
            fs::read(secondary_dir.join("config/settings.toml")).unwrap(),
            b"secondary"
        );
        assert!(secondary_dir.join("mods/new.jar").exists());
        assert_eq!(
            prepare_account_directory(&game, &meta, &instance_id, primary)
                .unwrap(),
            game
        );
    }
}
