use juzo_core::{
    application::{JuzoAnswer, ParseTgLink, UserIndex, UserModel},
    common::{
        emojis::{smail_pensil, smail_tick},
        inflection::{plur_deleted_a, plur_mark, plur_mark_y},
    },
    db::agent::{BlockFunc, agent, prelude::Agent},
};
use sea_orm::{ConnectionTrait, EntityTrait, QuerySelect, raw_sql};
use telers::types::ReplyParameters;

use super::super::*;

async fn delete_core(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
    function: BlockFunc,
) -> HandlerResult<()> {
    // SAFETY: TBA will never return None in message.from().
    let my_ids = unsafe {
        message
            .from()
            .unwrap_unchecked()
    }
    .id;

    let Ok(Some(true)) = Agent::find_by_id(my_ids)
        .select_only()
        .column(agent::Column::Spam)
        .into_tuple::<bool>()
        .one(&db)
        .await
    else {
        return Ok(());
    };

    let user_ind = UserIndex::new(&bot, &db);

    // SAFETY: The Command filter will not allow processing of a "None" value.
    let text = unsafe {
        message
            .text()
            .or_else(|| message.caption())
            .unwrap_unchecked()
    };
    let args = result.args::<2>(text);

    let (count, user): (u16, UserModel) = match args {
        ArgsResult::Some([a1, a2], 2) => {
            let Ok(count) = text[a1].parse::<u16>() else {
                return Ok(());
            };

            let Ok(found_user) = user_ind
                .search_user(&text[a2])
                .await
            else {
                return Ok(());
            };

            (count, found_user)
        }
        ArgsResult::Some([a1, _], 1) => unsafe {
            if let Some(link) = ParseTgLink::new(&text[a1]) {
                let Ok(found_user) = user_ind
                    .fetch_user(link)
                    .await
                else {
                    return Ok(());
                };

                (1, found_user)
            } else {
                let found_user = if let Some(r) = message.reply_to_message() {
                    // SAFETY: TBA will never return None in message.from().
                    r.from()
                        .unwrap_unchecked()
                        .into()
                } else if message
                    .business_connection_id()
                    .is_some()
                {
                    message.chat().into()
                } else {
                    return Ok(());
                };

                (1, found_user)
            }
        },
        // SAFETY: TBA will never return None in message.from().
        ArgsResult::None => unsafe {
            let found_user = if let Some(r) = message.reply_to_message() {
                r.from()
                    .unwrap_unchecked()
                    .into()
            } else if message
                .business_connection_id()
                .is_some()
            {
                message.chat().into()
            } else {
                return Ok(());
            };

            (1, found_user)
        },
        _ => return Ok(()),
    };

    let Ok(row) = db
        .execute_raw(raw_sql!(
            Postgres,
            r#"
            DELETE FROM a2
            WHERE ids IN (
                SELECT ids
                FROM a2
                WHERE user_ids = {user.ids}
                    AND removed >= EXTRACT(EPOCH FROM NOW()) - 172800
                    AND function = {function}
                ORDER BY removed DESC
                LIMIT NULLIF({count}, 0)
            );
            "#
        ))
        .await
    else {
        return Ok(());
    };

    let affected = row.rows_affected();
    let f = if function == BlockFunc::Spam {
        "Juzo | Anti-Spam"
    } else {
        "Juzo | Ignore System"
    };

    if affected == 0 {
        bot.send(
            JuzoAnswer::message(&message)
                .text(format!("{0} Нет ни одной пометки «{f}» для удаления.", smail_pensil(true))),
        )
        .await?;
        return Ok(());
    }

    bot.send(JuzoAnswer::message(&message).text(format!(
        "{0} У <a href='{1}'>{2}</a> {3} {4} «{f}»",
        smail_tick(true),
        user.link(),
        user.full_name(),
        plur_deleted_a(affected).text,
        plur_mark(affected)
    )))
    .await?;

    let _ = bot
        .send(
            JuzoAnswer::message(&message)
                .text(format!("🗓 Вам удалили {0} «{f}»", plur_mark_y(affected)))
                .chat_id(user.ids.0)
                .business_connection_id_option::<&str>(None)
                .reply_parameters_option::<ReplyParameters>(None),
        )
        .await;

    Ok(())
}

pub async fn delete_ignore(
    bot: Bot,
    message: Message,
    db: Extension<DbConn>,
    result: Extension<CommandResult>,
) -> HandlerResult<()> {
    delete_core(bot, message, db, result, BlockFunc::Ignore).await
}

pub async fn delete_spam(
    bot: Bot,
    message: Message,
    db: Extension<DbConn>,
    result: Extension<CommandResult>,
) -> HandlerResult<()> {
    delete_core(bot, message, db, result, BlockFunc::Spam).await
}
