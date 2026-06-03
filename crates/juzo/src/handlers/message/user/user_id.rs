use juzo_core::{
    application::{UserIndex, UserModel},
    common::emojis::smail_tick,
};
use sea_orm::DbConn;

use super::super::*;

pub async fn show(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    let user_ind = UserIndex::new(&bot, &db);

    // SAFETY: The Command filter will not allow processing of a "None" value.
    let text = unsafe {
        message
            .text()
            .or_else(|| message.caption())
            .unwrap_unchecked()
    };
    let args = result.args::<1>(text);

    let user: UserModel = match args {
        Some([a1]) => {
            let Ok(found_user) = user_ind
                .search_user(&text[a1])
                .await
            else {
                return Ok(());
            };
            found_user
        }
        None => unsafe {
            if let Some(r) = message.reply_to_message() {
                r.from()
                    .unwrap_unchecked()
                    .into()
            } else if message
                .business_connection_id()
                .is_some()
            {
                message.chat().into()
            } else {
                message
                    .from()
                    .unwrap_unchecked()
                    .into()
            }
        },
    };

    let smail = if user.ids.is_creator_bot() {
        "<tg-emoji emoji-id='5251236460169828505'>🎭</tg-emoji>"
    } else {
        smail_tick(true)
    };

    bot.send(JuzoAnswer::message(&message).text(format!(
        "{smail} {0}: <code>@{1}</code>",
        user.full_name(),
        user.ids,
    )))
    .await?;

    Ok(())
}
