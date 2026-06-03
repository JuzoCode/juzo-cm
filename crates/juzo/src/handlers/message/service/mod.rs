use telers::{Router, enums, event::telegram::Handler, filters::MessageType};

mod creator_left;
mod managed_bot_created;

pub fn routers() -> Router {
    Router::new("router SERVICE connect").on_message(|observer| {
        observer.registers([Handler::new(managed_bot_created::send)
            .filter(MessageType::one(enums::MessageType::ManagedBotCreated))])
    })
}
