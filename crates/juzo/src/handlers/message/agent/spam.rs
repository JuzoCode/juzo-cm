use juzo_core::{
    application::{ParseTgLink, UserIds, UserIndex, UserModel},
    common::emojis::{smail_pensil, smail_tick},
    db::agent::{agent, prelude::Agent},
    middlewares::outer::IgnoreSystem,
};
use sea_orm::{ConnectionTrait, EntityTrait, QuerySelect, raw_sql};
use telers::types::ReplyParameters;

use super::super::*;

pub async fn add(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    // SAFETY: TBA will never return None in message.from().
    let my_ids: UserIds = unsafe {
        message
            .from()
            .unwrap_unchecked()
    }
    .id
    .into();

    let Ok(Some((is_agent, true))) = Agent::find_by_id(my_ids)
        .select_only()
        .columns([agent::Column::Agent, agent::Column::Spam])
        .into_tuple::<(bool, bool)>()
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
    let args = result.args::<4>(text);
    let comment = &text[result.first_line];

    let (system, user): (&str, UserModel) = match args {
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
                let found_user = if let Some(r) = message.reply_to_message() {
                    // SAFETY: TBA will never return None in message.from().
                    unsafe {
                        r.from()
                            .unwrap_unchecked()
                            .into()
                    }
                } else if message
                    .business_connection_id()
                    .is_some()
                {
                    message.chat().into()
                } else {
                    return Ok(());
                };

                (&text[args[0].start..last.end], found_user)
            }
        }
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

            ("", found_user)
        },
        ArgsResult::Unk => return Ok(()),
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

    let bytes = system.as_bytes();

    let mut functions: i16 = 4;
    let mut start = 0;
    let mut kick = false;

    for i in 0..=bytes.len() {
        if i != bytes.len() && bytes[i] != b' ' {
            continue;
        }

        if start != i {
            match &bytes[start..i] {
                KICK => kick = true,
                IGNORE => functions |= 2,
                SCAM => functions |= 1,
                _ => return Ok(()),
            }
        }

        if i == bytes.len() || functions == 7 {
            break;
        }

        start = i + 1;
    }

    if (kick || functions & 1 != 0) && !is_agent {
        return Ok(());
    }

    let _ = db
        .execute_raw(raw_sql!(
            Postgres,
            r#"
            INSERT INTO a3 (
                user_ids,
                function,
                agents_ids,
                reason
            )
            SELECT
                {user.ids},
                function,
                {my_ids},
                {comment}
            FROM generate_series(0, 2) AS function
            WHERE ({functions} & (1 << function)) != 0
            ON CONFLICT (user_ids, function) DO UPDATE SET
                reason = EXCLUDED.reason,
                agents_ids = EXCLUDED.agents_ids,
                added = EXTRACT(EPOCH FROM NOW())
            "#
        ))
        .await;

    let mut text = format!(
        "{0} <a href='{1}'>{2}</a> занесён в «Juzo | Anti-Spam»",
        smail_tick(true),
        user.link(),
        user.full_name(),
    );

    if functions & 2 != 0 {
        IgnoreSystem::edit(user.ids, true);

        text.push_str("<b> c игнором команд</b>");
    }

    if functions & 1 != 0 {
        text.push_str("<b>, а также в скам-базу</b>");
    }

    bot.send(JuzoAnswer::message(&message).text(text))
        .await?;

    Ok(())
}

async fn delete_core(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
    takeaway: bool,
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
            } else if message
                .business_connection_id()
                .is_some()
            {
                message.chat().into()
            } else {
                return Ok(());
            }
        },
        ArgsResult::Unk => return Ok(()),
    };

    let res = db
        .query_one_raw(raw_sql!(
            Postgres,
            r#"
            DELETE FROM a3
            WHERE user_ids = {user.ids}
                AND function = 2
            RETURNING reason
            "#
        ))
        .await
        .and_then(|row| match row {
            Some(row) => row
                .try_get::<String>("", "reason")
                .map(Some),
            None => Ok(None),
        });

    match res {
        Ok(Some(r)) if !takeaway => {
            let _ = db
                .execute_raw(raw_sql!(
                    Postgres,
                    r#"
                    INSERT INTO a2 (
                        user_ids,
                        agents_ids,
                        reason,
                        function
                    )
                    VALUES ({user.ids}, {my_ids}, {r}, 2)
                    "#
                ))
                .await;

            bot.send(JuzoAnswer::message(&message).text(format!(
                "{0} <a href='{1}'>{2}</a> вынесен из «Juzo | Anti-Spam»",
                smail_tick(true),
                user.link(),
                user.full_name()
            )))
            .await?;

            let _ = bot
                .send(
                    JuzoAnswer::message(&message)
                        .text("🗓 Вас вынесли из «Juzo | Anti-Spam».\n<b>Впредь больше не нарушайте</b>, лучше почитайте моё <a href='https://teletype.in/@juzo_cm/EULA'>пользовательское соглашение</a> =)")
                        .chat_id(user.ids.0)
                        .business_connection_id_option::<&str>(None)
                        .reply_parameters_option::<ReplyParameters>(None),
                )
                .await;
        }
        Ok(Some(_)) => {
            bot.send(JuzoAnswer::message(&message).text(format!(
                "{0} <a href='{1}'>{2}</a> вынесен из «Juzo | Anti-Spam» без пометки о выносе",
                smail_tick(true),
                user.link(),
                user.full_name(),
            )))
            .await?;

            let _ = bot
                .send(
                    JuzoAnswer::message(&message)
                        .text("🗓 Вас вынесли из «Juzo | Anti-Spam» без пометки о выносе")
                        .chat_id(user.ids.0)
                        .business_connection_id_option::<&str>(None)
                        .reply_parameters_option::<ReplyParameters>(None),
                )
                .await;
        }
        _ => {
            bot.send(JuzoAnswer::message(&message).text(format!(
                "{0} <a href='{1}'>{2}</a> не находится в базе спама.",
                smail_pensil(true),
                user.link(),
                user.full_name(),
            )))
            .await?;
        }
    }

    Ok(())
}

pub async fn delete(
    bot: Bot,
    message: Message,
    db: Extension<DbConn>,
    result: Extension<CommandResult>,
) -> HandlerResult<()> {
    delete_core(bot, message, db, result, false).await
}

pub async fn delete_takeaway(
    bot: Bot,
    message: Message,
    db: Extension<DbConn>,
    result: Extension<CommandResult>,
) -> HandlerResult<()> {
    delete_core(bot, message, db, result, true).await
}

const KICK: &[u8] = "кик".as_bytes();
const IGNORE: &[u8] = "игнор".as_bytes();
const SCAM: &[u8] = "скам".as_bytes();
