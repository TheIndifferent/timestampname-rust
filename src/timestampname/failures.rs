use std::fmt;
use std::fmt::Formatter;
use std::error::Error;

#[derive(Debug)]
enum FailureType {
    File {
        file_name: String,
        description: String,
        cause: Option<Box<Error>>,
    },
    Env {
        operation: String,
        cause: Option<Box<Error>>,
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
                write!(f, "\tFile: {}\n\tDescription: {}", *file_name, *description)?;
                if cause.is_some() {
                    write!(f, "\n\tCause: {}", *cause.unwrap())?;
                }
            }
            FailureType::Env { operation, cause } => {
                write!(f, "\tOperation: {}", operation)?;
                if cause.is_some() {
                    write!(f, "\n\tCause: {}", cause.unwrap())?;
                }
            }
        };
        return Ok(());
    }
}

impl Failure {
    pub fn env_failure_caused<E: Error + 'static>(operation: String, cause: E) -> Failure {
        Failure {
            fail_type: FailureType::Env {
                operation,
                cause: Option::Some(Box::new(cause)),
            }
        }
    }
}
