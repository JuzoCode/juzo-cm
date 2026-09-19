use juzo_core::{
    common::emojis::{smail_cross, smail_tick},
    db::chat::{chat, prelude::Chat},
};
use sea_orm::{EntityTrait, Set, sea_query::OnConflict};

use super::super::*;

async fn edit_core(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
    enabled: bool,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    let module = ModuleChecker::new(&bot, &db);

    let true = module
        .check::<22>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    let model = chat::ActiveModel {
        chat_ids: Set(message
            .chat()
            .id()
            .into()),
        full_name: unsafe {
            Set(message
                .chat()
                .title()
                .unwrap_unchecked()
                .into())
        },
        spam: Set(enabled),
        ..Default::default()
    };

    let _ = Chat::insert(model)
        .on_conflict(
            OnConflict::column(chat::Column::ChatIds)
                .update_column(chat::Column::Spam)
                .to_owned(),
        )
        .exec(&db)
        .await;

    if !enabled {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Фильтр «Джузо-антиспам» включён",
            smail_tick(true)
        )))
        .await?;
    } else {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Фильтр «Джузо-антиспам» отключён",
            smail_cross(true)
        )))
        .await?;
    }

    Ok(())
}

pub async fn yes(
    bot: Bot,
    message: Message,
    db: Extension<DbConn>,
    result: Extension<CommandResult>,
) -> HandlerResult<()> {
    edit_core(bot, message, db, result, true).await
}

pub async fn no(
    bot: Bot,
    message: Message,
    db: Extension<DbConn>,
    result: Extension<CommandResult>,
) -> HandlerResult<()> {
    edit_core(bot, message, db, result, false).await
}
