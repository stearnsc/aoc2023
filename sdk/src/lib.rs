pub use log::{trace, debug, info, warn, error};
pub use anyhow::Result;
pub use anyhow;
pub use dotenvy;
pub use lazy_static::lazy_static;

pub fn init() {
    dotenvy::dotenv().expect(".env file not found");
    pretty_env_logger::init();
}

