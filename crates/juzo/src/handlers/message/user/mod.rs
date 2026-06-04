use juzo_core::filters::Command;
use telers::{Router, event::telegram::Handler};

mod anketa;
mod bag;
mod description;
mod start;
mod test;
mod user_id;

pub fn routers() -> Router {
    Router::new("router USER connect")
        .on_message(|observer| {
            observer.registers([
                Handler::new(test::sms_ids).filter(Command::one("смс ид").no_prefix()),
                Handler::new(test::time_sms).filter(Command::one("смс время").no_prefix()),
                Handler::new(test::ping).filter(Command::one("пинг").no_prefix()),
                Handler::new(test::creator_premium_pack).filter(Command::one("создатель пака")),
                Handler::new(user_id::show).filter(Command::one("ид")),
                Handler::new(bag::show).filter(Command::one("мешок").no_prefix()),
            ])
        })
        .on_business_message(|observer| {
            observer.registers([
                Handler::new(test::ping).filter(Command::one("пинг")),
                Handler::new(user_id::show).filter(Command::one("ид")),
                Handler::new(test::time_sms).filter(Command::one("смс время")),
            ])
        })
}
