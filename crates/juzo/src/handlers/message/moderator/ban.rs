use core::fmt::Write;

use chrono::Utc;
use juzo_core::{
    application::{ParseTgLink, UserIndex, UserModel},
    common::{
        emojis::{smail_pensil, smail_tick},
        tools::time::add_datetime,
    },
    domain::{AttachResult, TimeFormatted},
    middlewares::inner::MemberTraffic,
};
use sea_orm::{ConnectionTrait, raw_sql};
use telers::{
    methods::{BanChatMember, BanChatSenderChat, GetChatMember, UnbanChatMember},
    types::{ChatMemberLeft, User},
};

use super::super::*;

pub async fn yes(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(arch): Extension<AttachResult>, // не спрашивайте почему
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
    let chat_ids = arch.chat_ids.0;

    // Из-за него пришлось делать новый формат и перепридумывать велосипед парсинга аргии
    // Из-за него появилась идея парсить комбинированные аргию,
    // просто убрав абстракцию search_user(user_line: &'a str) и сделав парсинг ссылок прямо тут
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
        .check::<9>(ModuleAccess::CustomM(&message, chat_ids))
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

    let sms_ids = message.message_id();
    let now = Utc::now();

    let Some(delta) = add_datetime(now, duration) else {
        return Ok(());
    };

    let now_ts = now.timestamp();
    let until = delta.timestamp();

    if until == 1 {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Уберите дубликаты периода бана, тогда мы сможем исполнить мечту <a \
             href='{1}'>{2}</a>.",
            smail_pensil(true),
            user.link(),
            user.full_name(),
        )))
        .await?;
        return Ok(());
    }

    let member = bot
        .send(GetChatMember::new(chat_ids, user.ids))
        .await
        .unwrap_or_else(|_| {
            ChatMemberLeft::new(User::new(392851555, false, "Hello, Juzo Code")).into()
        });

    let to_return = MemberTraffic::state(&member) == 1;

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
                to_return,
                rank
            )
            SELECT
                {user.ids},
                {chat_ids},
                true,
                {iam.ids},
                {sms_ids},
                {comment},
                {now_ts},
                {until},
                {to_return},
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

    let tg_ban = if user.ids.0 < 0 {
        bot.send(BanChatSenderChat::new(chat_ids, user.ids))
            .await
            .unwrap_or_default()
    } else {
        bot.send(BanChatMember::new(chat_ids, user.ids))
            .await
            .unwrap_or_default()
    };

    let mut text = String::with_capacity(2048);

    let _ = write!(text, "🔴 <a href='{0}'>{1}</a> получает бан ", user.link(), user.full_name());

    if until == 0 {
        text.push_str("навсегда");
    } else {
        let _ = write!(text, "на {0}", TimeFormatted::until(until, now));
    }

    if !tg_ban {
        text.push_str(", <b>без тг-бана</b>");
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
        .check::<10>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    let chat_ids = message.chat().id();

    let member = bot
        .send(GetChatMember::new(chat_ids, user.ids))
        .await
        .unwrap_or_else(|_| {
            ChatMemberLeft::new(User::new(392851555, false, "Hello, Juzo Code")).into()
        });

    // SAFETY: TBA will never return None in message.from().
    let my_ids = unsafe {
        message
            .from()
            .unwrap_unchecked()
    }
    .id;
    let state = MemberTraffic::state(&member) == 1;
    let Ok(Some(row)) = db
        .query_one_raw(raw_sql!(
            Postgres,
            r#"
            WITH target AS (
                SELECT
                    c8.rank,
                    c8.moder_ids,
                    coalesce(c4.rank, 0) AS my_rank
                FROM c8
                LEFT JOIN c4
                ON c4.user_ids = {my_ids}
                AND c4.chat_ids = {chat_ids}
                WHERE c8.user_ids = {user.ids}
                AND c8.chat_ids = {chat_ids}
                AND c8.is_ban
            ),
            deleted AS (
                DELETE FROM c8
                USING target
                WHERE c8.user_ids = {user.ids}
                AND c8.chat_ids = {chat_ids}
                AND c8.is_ban
                AND (
                    target.moder_ids = {my_ids}
                    OR target.rank <= target.my_rank
                )
                RETURNING 1
            )
            SELECT CASE
                WHEN EXISTS (SELECT 1 FROM deleted) THEN 1
                WHEN EXISTS (SELECT 1 FROM target) THEN 0
                ELSE 2
            END AS affected;
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
            "{0} Чтобы разбанить <a href='{1}'>{2}</a>, вашего ранга не хватает.",
            smail_pensil(true),
            user.link(),
            user.full_name(),
        )))
        .await?;
        return Ok(());
    }

    bot.send(UnbanChatMember::new(chat_ids, user.ids.0).only_if_banned(true))
        .await?;

    match (affected, state) {
        (1, false) => {
            bot.send(JuzoAnswer::message(&message).text(format!(
                "{0} <a href='{1}'>{2}</a> разбанен. Теперь можно добавить его в чат или <b>снова \
                 забанить</b> =)",
                smail_tick(true),
                user.link(),
                user.full_name(),
            )))
            .await?;
        }
        (1, true) => {
            bot.send(JuzoAnswer::message(&message).text(format!(
                "🗓 <a href='{0}'>{1}</a> исключён из бан-листа, но уже находится в чате",
                user.link(),
                user.full_name(),
            )))
            .await?;
        }
        (_, true) => {
            bot.send(JuzoAnswer::message(&message).text(format!(
                "{0} <a href='{1}'>{2}</a> не забанен и уже находится в чате.",
                smail_pensil(true),
                user.link(),
                user.full_name(),
            )))
            .await?;
        }
        _ => {
            bot.send(JuzoAnswer::message(&message).text(format!(
                "{0} <a href='{1}'>{2}</a> не забанен.\n<blockquote>Но было выполнено вынесение \
                 из черного списка в Телеграм</blockquote>",
                smail_pensil(true),
                user.link(),
                user.full_name(),
            )))
            .await?;
        }
    }

    Ok(())
}
