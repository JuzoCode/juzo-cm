use chrono::Utc;
use juzo_core::{
    application::{ParseTgLink, UserIndex, UserModel},
    common::{emojis::smail_pensil, tools::time::add_datetime},
};
use sea_orm::{ConnectionTrait, raw_sql};

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
    let args = result.args::<11>(text);
    let comment = &text[result.first_line];

    let (duration, user): (&str, UserModel) = match args {
        ArgsResult::Some(args, len) => {
            let last = args[len - 1];

            if let Some(link) = ParseTgLink::new(&text[last]) {
                let Ok(found_user) = user_ind
                    .fetch_user(link)
                    .await
                else {
                    return Ok(());
                };

                (&text[args[0].start..last.start], found_user)
            } else {
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

                (&text[args[0].start..], found_user)
            }
        }
        ArgsResult::None => {
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

            ("навсегда", found_user)
        }
        ArgsResult::Unk => return Ok(()),
    };

    // SAFETY: TBA will never return None in message.from().
    let iam: UserModel = unsafe {
        message
            .from()
            .unwrap_unchecked()
    }
    .into();

    if iam.ids == user.ids {
        return Ok(());
    }

    let true = module
        .check::<14>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    if comment
        .chars()
        .nth(64)
        .is_some()
    {
        bot.send(
            JuzoAnswer::message(&message)
                .text(format!("{0} Длина текста превышает 64 символов.", smail_pensil(true))),
        )
        .await?;
        return Ok(());
    }
    let chat_ids = message.chat().id();
    let now = Utc::now();

    let Some(delta) = add_datetime(now, duration) else {
        return Ok(());
    };

    let until = delta.timestamp();

    if until == 1 {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Уберите дубликаты периода варна, тогда мы сможем исполнить мечту <a \
             href='{1}'>{2}</a>.",
            smail_pensil(true),
            user.link(),
            user.full_name(),
        )))
        .await?;
        return Ok(());
    }

    let Ok(row) = db
        .query_one_raw(raw_sql!(
            Postgres,
            r#"
            WITH ranks AS (
                SELECT COALESCE(me.rank, 0) AS rank
                FROM c4 AS me
                LEFT JOIN c4 AS target
                    ON target.user_ids = {user.ids}
                AND target.chat_ids = {chat_ids}
                WHERE me.user_ids = {iam.ids}
                AND me.chat_ids = {chat_ids}
                AND COALESCE(me.rank, 0) > COALESCE(target.rank, 0)
            ),
            updated AS (
                UPDATE c9 AS c
                SET
                    moder_ids = {iam.ids},
                    reason = {comment},
                    removed = {until},
                    rank = r.rank
                FROM ranks AS r
                WHERE c.ctid = (
                    SELECT ctid
                    FROM c9
                    WHERE user_ids = {user.ids}
                    AND chat_ids = {chat_ids}
                    AND removed > 0
                    AND removed <= EXTRACT(EPOCH FROM NOW())::bigint
                    ORDER BY removed
                    LIMIT 1
                )
                RETURNING 1
            )
            INSERT INTO c9 (
                user_ids,
                chat_ids,
                moder_ids,
                reason,
                removed,
                rank
            )
            SELECT
                {user.ids},
                {chat_ids},
                {iam.ids},
                {comment},
                {until},
                rank
            FROM ranks
            WHERE NOT EXISTS (SELECT 1 FROM updated)
            RETURNING 1;
            "#
        ))
        .await
    else {
        return Ok(());
    };

    if row.is_none() {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Ваш ранг либо недостаточен, либо его вовсе не хватает.",
            smail_pensil(true)
        )))
        .await?;
        return Ok(());
    }

    bot.send(JuzoAnswer::message(&message).text("meow"))
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
    let args = result.args::<2>(text);

    let (count, user): (u8, UserModel) = match args {
        ArgsResult::Some([a1, a2], 2) => {
            let Ok(count) = text[a1].parse::<u8>() else {
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

    // SAFETY: TBA will never return None in message.from().
    let my_ids = unsafe {
        message
            .from()
            .unwrap_unchecked()
    }
    .id;

    if my_ids == user.ids.0 {
        return Ok(());
    }

    let true = module
        .check::<15>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    let chat_ids = message.chat().id();
    let Ok(Some(row)) = db
        .query_one_raw(raw_sql!(
            Postgres,
            r#"
            WITH target AS (
                SELECT
                    c9.ctid,
                    c9.rank,
                    coalesce(c4.rank, 0) AS my_rank
                FROM c9
                LEFT JOIN c4
                ON c4.user_ids = {my_ids}
                AND c4.chat_ids = {chat_ids}
                WHERE c9.user_ids = {user.ids}
                AND c9.chat_ids = {chat_ids}
            ),
            deleted AS (
                DELETE FROM c9
                WHERE ctid IN (
                    SELECT ctid
                    FROM target
                    WHERE my_rank >= rank
                    ORDER BY ctid
                    LIMIT NULLIF({count}, 1)
                )
                RETURNING 1
            )
            SELECT (SELECT count(*) FROM deleted) AS affected;
            "#
        ))
        .await
    else {
        return Ok(());
    };

    let affected = row
        .try_get::<i16>("", "affected")
        .unwrap_or(0);

    if affected == 0 {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} У <a href='{1}'>{2}</a> нету предупреждений.",
            smail_pensil(true),
            user.link(),
            user.full_name(),
        )))
        .await?;
        return Ok(());
    }

    // bot.send(JuzoAnswer::message(&message).text(format!("<a href='{1}'>{2}</a> ")))
    //     .await?;

    Ok(())
}
