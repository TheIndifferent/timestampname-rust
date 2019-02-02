use std::fs;
use std::os::unix::fs::PermissionsExt;

use super::RenameOperation;
use super::failures::Failure;

pub fn execute_operations(operations: &Vec<RenameOperation>, dry_run: bool) -> Result<(), Failure> {
    for (i, o) in operations.iter().enumerate() {
        print!("\rRenaming files: {}/{}...", i + 1, operations.len());
        if !dry_run {
            fs::rename(&o.from, &o.to)
                .map_err(|e| Failure::file_failure_caused(
                    o.from.to_string(),
                    "Failed to rename".to_string(),
                    e))?;
            if cfg!(unix) {
                fs::set_permissions(&o.to, PermissionsExt::from_mode(0o444))
                    .map_err(|e| Failure::file_failure_caused(
                        o.to.to_string(),
                        "Failed to set permissions".to_string(),
                        e))?;
            }
        }
    }
    println!(" done.");
    return Ok(());
}
