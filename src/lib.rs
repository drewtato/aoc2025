mod error;
pub use error::AocError;
pub mod runner;

pub type Res<T> = Result<T, AocError>;
pub use runner::Settings;
