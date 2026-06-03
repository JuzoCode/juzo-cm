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
mod scam;
mod spam;

pub fn routers() -> Router {
    Router::new("router AGENT connect")
        .on_message(|observer| {
            observer.registers([
                Handler::new(managed_bot::info).filter(Command::one("клон инфо")),
                Handler::new(managed_bot::delete).filter(Command::one("-клон").no_prefix()),
                Handler::new(managed_bot::set_official)
                    .filter(Command::one("+офф джузо клон").no_prefix()),
                Handler::new(agent::add).filter(Command::one("+агент").no_prefix()),
                Handler::new(agent::delete).filter(Command::one("-агент").no_prefix()),
                Handler::new(chat_domen::edit).filter(Command::one("домен")),
                Handler::new(ignore::add).filter(Command::one("+игнор").no_prefix()),
                Handler::new(ignore::delete_takeaway)
                    .filter(Command::one("-игнор ошибка").no_prefix()),
                Handler::new(ignore::delete).filter(Command::one("-игнор").no_prefix()),
                Handler::new(scam::add).filter(Command::one("+скам").no_prefix()),
                Handler::new(scam::delete).filter(Command::one("-скам").no_prefix()),
                Handler::new(spam::add).filter(Command::one("+ас").no_prefix()),
                Handler::new(spam::delete_takeaway).filter(Command::one("-ас ошибка").no_prefix()),
                Handler::new(spam::delete).filter(Command::one("-ас").no_prefix()),
            ])
        })
        .on_business_message(|observer| {
            observer.registers([
                Handler::new(managed_bot::info).filter(Command::one("клон инфо")),
                Handler::new(managed_bot::delete).filter(Command::one("-клон")),
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
            ])
        })
}
