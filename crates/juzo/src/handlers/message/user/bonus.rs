use juzo_core::application::{ParseTgLink, UserIndex, UserModel};
use telers::methods::SendMessage;

// use sea_orm::{ActiveValue::Set, EntityTrait, sea_query::OnConflict};
use super::super::*;

async fn buy_core<'a>(
    message: &'a Message,
    result: CommandResult,
    user_ind: UserIndex<'a>,
) -> Option<(u8, UserModel)> {
    let text = unsafe {
        message
            .text()
            .or_else(|| message.caption())
            .unwrap_unchecked()
    };
    let args = result.args::<2>(text);

    match args {
        ArgsResult::Some([a1, a2], 2) => {
            let Ok(value) = text[a1].parse() else {
                return None;
            };
            let Ok(found_user) = user_ind
                .search_user(&text[a2])
                .await
            else {
                return None;
            };
            Some((value, found_user))
        }
        ArgsResult::Some([a1, _], 1) => unsafe {
            if let Some(link) = ParseTgLink::new(&text[a1]) {
                let Ok(found_user) = user_ind
                    .fetch_user(link)
                    .await
                else {
                    return None;
                };
                Some((1, found_user))
            } else if !a1.is_empty() {
                let Ok(value) = text[a1].parse() else {
                    return None;
                };
                let found_user = if let Some(r) = message.reply_to_message() {
                    // SAFETY: TBA will never return None in message.from().
                    r.from()
                        .unwrap_unchecked()
                        .into()
                } else {
                    message
                        .from()
                        .unwrap_unchecked()
                        .into()
                };
                Some((value, found_user))
            } else {
                let found_user = if let Some(r) = message.reply_to_message() {
                    // SAFETY: TBA will never return None in message.from().
                    r.from()
                        .unwrap_unchecked()
                        .into()
                } else {
                    message
                        .from()
                        .unwrap_unchecked()
                        .into()
                };
                Some((1, found_user))
            }
        },
        // SAFETY: TBA will never return None in message.from().
        ArgsResult::None => unsafe {
            let found_user = if let Some(r) = message.reply_to_message() {
                r.from()
                    .unwrap_unchecked()
                    .into()
            } else {
                message
                    .from()
                    .unwrap_unchecked()
                    .into()
            };
            Some((1, found_user))
        },
        _ => return None,
    }
}

pub async fn buy_premium(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    let user_ind = UserIndex::new(&bot, &db);
    let module = ModuleChecker::new(&bot, &db);

    let Some((value, _)) = buy_core(&message, result, user_ind).await else {
        return Ok(());
    };

    let true = module
        .check::<47>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    bot.send(JuzoAnswer::<SendMessage>::to(&message).text(format!("{value}",)))
        .await?;

    Ok(())
}

pub async fn buy_vip(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    let user_ind = UserIndex::new(&bot, &db);
    let module = ModuleChecker::new(&bot, &db);

    let Some((_value, _user)) = buy_core(&message, result, user_ind).await else {
        return Ok(());
    };

    let true = module
        .check::<50>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    Ok(())
}
