use std::{error, result};

pub type Result<T> = result::Result<T, Box<dyn error::Error + 'static>>;

#[derive(Debug)]
pub struct Error {
    details: Option<Box<dyn error::Error + 'static>>,
}

pub type ErrorPtr = *mut Error;
