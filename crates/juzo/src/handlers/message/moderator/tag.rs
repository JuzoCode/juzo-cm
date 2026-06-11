use juzo_core::{
    application::{UserIndex, UserModel},
    common::emojis::smail_tick,
};
use sea_orm::DbConn;
use telers::methods::SetChatMemberTag;

use super::super::*;

pub async fn add(
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
    let args = &text[result.args];

    let (tag, user): (&str, UserModel) = match (args.is_empty(), message.reply_to_message()) {
        (_, Some(reply)) => {
            if args.is_empty()
                || args
                    .chars()
                    .nth(16)
                    .is_some()
            {
                return Ok(());
            }

            // SAFETY: Branching does not allow processing the None.
            unsafe {
                (
                    args,
                    reply
                        .from()
                        .unwrap_unchecked()
                        .into(),
                )
            }
        }
        (false, _) => {
            let (tag, user_line) = args
                .rsplit_once(char::is_whitespace)
                .unwrap_or(("", args));
            if tag.is_empty()
                || tag
                    .chars()
                    .nth(16)
                    .is_some()
            {
                return Ok(());
            }

            let Ok(found_user) = user_ind
                .search_user(user_line)
                .await
            else {
                return Ok(());
            };

            (tag, found_user)
        }
        _ => return Ok(()),
    };

    bot.send(SetChatMemberTag::new(message.chat().id(), user.ids).tag(tag))
        .await?;

    bot.send(JuzoAnswer::message(&message).text(format!(
        "{0} Тег {1} был успешно изменён",
        smail_tick(true),
        user.ids
    )))
    .await?;

    Ok(())
}

pub async fn delete(
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
            } else {
                return Ok(());
            }
        },
    };

    bot.send(SetChatMemberTag::new(message.chat().id(), user.ids))
        .await?;

    bot.send(JuzoAnswer::message(&message).text(format!(
        "{0} Тег {1} был успешно удалён",
        smail_tick(true),
        user.ids
    )))
    .await?;

    Ok(())
}
