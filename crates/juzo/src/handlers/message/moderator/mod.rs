use juzo_core::filters::{Attach, Command};
use telers::{Filter, Router, enums, event::telegram::Handler, filters::ChatType};

mod ban;
mod chat;
mod chat_description;
mod chat_name;
mod creator;
mod module_access;
mod mute;
mod pin;
mod rank;
mod reason;
mod tag;
mod tg_admin;
mod topic_close;
mod topic_name;
mod topic_reopen;
#[cfg(debug_assertions)]
mod warns;

pub fn routers() -> Router {
    Router::new("router MODER connect")
        .on_message(|observer| {
            observer.registers([
                Handler::new(module_access::edit_show_false)
                    .filter(Command::many(&["-дм"]).no_prefix())
                    .filter(Attach),
                Handler::new(module_access::edit_show_true)
                    .filter(Command::many(&["+дм"]).no_prefix())
                    .filter(Attach),
                #[cfg(debug_assertions)]
                Handler::new(warns::delete)
                    .filter(Command::many(&["-варн"]).no_prefix())
                    .filter(ChatType::one(enums::ChatType::Private).invert()),
                Handler::new(ban::no)
                    .filter(Command::many(&["разбан"]).no_prefix())
                    .filter(Attach),
                Handler::new(ban::yes)
                    .filter(Command::many(&["бан", "чс"]).no_prefix())
                    .filter(Attach),
                Handler::new(chat::bag)
                    .filter(Command::one("кубышка").no_prefix())
                    .filter(Attach),
                Handler::new(chat::show_ids)
                    .filter(Command::one("чат ид"))
                    .filter(ChatType::one(enums::ChatType::Private).invert()),
                Handler::new(mute::yes)
                    .filter(Command::many(&["мут"]).no_prefix())
                    .filter(Attach),
                Handler::new(tag::add)
                    .filter(Command::many(&["+тг тег", "+тг тэг"]).no_prefix())
                    .filter(ChatType::one(enums::ChatType::Private).invert()),
                Handler::new(tag::delete)
                    .filter(Command::many(&["-тг тег", "-тг тэг"]).no_prefix())
                    .filter(ChatType::one(enums::ChatType::Private).invert()),
                Handler::new(chat_description::set)
                    .filter(Command::one("+описание чата").no_prefix())
                    .filter(ChatType::one(enums::ChatType::Private).invert()),
                Handler::new(chat_description::delete)
                    .filter(Command::one("-описание чата").no_prefix())
                    .filter(ChatType::one(enums::ChatType::Private).invert()),
                Handler::new(tg_admin::add)
                    .filter(Command::one("+тг админ").no_prefix())
                    .filter(ChatType::one(enums::ChatType::Private).invert()),
                Handler::new(tg_admin::delete)
                    .filter(Command::one("-тг админ").no_prefix())
                    .filter(ChatType::one(enums::ChatType::Private).invert()),
                Handler::new(pin::add)
                    .filter(Command::many(&["пин", "закреп"]))
                    .filter(ChatType::one(enums::ChatType::Private).invert()),
                Handler::new(pin::delete)
                    .filter(Command::many(&["анпин", "открепить"]))
                    .filter(ChatType::one(enums::ChatType::Private).invert()),
                #[cfg(debug_assertions)]
                Handler::new(warns::add)
                    .filter(Command::many(&["варн"]).no_prefix())
                    .filter(ChatType::one(enums::ChatType::Private).invert()),
                Handler::new(creator::repair)
                    .filter(Command::many(&["восстановить создателя", "хв"]).no_prefix())
                    .filter(ChatType::one(enums::ChatType::Private).invert()),
                Handler::new(topic_name::set)
                    .filter(Command::one("топик название"))
                    .filter(ChatType::one(enums::ChatType::Private).invert()),
                Handler::new(chat_name::set)
                    .filter(Command::one("название"))
                    .filter(ChatType::one(enums::ChatType::Private).invert()),
                Handler::new(topic_reopen::set)
                    .filter(Command::one("+топик").no_prefix())
                    .filter(ChatType::one(enums::ChatType::Private).invert()),
                Handler::new(topic_close::set)
                    .filter(Command::one("-топик").no_prefix())
                    .filter(ChatType::one(enums::ChatType::Private).invert()),
                Handler::new(reason::scam).filter(
                    Command::many(&["скам причина", "причина скама", "причина скам"]).no_prefix(),
                ),
                Handler::new(reason::info_mute)
                    .filter(Command::many(&["причина мута", "проверить мут"]).no_prefix())
                    .filter(Attach),
                Handler::new(reason::info)
                    .filter(Command::one("причина").no_prefix())
                    .filter(Attach),
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
