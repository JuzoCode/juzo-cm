use core::fmt::Write;

use juzo_core::{
    application::{UserIndex, UserModel},
    common::emojis::smail_pensil,
    db::agent::{BlockFunc, block_system, prelude::BlockSystem},
};
use sea_orm::{DbConn, EntityTrait, QuerySelect, prelude::DateTimeUtc};

use super::super::*;

pub async fn info(
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

    let Ok(Some(reason)) = BlockSystem::find_by_id((user.ids, BlockFunc::AntiSpam))
        .select_only()
        .column(block_system::Column::Reason)
        .into_tuple::<String>()
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

    let mut text = format!("🗓 {0} находится в базе «Juzo | Anti-Spam»", user.ids);

    if !reason.is_empty() {
        let _ = write!(text, ".\n<blockquote expandable><b>Причина:</b> {reason}</blockquote>");
    }

    bot.send(JuzoAnswer::message(&message).text(text))
        .await?;

    Ok(())
}

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

    let Ok(Some((reason, time_add))) = BlockSystem::find_by_id((user.ids, BlockFunc::Scam))
        .select_only()
        .columns([block_system::Column::Reason, block_system::Column::TimeAdd])
        .into_tuple::<(String, DateTimeUtc)>()
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

    let fta = time_add.format("%d.%m.%Y");

    let mut text =
        format!("🗓 {0} находится в базе «Juzo | Scam System».\n<blockquote expandable>", user.ids);

    if !reason.is_empty() {
        let _ = write!(text, "<b>Причина:</b> {reason}\n<b>Добавлен:</b> {fta}</blockquote>");
    } else {
        let _ = write!(text, "<b>Добавлен:</b> {fta}</blockquote>",);
    }

    bot.send(JuzoAnswer::message(&message).text(text))
        .await?;

    Ok(())
}
