use juzo_core::{
    common::{emojis::smail_sweets, inflection::plur_sweets},
    db::user::prelude::UserBalance,
    domain::AttachResult,
};
use sea_orm::{EntityTrait, QuerySelect, sea_query::Expr};

use super::super::*;

pub async fn bag(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(arch): Extension<AttachResult>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    let chat_ids = arch.chat_ids.0;

    let module = ModuleChecker::new(&bot, &db);
    let true = module
        .check::<32>(ModuleAccess::CustomM(&message, chat_ids))
        .await
    else {
        return Ok(());
    };

    // ради кубышки делать отдельную таблицу в БД? Мне что, делать нечего?
    let sweets = UserBalance::find_by_id(chat_ids)
        .select_only()
        .column_as(Expr::cust("TRUNC(sweets)::int"), "sweets")
        .into_tuple::<u32>()
        .one(&db)
        .await
        .ok()
        .flatten()
        .unwrap_or_default();

    bot.send(JuzoAnswer::message(&message).text(format!(
        "{0} В кубышке чата сейчас лежит {1}",
        smail_sweets(true),
        plur_sweets(sweets)
    )))
    .await?;

    Ok(())
}

pub async fn show_ids(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    let module = ModuleChecker::new(&bot, &db);
    let true = module
        .check::<37>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    bot.send(
        JuzoAnswer::message(&message)
            .text(format!("🗓 ID чата: <code>{0}</code>", message.chat().id())),
    )
    .await?;

    Ok(())
}
