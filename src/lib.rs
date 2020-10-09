extern crate trust_dns_resolver;

mod data;
mod input;
mod resolver;

pub use crate::input::Input;
pub use crate::resolver::Resolver;
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
