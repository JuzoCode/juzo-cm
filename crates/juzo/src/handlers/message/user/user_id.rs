use juzo_core::{
    application::{UserIndex, UserModel},
    common::emojis::smail_tick,
};

use super::super::*;

pub async fn show(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    let user_ind = UserIndex::new(&bot, &db);
    let module = ModuleChecker::new(&bot, &db);

    // SAFETY: The Command filter will not allow processing of a "None" value.
    let text = unsafe {
        message
            .text()
            .or_else(|| message.caption())
            .unwrap_unchecked()
    };
    let args = result.args::<1>(text);

    let user: UserModel = match args {
        ArgsResult::Some([a1], _) => {
            let Ok(found_user) = user_ind
                .search_user(&text[a1])
                .await
            else {
                return Ok(());
            };
            found_user
        }
        // SAFETY: TBA will never return None in message.from().
        ArgsResult::None => unsafe {
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
        ArgsResult::Unk => return Ok(()),
    };

    if message
        .chat()
        .title()
        .is_some()
    {
        let true = module
            .check::<34>(ModuleAccess::M(&message))
            .await
        else {
            return Ok(());
        };
    }

    let bare_ids = user.ids.0;

    if bare_ids < 0 {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} <a href='{1}'>{2}</a>: <code>@_{3}</code>",
            smail_tick(true),
            user.link(),
            user.full_name(),
            -bare_ids,
        )))
        .await?;
    } else {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} <a href='{1}'>{2}</a>: <code>@{3}</code>",
            smail_tick(true),
            user.link(),
            user.full_name(),
            bare_ids,
        )))
        .await?;
    }

    Ok(())
}
