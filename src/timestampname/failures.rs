use std::fmt;
use std::fmt::Formatter;
use std::error::Error;

#[derive(Debug)]
pub struct FileError {
    file_name: String,
    description: String,
    cause: Option<Box<Error>>
}

#[derive(Debug)]
pub struct EnvError {
    operation: String,
    cause: Option<Box<Error>>
}

impl std::error::Error for FileError {}
impl std::error::Error for EnvError {}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "\tFile: {}\n\tDescription: {}", self.file_name, self.description)?;
        if self.cause.is_some() {
            write!(f, "\n\tCause: {}", self.cause.unwrap())?;
        }
        return Ok(())
    }
}

impl fmt::Display for EnvError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "\tOperation: {}", self.description)?;
        if self.cause.is_some() {
            write!(f, "\n\tCause: {}", self.cause.unwrap())?;
        }
        return Ok(())
    }
}
