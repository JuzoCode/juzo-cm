use juzo_core::{
    application::{UserIndex, UserModel},
    common::emojis::smail_pensil,
    db::user::prelude::UserAnketa,
};
use sea_orm::{DbConn, EntityTrait};

use super::super::*;

/// добавить в бизнес мод
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

    let Ok(Some(anketa)) = UserAnketa::find_by_id(user.ids)
        .one(&db)
        .await
    else {
        bot.send(
            JuzoAnswer::message(&message)
                .text(format!("{0} О данном пользователе ни слуху, ни духу.", smail_pensil(true))),
        )
        .await?;
        return Ok(());
    };

    if !anketa.show {
        bot.send(
            JuzoAnswer::message(&message)
                .text(format!("{0} Анкета же скрыта от незрелых глазок.", smail_pensil(true))),
        )
        .await?;
        return Ok(());
    }

    bot.send(JuzoAnswer::message(&message).text("Анкета есть"))
        .await?;

    Ok(())
}
