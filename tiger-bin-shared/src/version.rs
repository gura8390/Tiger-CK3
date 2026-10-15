//! Helper functions for handling the game's version compared to tiger's expected game version.

use anyhow::Result;

fn numeric_version(version: &str) -> Result<Vec<u64>> {
    version.split_whitespace().next().unwrap_or("").split('.')
        .map(|part| part.parse().map_err(Into::into)).collect()
}

/// Compare the game version tiger was built for to the detected game version, and warn
/// on stderr if tiger is too old or the game is too old.
pub fn warn_versions(
    tiger_game: &str,
    tiger_game_version: &str,
    launcher_game_version: &str,
) -> Result<()> {
    let differs = numeric_version(tiger_game_version)? != numeric_version(launcher_game_version)?;
    if differs {
        eprintln!("WARNING: game version mismatch; validation continues. Compatibility is not guaranteed.");
    }
    let components: Vec<_> = tiger_game_version.split(' ').collect();
    let tiger_version_components: Vec<_> = components[0].split('.').collect();
    let tiger_major: i64 = tiger_version_components.first().unwrap_or(&"0").parse()?;
    let tiger_minor: i64 = tiger_version_components.get(1).unwrap_or(&"0").parse()?;
    let tiger_patch: i64 = tiger_version_components.get(2).unwrap_or(&"0").parse()?;
    let game_version_components: Vec<_> = launcher_game_version.split('.').collect();
    let game_major: i64 = game_version_components.first().unwrap_or(&"0").parse()?;
    let game_minor: i64 = game_version_components.get(1).unwrap_or(&"0").parse()?;
    let game_patch: i64 = game_version_components.get(2).unwrap_or(&"0").parse()?;

    if (tiger_major, tiger_minor) < (game_major, game_minor) {
        eprintln!();
        eprintln!("PLEASE UPDATE!");
        eprintln!();
        eprintln!("Tiger was made for {tiger_game} version {tiger_game_version},");
        eprintln!("but the newer version {launcher_game_version} was detected in the game files.");
        eprintln!("This may lead to erroneous reports from Tiger.");
        eprintln!("Please check if there is a newer version of Tiger that supports this version.");
    } else if (tiger_major, tiger_minor) > (game_major, game_minor) {
        eprintln!();
        eprintln!("OLD GAME DETECTED");
        eprintln!();
        eprintln!("Tiger was made for {tiger_game} version {tiger_game_version},");
        eprintln!("but the older version {launcher_game_version} was detected in the game files.");
        eprintln!("This may lead to erroneous reports from Tiger.");
        eprintln!(
            "Please consider updating your game, or perhaps downgrading Tiger to a more compatible version."
        );
    } else if tiger_patch < game_patch {
        eprintln!("Tiger was made for {tiger_game} version {tiger_game_version},");
        eprintln!(
            "but the slightly newer version {launcher_game_version} was detected in the game files."
        );
        eprintln!("This is probably okay, but there may be some erroneous reports.");
    } else if tiger_patch > game_patch {
        eprintln!("Tiger was made for {tiger_game} version {tiger_game_version},");
        eprintln!(
            "but the slightly older version {launcher_game_version} was detected in the game files."
        );
        eprintln!("This is probably okay, but there may be some erroneous reports.");
    } else if differs {
        eprintln!("Tiger was made for {tiger_game} version {tiger_game_version}, but game build {launcher_game_version} differs. Validation continues.");
    } else {
        eprintln!("Tiger was made for {tiger_game} version {tiger_game_version},");
        eprintln!("which matches the detected game version {launcher_game_version}.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_version_includes_hotfix_component() {
        assert_ne!(numeric_version("1.20.0.4").unwrap(), numeric_version("1.20.0.5").unwrap());
        assert_eq!(numeric_version("1.20.1 (Crozier)").unwrap(), numeric_version("1.20.1").unwrap());
    }
    #[test]
    fn mismatched_versions_never_block_validation() {
        for version in ["1.19.0", "1.20.0.4", "1.20.2", "1.21.0", "2.0.0"] {
            assert!(warn_versions("CK3", "1.20.1 (Crozier)", version).is_ok());
        }
    }
}
