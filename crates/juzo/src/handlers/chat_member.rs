use juzo_core::filters::{MemberEvent, MyMemberEvent};
pub use telers::types::ChatMemberUpdated;
use telers::{Router, enums, event::telegram::Handler, filters::ChatType};

pub use super::{Bot, DbConn, Extension, HandlerResult};

mod bot;
mod user;

pub fn routers() -> Router {
    Router::new("router CHAT MEMBER connect")
        .on_my_chat_member(|observer| {
            observer
                .filter(ChatType::one(enums::ChatType::Supergroup))
                .registers([
                    Handler::new(bot::admin::added).filter(MyMemberEvent::<2>::new().promoted())
                ])
        })
        .on_chat_member(|observer| {
            observer
                .filter(ChatType::one(enums::ChatType::Supergroup))
                .registers([
                    Handler::new(user::ban::yes).filter(MemberEvent::<1>::new().join()),
                    Handler::new(user::spam::yes).filter(MemberEvent::<1>::new().join()),
                    Handler::new(user::ban::no_tg).filter(MemberEvent::<2>::new().demoted()),
                ])
        })
}
