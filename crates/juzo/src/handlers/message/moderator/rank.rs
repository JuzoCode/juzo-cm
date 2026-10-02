use juzo_core::{
    application::{ParseTgLink, UserIndex, UserModel},
    common::emojis::smail_pensil,
    domain::UserModelExt,
};
use sea_orm::{ConnectionTrait, raw_sql};

use super::super::*;

async fn _up_core(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
    new_rank: u8,
) -> HandlerResult<()> {
    let user_ind = UserIndex::new(&bot, &db);

    // SAFETY: The Command filter will not allow processing of a "None" value.
    let text = unsafe {
        message
            .text()
            .or_else(|| message.caption())
            .unwrap_unchecked()
    };
    let args = result.args::<2>(text);

    let (_rank, _user): (u8, UserModel) = match args {
        ArgsResult::Some([a1, a2], 2) => {
            let Ok(found_user) = user_ind
                .search_user(&text[a2])
                .await
            else {
                return Ok(());
            };

            let Ok(rank) = text[a1].parse::<u8>() else {
                return Ok(());
            };

            (rank, found_user)
        }
        ArgsResult::Some([a1, _], 1) => unsafe {
            if let Some(link) = ParseTgLink::new(&text[a1]) {
                let Ok(found_user) = user_ind
                    .fetch_user(link)
                    .await
                else {
                    return Ok(());
                };

                (new_rank, found_user)
            } else {
                let found_user = if let Some(r) = message.reply_to_message() {
                    // SAFETY: TBA will never return None in message.from().
                    UserModel::new(
                        &db,
                        r.from()
                            .unwrap_unchecked(),
                    )
                    .await
                } else {
                    return Ok(());
                };

                (new_rank, found_user)
            }
        },
        // SAFETY: TBA will never return None in message.from().
        ArgsResult::None => unsafe {
            let found_user = if let Some(r) = message.reply_to_message() {
                UserModel::new(
                    &db,
                    r.from()
                        .unwrap_unchecked(),
                )
                .await
            } else {
                return Ok(());
            };

            (new_rank, found_user)
        },
        _ => return Ok(()),
    };

    Ok(())
}

pub async fn down(
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
                UserModel::new(
                    &db,
                    r.from()
                        .unwrap_unchecked(),
                )
                .await
            } else {
                return Ok(());
            }
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
    let sms_ids = message.message_id();
    let chat_ids = message.chat().id();

    let Ok(Some(row)) = db
        .query_one_raw(raw_sql!(
            Postgres,
            r#"
            WITH result AS (
                UPDATE c4 SET
                    rank = rank - 1,
                    peer_ids = {my_ids},
                    sms_ids = {sms_ids},
                    added = EXTRACT(EPOCH FROM NOW())::bigint
                WHERE user_ids = {user.ids}
                    AND chat_ids = {chat_ids}
                    AND rank > 0
                    AND (
                        rank < COALESCE(
                            (SELECT rank FROM c4
                            WHERE user_ids = {my_ids}
                                AND chat_ids = {chat_ids}),
                            0
                        )
                        OR EXISTS (
                            SELECT 1 FROM c
                            WHERE chat_ids = {chat_ids}
                            AND owner_ids = {my_ids}
                        )
                    )
                RETURNING rank
            )
            SELECT COALESCE((SELECT rank FROM result), -1)::smallint AS affected
            WHERE EXISTS (SELECT 1 FROM result)
                OR NOT EXISTS (
                    SELECT 1 FROM c4
                    WHERE user_ids = {user.ids}
                        AND chat_ids = {chat_ids}
                        AND rank > 0
                );
            "#
        ))
        .await
    else {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Пытаться понизить должность тому, кто выше или равен вам? Смело, но корона не \
             жмёт?",
            smail_pensil(true)
        )))
        .await?;
        return Ok(());
    };

    let affected = {
        row.try_get::<i16>("", "affected")
            .unwrap()
    };

    if affected == -1 {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} <a href='{1}'>{2}</a> не является модератором.",
            smail_pensil(true),
            user.full_name(),
            user.link()
        )))
        .await?;
        return Ok(());
    }

    bot.send(JuzoAnswer::message(&message).text(format!("{affected}")))
        .await?;

    Ok(())
}
