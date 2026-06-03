use juzo_core::{
    filters::Business,
    middlewares::{
        inner::Error,
        outer::{ChatSync, UserSync},
    },
};
use telers::Router;
pub use telers::{Bot, Extension, event::telegram::HandlerResult};

mod callback;
mod chat_member;
mod managed_bot;
mod message;

pub fn routers_connect() -> Router {
    Router::new("router connect")
        .on_all(|observer| {
            observer
                // Outer-middleware
                .register_outer_middleware(UserSync)
                .register_outer_middleware(ChatSync)
                // Inner-middleware
                .register_inner_middleware(Error)
        })
        .on_business_message(|observer| observer.filter(Business))
        .include(message::routers())
        .include(callback::routers())
        .include(chat_member::routers())
        .include(managed_bot::routers())
}
