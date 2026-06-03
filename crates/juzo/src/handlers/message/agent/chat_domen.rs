use juzo_core::{application::UserIds, common::emojis::smail_tick, db::agent::prelude::Agent};
use sea_orm::{DbConn, EntityTrait, SelectExt};

use super::super::*;

pub async fn edit(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if result.args.is_empty() {
        return Ok(());
    };

    // SAFETY: TBA will never return None in message.from().
    let my_ids: UserIds = unsafe {
        message
            .from()
            .unwrap_unchecked()
            .id
            .into()
    };

    let Ok(exists) = Agent::find_by_id(my_ids)
        .exists(&db)
        .await
    else {
        return Ok(());
    };
    if !exists {
        return Ok(());
    }

    // SAFETY: The Command filter will not allow processing of a "None" value.
    let text = unsafe {
        message
            .text()
            .or_else(|| message.caption())
            .unwrap_unchecked()
    };
    let Some([a1, a2]) = result.args::<2>(text) else {
        return Ok(());
    };
    if a2.is_empty() {
        return Ok(());
    }

    let ru = &text[a1];
    let eng = &text[a2];

    bot.send(JuzoAnswer::message(&message).text(format!("{0} {ru} {eng}", smail_tick(true))))
        .await?;

    Ok(())
}
