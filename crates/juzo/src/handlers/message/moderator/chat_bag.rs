use juzo_core::{
    common::{emojis::smail_sweets, inflection::plur_sweets},
    db::user::prelude::UserBalance,
};
use sea_orm::{DbConn, EntityTrait, QuerySelect, sea_query::Expr};

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
