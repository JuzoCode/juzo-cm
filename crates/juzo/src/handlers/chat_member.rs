use juzo_core::filters::MemberEvent;
pub use telers::types::ChatMemberUpdated;
use telers::{
    Router, enums,
    enums::ChatMemberType as MemberStatus,
    event::telegram::Handler,
    filters::{ChatMemberUpdated as MemberFilter, ChatType},
};

pub use super::{Bot, DbConn, Extension, HandlerResult};

mod bot;
mod user;

pub fn routers() -> Router {
    Router::new("router CHAT MEMBER connect")
        .on_my_chat_member(|observer| {
            observer
                .filter(ChatType::one(enums::ChatType::Supergroup))
                .registers([
                    Handler::new(bot::leave::yes),
                    Handler::new(bot::admin::added)
                        .filter(MemberFilter::new(MemberStatus::Administrator)),
                ])
        })
        .on_chat_member(|observer| {
            observer
                .filter(ChatType::one(enums::ChatType::Supergroup))
                .registers([
                    Handler::new(user::ban::yes).filter(MemberEvent::new().join()),
                    Handler::new(user::spam::yes).filter(MemberEvent::new().join()),
                ])
        })
}
