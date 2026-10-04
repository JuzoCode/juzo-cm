use juzo_core::{
    application::{ParseTgLink, UserIndex, UserModel},
    common::emojis::smail_tick,
};
use telers::methods::SetChatMemberTag;

use super::super::*;

pub async fn add(
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
    let args = result.args::<9>(text);

    let (tag, user): (&str, UserModel) = match args {
        ArgsResult::Some(args, len) => {
            let last = args[len - 1];

            if let Some(link) = ParseTgLink::new(&text[last]) {
                let tag = &text[args[0].start..last.start];
                if tag.is_empty()
                    || tag
                        .chars()
                        .nth(16)
                        .is_some()
                {
                    return Ok(());
                }

                let Ok(found_user) = user_ind
                    .fetch_user(link)
                    .await
                else {
                    return Ok(());
                };

                (tag, found_user)
            } else {
                let tag = &text[args[0].start..last.end];
                if tag.is_empty()
                    || tag
                        .chars()
                        .nth(16)
                        .is_some()
                {
                    return Ok(());
                }

                let Some(reply) = message.reply_to_message() else {
                    return Ok(());
                };

                // SAFETY: TBA will never return None in message.from().
                let found_user = unsafe {
                    reply
                        .from()
                        .unwrap_unchecked()
                }
                .into();

                (tag, found_user)
            }
        }
        _ => return Ok(()),
    };

    let true = module
        .check::<22>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    bot.send(SetChatMemberTag::new(message.chat().id(), user.ids).tag(tag))
        .await?;

    bot.send(JuzoAnswer::message(&message).text(format!(
        "{0} Тег <a href='{1}'>{2}</a> изменён",
        smail_tick(true),
        user.link(),
        user.full_name(),
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
            } else {
                return Ok(());
            }
        },
        ArgsResult::Unk => return Ok(()),
    };

    let true = module
        .check::<22>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    bot.send(SetChatMemberTag::new(message.chat().id(), user.ids))
        .await?;

    bot.send(JuzoAnswer::message(&message).text(format!(
        "{0} Тег <a href='{1}'>{2}</a> удалён",
        smail_tick(true),
        user.link(),
        user.full_name(),
    )))
    .await?;

    Ok(())
}
