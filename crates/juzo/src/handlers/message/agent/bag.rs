use juzo_core::{
    application::{ParseTgLink, UserIds, UserIndex, UserModel},
    common::emojis::{smail_tick, smail_warning},
    db::agent::{agent, prelude::Agent},
};
use sea_orm::{ConnectionTrait, EntityTrait, QuerySelect, raw_sql};

use super::super::*;

pub async fn yes(
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

    let Ok(Some(true)) = Agent::find_by_id(my_ids)
        .select_only()
        .column(agent::Column::Agent)
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
    let args = result.args::<6>(text);

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
        _ => return Ok(()),
    };

    let bytes = system.as_bytes();

    let mut functions: i16 = 0;
    let mut start = 0;

    for i in 0..=bytes.len() {
        if i != bytes.len() && bytes[i] != b' ' && bytes[i] != b',' {
            continue;
        }

        if start != i {
            match &bytes[start..i] {
                CANDY => functions |= 1,
                SS | STARS => functions |= 2,
                COINS => functions |= 4,
                GOLD => functions |= 8,
                SCORE => functions |= 16,
                BAG => functions |= 32,
                _ => return Ok(()),
            }

            if functions > 32 {
                return Ok(());
            }
        }

        if i == bytes.len() || functions == 31 {
            break;
        }

        start = i + 1;
    }

    let functions = functions | (functions >> 5) * 31;

    let Ok(_) = db
        .execute_raw(raw_sql!(
            Postgres,
            r#"
            WITH upd AS (
                UPDATE u2 SET
                    sweets    = CASE WHEN ({functions} & 1) != 0 THEN 0 ELSE sweets END,
                    asterisks = CASE WHEN ({functions} & 2) != 0 THEN 0::oid ELSE asterisks END,
                    coins     = CASE WHEN ({functions} & 4) != 0 THEN 0::oid ELSE coins END,
                    gold      = CASE WHEN ({functions} & 8) != 0 THEN 0::oid ELSE gold END,
                    score     = CASE WHEN ({functions} & 16) != 0 THEN 0::oid ELSE score END
                WHERE user_ids = {user.ids}
                RETURNING 1
            )
            INSERT INTO u5 (
                user_ids,
                peer_ids,
                balance,
                amount,
                log,
                currency
            )
            SELECT {user.ids}, {my_ids}, 0, 0, v.log, v.currency
            FROM upd,
                (VALUES (1, 4, 1), (3, 5, 8), (2, 3, 16)) AS v (currency, log, bit)
            WHERE ({functions} & v.bit) != 0
            "#
        ))
        .await
    else {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Обнуление мешка <a href='{1}'>{2}</a> получилось <b>неудачным</b>.",
            smail_warning(true),
            user.link(),
            user.full_name(),
        )))
        .await?;
        return Ok(());
    };

    bot.send(JuzoAnswer::message(&message).text(format!(
        "{0} Мешок <a href='{1}'>{2}</a> обнулен",
        smail_tick(true),
        user.link(),
        user.full_name(),
    )))
    .await?;

    Ok(())
}

const BAG: &[u8] = "мешок".as_bytes();
const CANDY: &[u8] = "леденцы".as_bytes();
const SS: &[u8] = "зв".as_bytes();
const STARS: &[u8] = "звезды".as_bytes();
const COINS: &[u8] = "коины".as_bytes();
const GOLD: &[u8] = "голду".as_bytes();
const SCORE: &[u8] = "од".as_bytes();
