use juzo_core::{
    application::{ParseTgLink, UserIndex, UserModel},
    common::emojis::{smail_cross, smail_tick},
};
use telers::methods::PromoteChatMember;

use super::super::*;

// ДОДЕЛАТЬ
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
    let args = result.args::<17>(text);

    let (_, user): (&str, UserModel) = match args {
        ArgsResult::Some(args, len) => {
            let last = args[len - 1];

            let (tag, link) = if let Some(link) = ParseTgLink::new(&text[last]) {
                (&text[args[0].start..last.start], Some(link))
            } else {
                (&text[args[0].start..last.end], None)
            };
            if tag.is_empty()
                || tag
                    .chars()
                    .nth(16)
                    .is_some()
            {
                return Ok(());
            }

            if let Some(link) = link {
                let Ok(found_user) = user_ind
                    .fetch_user(link)
                    .await
                else {
                    return Ok(());
                };
                (tag, found_user)
            } else {
                let Some(reply) = message.reply_to_message() else {
                    return Ok(());
                };
                // SAFETY: TBA will never return None in message.from().
                (tag, unsafe {
                    reply
                        .from()
                        .unwrap_unchecked()
                        .into()
                })
            }
        }
        ArgsResult::None => {
            let Some(reply) = message.reply_to_message() else {
                return Ok(());
            };
            // SAFETY: TBA will never return None in message.from().
            ("", unsafe {
                reply
                    .from()
                    .unwrap_unchecked()
                    .into()
            })
        }
        ArgsResult::Unk => return Ok(()),
    };

    let true = module
        .check::<23>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    // match juzo.add_tg_admin(user_id).await {
    //     Ok(_) => {
    //         let _ = juzo.bot.set_chat_administrator_custom_title(
    //             juzo.message.chat.id, user_id, custom_title,
    //         ).await?;
    //     },
    //     Err(_) => return
    // }

    bot.send(JuzoAnswer::message(&message).text(format!(
        "{0} <a href='{1}'>{2}</a> успешно назначен тг-администратором<a \
         href='tg://user?id={3}'>\u{2069}</a>",
        smail_tick(true),
        user.link(),
        user.full_name(),
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
        .check::<23>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    bot.send(PromoteChatMember::new(message.chat().id(), user.ids).can_manage_chat(false))
        .await?;

    bot.send(JuzoAnswer::message(&message).text(format!(
        "{0} <a href='{1}'>{2}</a> исключён из тг-администраторов",
        smail_cross(true),
        user.link(),
        user.full_name(),
    )))
    .await?;

    Ok(())
}
