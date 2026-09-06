use juzo_core::filters::Command;
use telers::{
    Router,
    event::telegram::Handler,
    // enums::ChatType, filters::ChatType as FilterChatType,
};

mod agent;
mod bans_user;
mod chat_domen;
mod ignore;
mod managed_bot;
mod module;
mod scam;
mod spam;
mod takeaway;

pub fn routers() -> Router {
    Router::new("router AGENT connect")
        .on_message(|observer| {
            observer.registers([
                Handler::new(agent::edit_show_false)
                    .filter(Command::one("-моя видимость").no_prefix()),
                Handler::new(agent::edit_show_true)
                    .filter(Command::one("+моя видимость").no_prefix()),
                Handler::new(managed_bot::info).filter(Command::one("клон инфо")),
                Handler::new(managed_bot::delete).filter(Command::one("-клон").no_prefix()),
                Handler::new(managed_bot::set_official)
                    .filter(Command::one("+офф джузо клон").no_prefix()),
                Handler::new(agent::add_main).filter(Command::one("+гл агент").no_prefix()),
                Handler::new(agent::delete_main).filter(Command::one("-гл агент").no_prefix()),
                Handler::new(agent::add_spam).filter(Command::one("+ас агент").no_prefix()),
                Handler::new(agent::delete_spam).filter(Command::one("-ас агент").no_prefix()),
                Handler::new(agent::add).filter(Command::one("+агент").no_prefix()),
                Handler::new(agent::delete).filter(Command::one("-агент").no_prefix()),
                Handler::new(chat_domen::edit).filter(Command::one("домен")),
                Handler::new(ignore::add).filter(Command::one("+игнор").no_prefix()),
                Handler::new(ignore::delete_takeaway)
                    .filter(Command::one("-игнор ошибка").no_prefix()),
                Handler::new(ignore::delete).filter(Command::one("-игнор").no_prefix()),
                Handler::new(scam::add).filter(Command::one("+скам").no_prefix()),
                Handler::new(scam::delete).filter(Command::one("-скам").no_prefix()),
                Handler::new(spam::add).filter(Command::many(&["+ас", "+спам"]).no_prefix()),
                Handler::new(spam::delete_takeaway)
                    .filter(Command::many(&["-ас ошибка", "-спам ошибка"]).no_prefix()),
                Handler::new(spam::delete).filter(Command::many(&["-ас", "-спам"]).no_prefix()),
                Handler::new(takeaway::delete_spam).filter(
                    Command::many(&["-вынос ас", "-вынос спам", "-вынос антиспам"]).no_prefix(),
                ),
                Handler::new(takeaway::delete_ignore)
                    .filter(Command::one("-вынос игнор").no_prefix()),
                Handler::new(module::show_add_parent)
                    .filter(Command::one("+!модуль раздел").no_prefix()),
                Handler::new(module::add_parent).filter(Command::one("+модуль раздел")),
                Handler::new(module::show_add).filter(Command::one("+!модуль").no_prefix()),
                Handler::new(module::add).filter(Command::one("+модуль")),
            ])
        })
        .on_business_message(|observer| {
            observer.registers([
                Handler::new(managed_bot::info).filter(Command::one("клон инфо")),
                Handler::new(managed_bot::delete).filter(Command::one("-клон")),
                Handler::new(agent::add_main).filter(Command::one("+гл агент")),
                Handler::new(agent::delete_main).filter(Command::one("-гл агент")),
                Handler::new(agent::add_spam).filter(Command::one("+ас агент")),
                Handler::new(agent::delete_spam).filter(Command::one("-ас агент")),
                Handler::new(agent::add).filter(Command::one("+агент")),
                Handler::new(agent::delete).filter(Command::one("-агент")),
                Handler::new(ignore::add).filter(Command::one("+игнор")),
                Handler::new(ignore::delete_takeaway).filter(Command::one("-игнор ошибка")),
                Handler::new(ignore::delete).filter(Command::one("-игнор")),
                Handler::new(scam::add).filter(Command::one("+скам")),
                Handler::new(scam::delete).filter(Command::one("-скам")),
                Handler::new(spam::add).filter(Command::one("+ас")),
                Handler::new(spam::delete_takeaway).filter(Command::one("-ас ошибка")),
                Handler::new(spam::delete).filter(Command::one("-ас")),
                Handler::new(takeaway::delete_spam).filter(Command::many(&[
                    "-вынос ас",
                    "-вынос спам",
                    "-вынос антиспам",
                ])),
                Handler::new(takeaway::delete_ignore).filter(Command::one("-вынос игнор")),
                Handler::new(module::show_add_parent)
                    .filter(Command::one("+!модуль раздел").no_prefix()),
                Handler::new(module::add_parent).filter(Command::one("+модуль раздел")),
                Handler::new(module::show_add).filter(Command::one("+!модуль").no_prefix()),
                Handler::new(module::add).filter(Command::one("+модуль")),
            ])
        })
}
