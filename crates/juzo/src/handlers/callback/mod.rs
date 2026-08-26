use juzo_core::filters::callback::{Callback, CallbackKind};
pub use telers::types::CallbackQuery;
use telers::{Router, event::telegram::Handler};

pub use super::{Bot, DbConn, Extension, HandlerResult, ModuleAccess, ModuleChecker};

mod test;
mod trade;

pub fn routers() -> Router {
    Router::new("router CALLBACK connect").on_callback_query(|observer| {
        observer.registers([
            Handler::new(trade::reload).filter(Callback(CallbackKind::ReloadOrderBook)),
            Handler::new(test::ping).filter(Callback(CallbackKind::Ping)),
        ])
    })
}
