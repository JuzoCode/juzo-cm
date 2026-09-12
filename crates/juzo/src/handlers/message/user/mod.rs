use juzo_core::filters::Command;
#[cfg(debug_assertions)]
use juzo_core::filters::{FloodKind, FloodType, FloodWait};
use telers::{Filter, Router, enums, event::telegram::Handler, filters::ChatType};

mod anketa;
mod bag;
mod chat_attach;
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
                Handler::new(bag::edit_show_false).filter(Command::many(&["-мешок"]).no_prefix()),
                Handler::new(bag::edit_show_true).filter(Command::many(&["+мешок"]).no_prefix()),
                Handler::new(anketa::edit_show_false)
                    .filter(Command::many(&["-анкета"]).no_prefix()),
                Handler::new(anketa::edit_show_true)
                    .filter(Command::many(&["+анкета"]).no_prefix()),
                Handler::new(anketa::show).filter(Command::many(&["анкета"]).no_prefix()),
                Handler::new(anketa::first_appearance)
                    .filter(Command::many(&["рег", "регистрация"]).no_prefix()),
                Handler::new(start::yes)
                    .filter(Command::many(&["start", "начать"]).no_prefix())
                    .filter(ChatType::one(enums::ChatType::Private)),
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
                Handler::new(test::sms_ids)
                    .filter(Command::one("смс ид").no_prefix())
                    .filter(ChatType::one(enums::ChatType::Private).invert()),
                Handler::new(test::time_sms).filter(Command::one("смс время").no_prefix()),
                Handler::new(test::ping).filter(Command::one("пинг").no_prefix()),
                Handler::new(user_id::show).filter(Command::one("ид")),
                Handler::new(bag::show).filter(Command::one("мешок").no_prefix()),
                Handler::new(test::show_thread_link)
                    .filter(Command::one("ветка"))
                    .filter(ChatType::one(enums::ChatType::Private).invert()),
                Handler::new(chat_attach::yes)
                    .filter(Command::one("привязать"))
                    .filter(ChatType::one(enums::ChatType::Private).invert()),
                Handler::new(chat_attach::no)
                    .filter(Command::one("отвязать"))
                    .filter(ChatType::one(enums::ChatType::Private)),
                Handler::new(test::my_spam)
                    .filter(Command::one("мой спам").no_prefix())
                    .filter(ChatType::one(enums::ChatType::Private)),
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
