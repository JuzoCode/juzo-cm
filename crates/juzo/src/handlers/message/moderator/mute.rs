use core::fmt::Write;

use chrono::Utc;
use juzo_core::{
    application::{ParseTgLink, UserIndex, UserModel},
    common::{emojis::smail_pensil, tools::time::add_datetime},
    db::chat::prelude::ChatBlock,
    domain::TimeFormatted,
};
use sea_orm::{ConnectionTrait, EntityTrait, raw_sql};

use super::super::*;

pub async fn yes(
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
        .check::<19>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    if comment
        .chars()
        .nth(128)
        .is_some()
    {
        bot.send(
            JuzoAnswer::message(&message)
                .text(format!("{0} Длина текста превышает 128 символов.", smail_pensil(true))),
        )
        .await?;
        return Ok(());
    }

    let chat_ids = message.chat().id();
    let sms_ids = message.message_id();

    let now = Utc::now();

    let Some(delta) = add_datetime(now, duration) else {
        return Ok(());
    };

    let now_ts = now.timestamp();
    let until = delta.timestamp();

    if until == 1 {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Уберите дубликаты периода мута, тогда мы сможем исполнить мечту <a \
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
            INSERT INTO c8 (
                user_ids,
                chat_ids,
                is_ban,
                moder_ids,
                sms_ids,
                reason,
                added,
                removed,
                rank
            )
            SELECT
                {user.ids},
                {chat_ids},
                false,
                {iam.ids},
                {sms_ids},
                {comment},
                {now_ts},
                {until},
                COALESCE(me.rank, 0)
            FROM c4 AS me
            LEFT JOIN c4 AS target
                ON target.chat_ids = me.chat_ids
                AND target.user_ids = {user.ids}
            WHERE me.user_ids = {iam.ids}
                AND me.chat_ids = {chat_ids}
                AND COALESCE(me.rank, 0) > COALESCE(target.rank, 0)
            ON CONFLICT (user_ids, chat_ids, is_ban)
            DO UPDATE SET
                moder_ids = EXCLUDED.moder_ids,
                sms_ids = EXCLUDED.sms_ids,
                reason = EXCLUDED.reason,
                added = EXCLUDED.added,
                removed = EXCLUDED.removed,
                rank = EXCLUDED.rank
            WHERE c8.rank <= EXCLUDED.rank
            RETURNING removed;
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

    let mut text = String::with_capacity(2048);

    let _ = write!(text, "🔴 <a href='{0}'>{1}</a> получает мут ", user.link(), user.full_name());

    if until == 0 {
        text.push_str("навсегда");
    } else {
        let _ = write!(text, "на {0}", TimeFormatted::until(until, now));
    }

    let _ = writeln!(
        text,
        ".<blockquote expandable><b>Модератор: </b><a href='{0}'>{1}</a>",
        iam.link(),
        iam.full_name()
    );

    if !comment.is_empty() {
        text.push_str("<b>Причина: </b>");
        text.push_str(comment);
    }

    text.push_str("</blockquote>");

    bot.send(JuzoAnswer::message(&message).text(text))
        .await?;

    Ok(())
}

pub async fn no(
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
        .check::<20>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    let chat_ids = message.chat().id();

    let _res = ChatBlock::delete_by_id((user.ids, chat_ids.into(), false))
        .exec(&db)
        .await;

    Ok(())
}
