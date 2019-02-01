use std::fmt;
use std::fmt::Formatter;
use std::error::Error;

#[derive(Debug)]
enum FailureType {
    File {
        file_name: String,
        description: String,
        cause: String
    },
    Env {
        operation: String,
        cause: String
    },
}

#[derive(Debug)]
pub struct Failure {
    fail_type: FailureType
}

impl std::error::Error for Failure {}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match &self.fail_type {
            FailureType::File { file_name, description, cause } => {
                write!(f, "\tFile: {}\n\tDescription: {}{}", *file_name, *description, *cause)?;
            }
            FailureType::Env { operation, cause } => {
                write!(f, "\tOperation: {}{}", *operation, *cause)?;
            }
        };
        return Ok(());
    }
}

impl Failure {
    pub fn env_failure_caused<E: Error>(operation: String, cause: E) -> Failure {
        Failure {
            fail_type: FailureType::Env {
                operation,
                cause: format!("\n\tCause: {}", cause),
            }
        }
    }
}
