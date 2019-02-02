use std::collections::HashSet;
use std::path::Path;

use super::failures::Failure;
use super::RenameOperation;

pub fn verify_operations(operations: &Vec<RenameOperation>,
                         longest_source_name: usize) -> Result<(), Failure> {
    let mut duplicates: HashSet<String> = HashSet::new();
    for operation in operations {
        println!("    {:width$}    =>    {}",
                 operation.from,
                 operation.to,
                 width = longest_source_name);
        // check for target name duplicates:
        if duplicates.contains(operation.to.as_str()) {
            return Err(Failure::file_failure(operation.to.to_string(),
                                             "Duplicate rename".to_string()));
        }
        duplicates.insert(operation.to.to_string());
        // check for renaming duplicates:
        if operation.from != operation.to {
            if Path::new(&operation.to).exists() {
                return Err(Failure::file_failure(operation.to.to_string(),
                                                 "File exists on file system".to_string()));
            }
        }
    }
    return Ok(());
}
