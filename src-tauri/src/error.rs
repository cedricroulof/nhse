//! Command error type (serialized to the UI as a plain message).
use serde::Serialize;

#[derive(Debug)]
pub struct CmdError(pub String);

impl Serialize for CmdError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}

impl<E: std::fmt::Display> From<E> for CmdError {
    fn from(e: E) -> Self {
        CmdError(e.to_string())
    }
}

pub type CmdResult<T> = Result<T, CmdError>;

pub fn err<T>(msg: impl Into<String>) -> CmdResult<T> {
    Err(CmdError(msg.into()))
}
