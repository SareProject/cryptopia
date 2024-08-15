use std::{fs, path::PathBuf};

use rpassword;
use secrecy::SecretString;

use crate::error::SareCLIError;

// TODO: Return SareCLIError Instead Of String
pub fn read_cli_secret(prompt: impl ToString) -> Result<SecretString, String> {
    let secret: SecretString = rpassword::prompt_password(prompt)
        .map_err(|e| e.to_string())?
        .into();

    Ok(secret)
}

pub fn create_directory(path: &PathBuf) -> Result<PathBuf, SareCLIError> {
    log::debug!("Directory {} exists, Skipping!", path.to_string_lossy());
    if !path.exists() {
        fs::create_dir(path)?;
        log::debug!("Directory {} Initialized!", path.to_string_lossy())
    }
    Ok(path.to_owned())
}

pub fn prepare_sare_directory() -> Result<PathBuf, SareCLIError> {
    let home_directory = dirs::home_dir().unwrap_or(PathBuf::new());
    let sare_directory = create_directory(&home_directory.join(".sare"))?;

    create_directory(&sare_directory.join("private_keys"))?;
    create_directory(&sare_directory.join("public_keys"))?;
    create_directory(&sare_directory.join("revocations"))?;

    
    Ok(sare_directory)

}