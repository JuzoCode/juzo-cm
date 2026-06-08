use juzo_core::filters::{Command, flood_wait::*};
use telers::{
    Filter, Router, enums::ChatType, event::telegram::Handler, filters::ChatType as FilterChatType,
};

mod chat_bag;
mod chat_description;
mod chat_name;
mod creator;
mod module_access;
mod pin;
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
                Handler::new(tg_admin::call)
                    .filter(FilterChatType::one(ChatType::Private).invert())
                    .filter(
                        Command::many(&[
                            "созвать тг-модеров",
                            "созвать тг модеров",
                            "созвать тг-админов",
                            "созвать тг админов",
                        ])
                        .no_prefix(),
                    )
                    .filter(FloodWait::new(FloodKind::CallModer, FloodType::User).second(180)),
                Handler::new(tag::add)
                    .filter(FilterChatType::one(ChatType::Private).invert())
                    .filter(Command::many(&["+тг тег", "+тг тэг"]).no_prefix()),
                Handler::new(tag::delete)
                    .filter(FilterChatType::one(ChatType::Private).invert())
                    .filter(Command::many(&["-тг тег", "-тг тэг"]).no_prefix()),
                Handler::new(chat_description::set)
                    .filter(FilterChatType::one(ChatType::Private).invert())
                    .filter(Command::one("+описание чата").no_prefix()),
                Handler::new(tg_admin::add)
                    .filter(FilterChatType::one(ChatType::Private).invert())
                    .filter(Command::one("+тг админ").no_prefix()),
                Handler::new(tg_admin::delete)
                    .filter(FilterChatType::one(ChatType::Private).invert())
                    .filter(Command::one("-тг админ").no_prefix()),
                Handler::new(pin::add)
                    .filter(FilterChatType::one(ChatType::Private).invert())
                    .filter(Command::many(&["пин", "закреп"])),
                Handler::new(pin::delete)
                    .filter(FilterChatType::one(ChatType::Private).invert())
                    .filter(Command::many(&["анпин", "открепить"])),
                Handler::new(creator::repair)
                    .filter(FilterChatType::one(ChatType::Private).invert())
                    .filter(Command::one("хв")),
                Handler::new(topic_name::set)
                    .filter(FilterChatType::one(ChatType::Private).invert())
                    .filter(Command::one("топик название")),
                Handler::new(topic_name::set)
                    .filter(FilterChatType::one(ChatType::Private).invert())
                    .filter(Command::one("название")),
                Handler::new(topic_reopen::set)
                    .filter(FilterChatType::one(ChatType::Private).invert())
                    .filter(Command::one("+топик").no_prefix()),
                Handler::new(topic_close::set)
                    .filter(FilterChatType::one(ChatType::Private).invert())
                    .filter(Command::one("-топик").no_prefix()),
                Handler::new(reason::scam)
                    .filter(FilterChatType::one(ChatType::Private).invert())
                    .filter(Command::one("скам причина").no_prefix()),
            ])
        })
        .on_business_message(|observer| {
            observer.registers([
                Handler::new(pin::add).filter(Command::many(&["пин", "закреп"]).no_prefix()),
                Handler::new(pin::delete)
                    .filter(Command::many(&["анпин", "открепить"]).no_prefix()),
            ])
        })
}
