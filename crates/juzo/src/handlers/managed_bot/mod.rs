pub use telers::types::ManagedBotUpdated;
use telers::{Router, event::telegram::Handler};

pub use super::{Bot, HandlerResult};

mod managed;

pub fn routers() -> Router {
    Router::new("router MANAGED BOT connect")
        .on_managed_bot(|observer| observer.on(Handler::new(managed::set)))
}
