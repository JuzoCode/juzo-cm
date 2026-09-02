use juzo_core::filters::Command;
#[cfg(debug_assertions)]
use juzo_core::filters::{FloodKind, FloodType, FloodWait};
use telers::{Router, event::telegram::Handler};

mod anketa;
mod bag;
mod convey;
mod description;
mod start;
mod test;
#[cfg(debug_assertions)]
mod trade;
mod user_id;

pub fn routers() -> Router {
    Router::new("router USER connect")
        .on_message(|observer| {
            observer.registers([
                Handler::new(anketa::first_appearance)
                    .filter(Command::many(&["рег", "регистрация"]).no_prefix()),
                Handler::new(start::yes).filter(Command::many(&["start", "начать"]).no_prefix()),
                Handler::new(start::help).filter(Command::many(&["помощь", "help"]).no_prefix()),
                Handler::new(convey::gold).filter(
                    Command::many(&["биржа передать", "передать голд", "перевести голд"])
                        .no_prefix(),
                ),
                #[cfg(debug_assertions)]
                Handler::new(trade::sell).filter(Command::one("биржа продать").no_prefix()),
                #[cfg(debug_assertions)]
                Handler::new(trade::buy).filter(Command::one("биржа купить").no_prefix()),
                #[cfg(debug_assertions)]
                Handler::new(trade::book)
                    .filter(Command::one("биржа").no_prefix())
                    .filter(
                        FloodWait::new(FloodKind::OrderBook, FloodType::Chat)
                            .limit(1)
                            .second(60),
                    ),
                Handler::new(convey::score)
                    .filter(Command::many(&["передать од", "перевести од"]).no_prefix()),
                Handler::new(convey::sweets)
                    .filter(Command::many(&["передать", "перевести"]).no_prefix()),
                Handler::new(test::sms_ids).filter(Command::one("смс ид").no_prefix()),
                Handler::new(test::time_sms).filter(Command::one("смс время").no_prefix()),
                Handler::new(test::ping).filter(Command::one("пинг").no_prefix()),
                Handler::new(user_id::show).filter(Command::one("ид")),
                Handler::new(test::chat_ids).filter(Command::one("чат ид")),
                Handler::new(bag::show).filter(Command::one("мешок").no_prefix()),
                Handler::new(test::show_thread_link).filter(Command::one("ветка")),
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
