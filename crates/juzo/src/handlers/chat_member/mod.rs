use juzo_core::middlewares::inner::MemberTraffic;
pub use telers::types::ChatMemberUpdated;
use telers::{
    Router, enums::ChatMemberType as MemberStatus, event::telegram::Handler,
    filters::ChatMemberUpdated as MemberFilter,
};

pub use super::{Bot, DbConn, Extension, HandlerResult};

mod bot;
mod user;

pub fn routers() -> Router {
    Router::new("router CHAT MEMBER connect")
        .on_my_chat_member(|observer| {
            observer.registers([
                Handler::new(bot::leave::yes),
                Handler::new(bot::admin::set)
                    .filter(MemberFilter::new(MemberStatus::Administrator)),
            ])
        })
        .on_chat_member(|observer| observer.register_inner_middleware(MemberTraffic))
}
