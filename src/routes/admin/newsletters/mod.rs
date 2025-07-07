mod get;
mod post;

pub use get::*;
pub use post::{PublishError, publish_newsletters_handler};
