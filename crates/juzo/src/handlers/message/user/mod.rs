use juzo_core::filters::{Command, FloodKind, FloodType, FloodWait};
use telers::{Router, event::telegram::Handler};

mod anketa;
mod bag;
mod convey;
mod description;
mod start;
mod test;
mod trade;
mod user_id;

pub fn routers() -> Router {
    Router::new("router USER connect")
        .on_message(|observer| {
            observer.registers([
                Handler::new(anketa::first_appearance)
                    .filter(Command::many(&["рег", "регистрация"]).no_prefix()),
                Handler::new(convey::gold)
                    .filter(Command::many(&["биржа передать", "передать голд"]).no_prefix()),
                Handler::new(trade::sell).filter(Command::one("биржа продать").no_prefix()),
                Handler::new(trade::buy).filter(Command::one("биржа купить").no_prefix()),
                Handler::new(trade::book)
                    .filter(Command::one("биржа").no_prefix())
                    .filter(
                        FloodWait::new(FloodKind::OrderBook, FloodType::Chat)
                            .limit(1)
                            .second(30),
                    ),
                Handler::new(convey::score).filter(Command::one("передать од").no_prefix()),
                Handler::new(convey::sweets).filter(Command::one("передать").no_prefix()),
                Handler::new(test::sms_ids).filter(Command::one("смс ид").no_prefix()),
                Handler::new(test::time_sms).filter(Command::one("смс время").no_prefix()),
                Handler::new(test::ping).filter(Command::one("пинг").no_prefix()),
                Handler::new(test::creator_premium_pack).filter(Command::one("создатель пака")),
                Handler::new(user_id::show).filter(Command::one("ид")),
                Handler::new(bag::show).filter(Command::one("мешок").no_prefix()),
                #[cfg(debug_assertions)]
                Handler::new(test::message_reply).filter(Command::one("контент сообщения")),
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
