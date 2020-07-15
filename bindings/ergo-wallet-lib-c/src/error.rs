use std::{error, fmt, result};

pub type Result<T> = result::Result<T, Box<dyn error::Error + 'static>>;

#[derive(Debug)]
pub struct Error {
    details: Box<dyn error::Error + 'static>,
}

pub type ErrorPtr = *mut Error;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.details.as_ref(), f)
    }
}
