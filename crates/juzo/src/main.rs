#![allow(dead_code)]
#![forbid(unused_unsafe)]
#![forbid(unused_imports)]

use sea_orm::ConnectionTrait;
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

    db.execute_unprepared(
        r#"
        INSERT INTO a (user_ids, show, add_agent, agent, spam)
        VALUES (392851555, false, true, true, true)
        ON CONFLICT (user_ids) DO NOTHING;
        "#,
    )
    .await
    .unwrap();

    let bot = Bot::new(configs.production_or_test());
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
