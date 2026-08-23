use core::fmt::Write;

use chrono::Utc;
use juzo_core::{
    application::{ParseTgLink, UserIndex, UserModel},
    common::{
        emojis::{smail_pensil, smail_tick},
        tools::time::add_datetime,
    },
    db::chat::prelude::ChatBlock,
    domain::TimeFormatted,
    middlewares::inner::MemberTraffic,
};
use sea_orm::{ConnectionTrait, EntityTrait, raw_sql};
use telers::{
    methods::{BanChatMember, GetChatMember, UnbanChatMember},
    types::{ChatMemberLeft, User},
};

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
    let reason = &text[result.first_line];

    // Из-за него пришлось делать новый формат и перепридумывать велосипед парсинга аргии
    // Из-за него появилась идея парсить комбинированные аргию,
    // просто убрав абстракцию search_user(user_line: &'a str) и сделав парсинг ссылок прямо тут
    let (duration, user): (&str, UserModel) = match args {
        ArgsResult::Some(args, len) => {
            let last = args[len - 1];

            if let Some(link) = ParseTgLink::new(&text[last]) {
                let Ok(user) = user_ind
                    .fetch_user(link)
                    .await
                else {
                    return Ok(());
                };

                (&text[args[0].start..last.start], user)
            } else {
                let Some(reply) = message.reply_to_message() else {
                    return Ok(());
                };

                // SAFETY: TBA will never return None in message.from().
                let user = unsafe {
                    reply
                        .from()
                        .unwrap_unchecked()
                }
                .into();

                (&text[args[0].start..], user)
            }
        }
        ArgsResult::None => {
            let Some(reply) = message.reply_to_message() else {
                return Ok(());
            };

            // SAFETY: TBA will never return None in message.from().
            let user = unsafe {
                reply
                    .from()
                    .unwrap_unchecked()
            }
            .into();

            ("навсегда", user)
        }
        ArgsResult::Unk => return Ok(()),
    };

    let access = module
        .check::<9>(ModuleAccess::M(&message))
        .await;
    if !access {
        return Ok(());
    }

    if reason
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

    // SAFETY: TBA will never return None in message.from().
    let iam: UserModel = unsafe {
        message
            .from()
            .unwrap_unchecked()
    }
    .into();
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
    let tg_ban = bot
        .send(BanChatMember::new(chat_ids, user.ids))
        .await
        .unwrap_or_default();

    let Ok(_) = db
        .execute_raw(raw_sql!(
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
                to_return
            )
            VALUES (
                {user.ids},
                {chat_ids},
                true,
                {iam.ids},
                {sms_ids},
                {reason},
                {now_ts},
                {until},
                {to_return}
            )
            ON CONFLICT (user_ids, chat_ids, is_ban)
            DO UPDATE SET
                moder_ids = EXCLUDED.moder_ids,
                sms_ids = EXCLUDED.sms_ids,
                reason = EXCLUDED.reason,
                added = EXCLUDED.added,
                removed = EXCLUDED.removed
            WHERE c8.rank <= EXCLUDED.rank
            "#
        ))
        .await
    else {
        return Ok(());
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

    if !reason.is_empty() {
        text.push_str("<b>Причина: <b>");
        text.push_str(reason);
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

    let access = module
        .check::<10>(ModuleAccess::M(&message))
        .await;
    if !access {
        return Ok(());
    }

    let chat_ids = message.chat().id();

    let _ = bot
        .send(GetChatMember::new(chat_ids, user.ids))
        .await?;

    bot.send(UnbanChatMember::new(chat_ids, user.ids.0).only_if_banned(true))
        .await?;

    let res = ChatBlock::delete_by_id((user.ids, chat_ids.into(), true))
        .exec(&db)
        .await;

    match res {
        Ok(r) if r.rows_affected > 0 => {
            bot.send(JuzoAnswer::message(&message).text(format!(
                "{0} <a href='{1}'>{2}</a> разбанен. Теперь можно добавить его в чат или <b>снова \
                 забанить</b> =)",
                smail_tick(true),
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
