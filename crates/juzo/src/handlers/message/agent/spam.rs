use juzo_core::{
    application::{ParseTgLink, UserIds, UserIndex, UserModel},
    common::emojis::{smail_pensil, smail_tick},
    db::agent::{agent, prelude::Agent},
    middlewares::outer::IgnoreSystem,
};
use sea_orm::{ConnectionTrait, EntityTrait, QuerySelect, raw_sql};
use telers::{methods::SendMessage, types::ReplyParameters};

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
        bot.send(JuzoAnswer::<SendMessage>::to(&message).text(format!(
            "{0} Длина текста превышает 128 символов.",
            smail_pensil(true)
        )))
        .await?;
        return Ok(());
    }

    let deny = if is_agent { 0 } else { 5 };
    let mut f = 0u8;
    let mut key = 1u32;

    for b in system
        .bytes()
        .chain(Some(b' '))
    {
        if b & 0xC0 == 0x80 {
            key = key << 5 | (b & 0x1F) as u32;
            continue;
        }
        if b == b' ' {
            f |= match key {
                1 => 0,
                SCAM => 1,
                IGNOR => 2,
                KICK => 4,
                _ => return Ok(()),
            };
            if f & deny != 0 || f == 7 {
                break;
            }
            key = 1;
        } else if b | 1 != 0xD1 {
            return Ok(());
        }
    }

    if f & deny != 0 {
        return Ok(());
    }
    let functions = (f & 3 | 4) as i16;

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

    bot.send(JuzoAnswer::<SendMessage>::to(&message).text(text))
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

    // убрать takeaway: bool, сделать на уровне парсинга. Смотреть в конце system
    // сделать массовое удаление из баз, как у вноса

    let Ok(Some(row)) = db
        .query_one_raw(raw_sql!(
            Postgres,
            r#"
            WITH d AS (
                DELETE FROM a3
                WHERE user_ids = {user.ids}
                    AND function BETWEEN 0 AND 2
                    AND (4::smallint & (1 << function)) != 0
                RETURNING function, reason
            ),
            ins AS (
                INSERT INTO a2 (user_ids, agents_ids, reason, function)
                SELECT {user.ids}, {my_ids}, reason, function
                FROM d
                WHERE {takeaway}::bool
                    AND function IN (1, 2)
            )
            SELECT COALESCE(BIT_OR(1 << function), 0)::smallint  AS mask
            FROM d
            "#
        ))
        .await
    else {
        bot.send(JuzoAnswer::<SendMessage>::to(&message).text(format!(
            "{0} <a href='{1}'>{2}</a> не был вынесен из баз.",
            smail_pensil(true),
            user.link(),
            user.full_name(),
        )))
        .await?;
        return Ok(());
    };

    // SAFETY: mask is always a non-NULL smallint (COALESCE), so try_get cannot fail
    let deleted = unsafe {
        row.try_get::<i16>("", "mask")
            .unwrap_unchecked()
    };

    if deleted == 0 {
        bot.send(JuzoAnswer::<SendMessage>::to(&message).text(format!(
            "{0} <a href='{1}'>{2}</a> не находится в базах.",
            smail_pensil(true),
            user.link(),
            user.full_name(),
        )))
        .await?;
        return Ok(());
    }

    let name = unsafe { *PLACES.get_unchecked(deleted as usize) };

    let (suffix, dm) = if takeaway {
        (
            "",
            format!("🗓 Вас вынесли из {name}.\n<b>Впредь больше не нарушайте</b>, \
             лучше почитайте моё <a href='https://teletype.in/@juzo_cm/EULA'>пользовательское \
             соглашение</a> =)"),
        )
    } else {
        (
            " без пометки о выносе",
            format!("🗓 Вас вынесли из {name} без пометки о выносе"),
        )
    };

    bot.send(JuzoAnswer::<SendMessage>::to(&message).text(format!(
        "{0} <a href='{1}'>{2}</a> вынесен из {name}{suffix}",
        smail_tick(true),
        user.link(),
        user.full_name(),
    )))
    .await?;

    let _ = bot
        .send(
            JuzoAnswer::<SendMessage>::to(&message)
                .text(dm)
                .chat_id(user.ids.0)
                .business_connection_id_option::<&str>(None)
                .reply_parameters_option::<ReplyParameters>(None),
        )
        .await;

    Ok(())
}

pub async fn delete(
    bot: Bot,
    message: Message,
    db: Extension<DbConn>,
    result: Extension<CommandResult>,
) -> HandlerResult<()> {
    delete_core(bot, message, db, result, true).await
}

pub async fn delete_takeaway(
    bot: Bot,
    message: Message,
    db: Extension<DbConn>,
    result: Extension<CommandResult>,
) -> HandlerResult<()> {
    delete_core(bot, message, db, result, false).await
}

const SCAM: u32 = 1 << 20 | 0x01 << 15 | 0x1A << 10 | 0x10 << 5 | 0x1C;
const IGNOR: u32 = 1 << 25 | 0x18 << 20 | 0x13 << 15 | 0x1D << 10 | 0x1E << 5;
const KICK: u32 = 1 << 15 | 0x1A << 10 | 0x18 << 5 | 0x1A;

static PLACES: [&str; 8] = [
    "<no_PLACES>", // чтобы сломать HTML
    "«Juzo | Scam System»",
    "«Juzo | Ignore System»",
    "«Juzo | Ignore System»<b> и скам-базы</b>",
    "«Juzo | Anti-Spam»",
    "«Juzo | Anti-Spam»<b> и скам-базы</b>",
    "«Juzo | Anti-Spam»<b> и игнора команд</b>",
    "«Juzo | Anti-Spam»<b> и игнора команд, а также из скам-базы</b>",
];
