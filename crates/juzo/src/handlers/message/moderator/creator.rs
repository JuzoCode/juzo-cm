use juzo_core::{
    // db::user::{profile, prelude::UserProfile},
    common::emojis::smail_tick,
};
use sea_orm::DbConn;

use super::super::*;

/// TODO
pub async fn repair(
    bot: Bot,
    message: Message,
    Extension(_db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    // SAFETY: TBA will never return None in message.from().
    let _user = unsafe {
        message
            .from()
            .unwrap_unchecked()
    };

    bot.send(
        JuzoAnswer::message(&message)
            .text(format!("{0} Основателю чата возвращены права владения", smail_tick(true))),
    )
    .await?;

    Ok(())
}
