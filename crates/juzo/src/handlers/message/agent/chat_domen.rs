use juzo_core::{
    common::emojis::smail_tick,
    db::agent::{agent, prelude::Agent},
};
use sea_orm::{EntityTrait, QuerySelect};

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
    let my_ids = unsafe {
        message
            .from()
            .unwrap_unchecked()
    }
    .id;

    let Ok(Some(true)) = Agent::find_by_id(my_ids)
        .select_only()
        .column(agent::Column::Agent)
        .into_tuple::<bool>()
        .one(&db)
        .await
    else {
        return Ok(());
    };
    // SAFETY: The Command filter will not allow processing of a "None" value.
    let text = unsafe {
        message
            .text()
            .or_else(|| message.caption())
            .unwrap_unchecked()
    };
    let ArgsResult::Some([a1, a2], 2) = result.args::<2>(text) else {
        return Ok(());
    };

    let ru = &text[a1];
    let eng = &text[a2];

    bot.send(JuzoAnswer::message(&message).text(format!("{0} {ru} {eng}", smail_tick(true))))
        .await?;

    Ok(())
}
