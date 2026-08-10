use telers::{Router, enums, event::telegram::Handler, filters::MessageType};

mod chat_owner;
mod community;
mod managed_bot_created;

pub fn routers() -> Router {
    Router::new("router SERVICE connect").on_message(|observer| {
        observer.registers([
            Handler::new(community::added)
                .filter(MessageType::one(enums::MessageType::CommunityChatAdded)),
            Handler::new(community::removed)
                .filter(MessageType::one(enums::MessageType::CommunityChatRemoved)),
            Handler::new(chat_owner::left)
                .filter(MessageType::one(enums::MessageType::ChatOwnerLeft)),
            Handler::new(managed_bot_created::send)
                .filter(MessageType::one(enums::MessageType::ManagedBotCreated)),
        ])
    })
}
