use juzo_core::{
    application::{UserIndex, UserModel},
    common::emojis::smail_pensil,
    db::agent::{BlockFunc, block_system, prelude::BlockSystem},
};
use sea_orm::{DbConn, EntityTrait, QuerySelect, prelude::DateTimeUtc};

use super::super::*;

pub async fn scam(
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

    let user: UserModel = match (args, message.reply_to_message()) {
        (Some(spans), _) => {
            let Ok(found_user) = user_ind
                .search_user(&text[spans[0]])
                .await
            else {
                return Ok(());
            };
            found_user
        }
        // SAFETY: Branching does not allow processing the None.
        (None, Some(reply)) => unsafe {
            reply
                .from()
                .unwrap_unchecked()
                .into()
        },
        _ => return Ok(()),
    };

    let Ok(Some(info)) = BlockSystem::find_by_id((user.ids, BlockFunc::Scam))
        .select_only()
        .columns([
            block_system::Column::AgentsIds,
            block_system::Column::Reason,
            block_system::Column::TimeAdd,
        ])
        .into_tuple::<(i64, String, DateTimeUtc)>()
        .one(&db)
        .await
    else {
        bot.send(
            JuzoAnswer::message(&message)
                .text(format!("{0} Пользователь не находится в базе скама.", smail_pensil(true))),
        )
        .await?;
        return Ok(());
    };

    bot.send(JuzoAnswer::message(&message).text(format!("{0} => {1:?}", user.ids, info)))
        .await?;

    Ok(())
}

pub async fn spam(
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

    let user: UserModel = match (args, message.reply_to_message()) {
        (Some(spans), _) => {
            let Ok(found_user) = user_ind
                .search_user(&text[spans[0]])
                .await
            else {
                return Ok(());
            };
            found_user
        }
        // SAFETY: Branching does not allow processing the None.
        (None, Some(reply)) => unsafe {
            reply
                .from()
                .unwrap_unchecked()
                .into()
        },
        _ => return Ok(()),
    };

    let Ok(Some(info)) = BlockSystem::find_by_id((user.ids, BlockFunc::AntiSpam))
        .select_only()
        .columns([
            block_system::Column::AgentsIds,
            block_system::Column::Reason,
            block_system::Column::TimeAdd,
        ])
        .into_tuple::<(i64, String, DateTimeUtc)>()
        .one(&db)
        .await
    else {
        bot.send(
            JuzoAnswer::message(&message)
                .text(format!("{0} Пользователь не находится в базе спама.", smail_pensil(true))),
        )
        .await?;
        return Ok(());
    };

    bot.send(JuzoAnswer::message(&message).text(format!("{0} => {1:?}", user.ids, info)))
        .await?;

    Ok(())
}
