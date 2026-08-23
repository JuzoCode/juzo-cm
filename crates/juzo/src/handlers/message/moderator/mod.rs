use juzo_core::filters::Command;
use telers::{Router, event::telegram::Handler};

mod ban;
mod chat_bag;
mod chat_description;
mod chat_name;
mod creator;
mod module_access;
mod pin;
mod rank;
mod reason;
mod tag;
mod tg_admin;
mod topic_close;
mod topic_name;
mod topic_reopen;

pub fn routers() -> Router {
    Router::new("router MODER connect")
        .on_message(|observer| {
            observer.registers([
                Handler::new(ban::no).filter(Command::many(&["разбан"]).no_prefix()),
                Handler::new(ban::yes).filter(Command::many(&["бан"]).no_prefix()),
                Handler::new(tag::add).filter(Command::many(&["+тг тег", "+тг тэг"]).no_prefix()),
                Handler::new(tag::delete)
                    .filter(Command::many(&["-тг тег", "-тг тэг"]).no_prefix()),
                Handler::new(chat_description::set)
                    .filter(Command::one("+описание чата").no_prefix()),
                Handler::new(tg_admin::add).filter(Command::one("+тг админ").no_prefix()),
                Handler::new(tg_admin::delete).filter(Command::one("-тг админ").no_prefix()),
                Handler::new(pin::add).filter(Command::many(&["пин", "закреп"])),
                Handler::new(pin::delete).filter(Command::many(&["анпин", "открепить"])),
                Handler::new(creator::repair).filter(Command::one("хв")),
                Handler::new(topic_name::set).filter(Command::one("топик название")),
                Handler::new(topic_name::set).filter(Command::one("название")),
                Handler::new(topic_reopen::set).filter(Command::one("+топик").no_prefix()),
                Handler::new(topic_close::set).filter(Command::one("-топик").no_prefix()),
                Handler::new(reason::scam).filter(Command::one("скам причина").no_prefix()),
                Handler::new(reason::info).filter(Command::one("причина").no_prefix()),
            ])
        })
        .on_business_message(|observer| {
            observer.registers([
                Handler::new(pin::add).filter(Command::many(&["пин", "закреп"])),
                Handler::new(pin::delete).filter(Command::many(&["анпин", "открепить"])),
                Handler::new(reason::scam).filter(Command::one("скам причина")),
            ])
        })
}
