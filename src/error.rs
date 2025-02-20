use std::{fmt::Debug, io};

use ron::de::SpannedError;

pub type Result<T> = std::result::Result<T, ErrorKind>;

pub enum ErrorKind {
    ReadParameterFile(io::Error),
    WriteParameterFile(io::Error),
    DecodeParameterFile(SpannedError),
    EncodeParameterFile(ron::Error),
    SaveImage(image::ImageError),
}

impl Debug for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::ReadParameterFile(e) => {
                writeln!(f, "Failed to read parameter file: {}", e)
            }
            ErrorKind::WriteParameterFile(e) => {
                writeln!(f, "Failed to write parameter file: {}", e)
            }
            ErrorKind::DecodeParameterFile(e) => {
                writeln!(f, "Failed to decode parameter file: {}", e)
            }
            ErrorKind::EncodeParameterFile(e) => {
                writeln!(f, "Failed to encode parameter file: {}", e)
            }
            ErrorKind::SaveImage(e) => {
                writeln!(f, "Failed to save image: {}", e)
            }
        }
    }
}
