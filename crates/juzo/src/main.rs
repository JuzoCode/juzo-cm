#![allow(dead_code)]

use telers::{Bot, Dispatcher, Router, methods::DeleteWebhook};

mod handlers;

pub use juzo_core::{
    config::{ALLOWEDS_UPDATES, load_config},
    db,
};

#[tokio::main]
async fn main() {
    let configs = load_config();

    let db = db::connect(
        configs
            .db_conn
            .to_owned(),
    )
    .await
    .expect("error DB");

    let bot = Bot::new(configs.test);
    bot.send(DeleteWebhook::new().drop_pending_updates(true))
        .await
        .unwrap();

    let router = Router::new("main").include(handlers::routers_connect());

    let dispatcher = Dispatcher::builder()
        .allowed_updates(ALLOWEDS_UPDATES)
        .main_router(router.configure_default())
        .polling_timeout(10)
        .extension(db)
        .bot(bot)
        .build();

    println!("Meow");
    let _ = dispatcher
        .run_polling()
        .await;
}
