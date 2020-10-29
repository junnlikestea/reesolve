extern crate trust_dns_resolver;

mod data;
mod error;
mod input;
mod resolver;

pub use crate::error::ReeError;
pub use crate::input::Input;
pub use crate::resolver::Resolver;
pub type Result<T> = std::result::Result<T, ReeError>;

#[derive(Debug)]
pub enum OutputFormat {
    Csv,
    Json,
}
