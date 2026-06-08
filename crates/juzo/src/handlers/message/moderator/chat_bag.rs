use juzo_core::{
    application::JuzoAnswer,
    common::emojis::smail_sweets,
    db::user::{balance, prelude::UserBalance},
};
use sea_orm::{DbConn, EntityTrait, QuerySelect, prelude::Decimal};

use super::super::*;

pub async fn show(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    // ради кубышки делать отдельную таблицу в БД? Мне что, делать нечего?
    let sweets = UserBalance::find_by_id(message.chat().id())
        .select_only()
        .column(balance::Column::Sweets)
        .into_tuple::<Decimal>()
        .one(&db)
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| Decimal::from(0));

    bot.send(
        JuzoAnswer::message(&message)
            .text(format!("{0} В кубышке чата сейчас лежит {sweets}", smail_sweets(true),)),
    )
    .await?;

    Ok(())
}
