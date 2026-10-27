// pub use telers::types::ChatJoinRequest;
use telers::{
    Router,
    enums,
    // event::telegram::Handler,
    filters::ChatType,
};

// pub use super::{Bot, DbConn, Extension, HandlerResult};

mod spam;

pub fn routers() -> Router {
    Router::new("router CHAT JOIN REQUEST connect").on_chat_join_request(|observer| {
        observer.filter(ChatType::one(enums::ChatType::Supergroup))
    })
}
