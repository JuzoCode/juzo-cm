pub use juzo_core::{
    application::JuzoAnswer,
    filters::{ArgsResult, CommandResult},
};
use telers::Router;
pub use telers::types::Message;

pub use super::{Bot, Extension, HandlerResult};

mod agent;
mod moderator;
mod service;
mod user;

pub fn routers() -> Router {
    Router::new("router MESSAGE connect")
        .include(user::routers())
        .include(agent::routers())
        .include(service::routers())
        .include(moderator::routers())
}
