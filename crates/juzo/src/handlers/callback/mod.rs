use juzo_core::filters::callback::{Callback, CallbackKind};
pub use telers::types::CallbackQuery;
use telers::{Router, event::telegram::Handler};

pub use super::{Bot, HandlerResult};

mod test;

pub fn routers() -> Router {
    Router::new("router CALLBACK connect").on_callback_query(|observer| {
        observer.on(Handler::new(test::ping).filter(Callback(CallbackKind::Ping)))
    })
}
