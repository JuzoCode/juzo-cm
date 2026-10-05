use juzo_core::{
    application::{ParseTgLink, UserIndex, UserModel},
    common::emojis::{smail_cross, smail_pensil, smail_tick},
    domain::{AttachResult, UserModelExt},
    gender,
};
use sea_orm::{ConnectionTrait, raw_sql};

use super::super::*;

async fn up_core(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(arch): Extension<AttachResult>,
    Extension(result): Extension<CommandResult>,
    default_value: u8,
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
    let chat_ids = arch.chat_ids;

    let (value, user): (u8, UserModel) = match args {
        ArgsResult::Some([a1, _], 1) => unsafe {
            if let Some(link) = ParseTgLink::new(&text[a1]) {
                let Ok(found_user) = user_ind
                    .fetch_user(link)
                    .await
                else {
                    return Ok(());
                };
                (default_value, found_user)
            } else if a1.is_empty() {
                let Some(reply) = message.reply_to_message() else {
                    return Ok(());
                };
                // SAFETY: TBA will never return None in message.from().
                (
                    default_value,
                    UserModel::new(
                        &db,
                        reply
                            .from()
                            .unwrap_unchecked(),
                    )
                    .await,
                )
            } else if default_value <= 1 {
                let Some(reply) = message.reply_to_message() else {
                    return Ok(());
                };
                let Ok(value) = text[a1].parse() else {
                    return Ok(());
                };
                // SAFETY: TBA will never return None in message.from().
                (
                    value,
                    UserModel::new(
                        &db,
                        reply
                            .from()
                            .unwrap_unchecked(),
                    )
                    .await,
                )
            } else {
                return Ok(());
            }
        },
        ArgsResult::Some([a1, a2], 2) if default_value <= 1 => {
            let Ok(value) = text[a1].parse() else {
                return Ok(());
            };

            if let Some(link) = ParseTgLink::new(&text[a2]) {
                let Ok(found_user) = user_ind
                    .fetch_user(link)
                    .await
                else {
                    return Ok(());
                };
                (value, found_user)
            } else {
                let Some(reply) = message.reply_to_message() else {
                    return Ok(());
                };
                // SAFETY: TBA will never return None in message.from().
                (value, unsafe {
                    UserModel::new(
                        &db,
                        reply
                            .from()
                            .unwrap_unchecked(),
                    )
                    .await
                })
            }
        }
        // SAFETY: TBA will never return None in message.from().
        ArgsResult::None => {
            let Some(reply) = message.reply_to_message() else {
                return Ok(());
            };
            (default_value, unsafe {
                UserModel::new(
                    &db,
                    reply
                        .from()
                        .unwrap_unchecked(),
                )
                .await
            })
        }
        _ => return Ok(()),
    };

    let true = module
        .check::<8>(ModuleAccess::CustomM(&message, chat_ids.0))
        .await
    else {
        return Ok(());
    };

    if value > 6 {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Выдавать ранг выше своего... Амбициям вашим я поражаюсь.",
            smail_pensil(true)
        )))
        .await?;
        return Ok(());
    }

    // SAFETY: TBA will never return None in message.from().
    let my_ids = unsafe {
        message
            .from()
            .unwrap_unchecked()
    }
    .id;
    let sms_ids = message.message_id();

    let Ok(Some(row)) = db
        .query_one_raw(raw_sql!(
            Postgres,
            r#"
            WITH data AS (
                SELECT
                    COALESCE(
                        (SELECT rank FROM c4
                        WHERE user_ids = {my_ids}
                            AND chat_ids = {chat_ids}),
                        0
                    ) AS my_rank,
                    t.target_rank,
                    CASE
                        WHEN {value} = 0 THEN t.target_rank + 1
                        ELSE {value}::smallint
                    END AS rank
                FROM (
                    SELECT COALESCE(
                        (SELECT rank FROM c4
                        WHERE user_ids = {user.ids}
                            AND chat_ids = {chat_ids}),
                        0
                    ) AS target_rank
                ) AS t
            ),
            result AS (
                INSERT INTO c4 (
                    user_ids,
                    chat_ids,
                    peer_ids,
                    rank,
                    sms_ids
                )
                SELECT
                    {user.ids},
                    {chat_ids},
                    {my_ids},
                    data.rank,
                    {sms_ids}
                FROM data
                WHERE data.rank > data.target_rank
                    AND data.rank <= data.my_rank
                ON CONFLICT (user_ids, chat_ids)
                DO UPDATE SET
                    rank = EXCLUDED.rank,
                    peer_ids = EXCLUDED.peer_ids,
                    sms_ids = EXCLUDED.sms_ids,
                    added = EXTRACT(EPOCH FROM NOW())::bigint
                RETURNING rank
            )
            SELECT COALESCE((SELECT rank FROM result), 0)::smallint AS affected
            FROM data
            WHERE EXISTS (SELECT 1 FROM result)
                OR data.rank <= data.my_rank;
            "#
        ))
        .await
    else {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Выдавать ранг выше своего... Амбициям вашим я поражаюсь.",
            smail_pensil(true)
        )))
        .await?;
        return Ok(());
    };

    let affected = unsafe {
        row.try_get::<i16>("", "affected")
            .unwrap_unchecked()
    };

    if affected == 0 {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Повысить до того, что уже есть? Просто прекрасно. А дальше что — выдать тот же \
             ранг ещё раз?",
            smail_pensil(true)
        )))
        .await?;
    } else {
        let g1 = gender!(user.gender => ["а", ""]);

        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} <a href='{1}'>{2}</a> назначен{g1} на {affected} ранг",
            smail_tick(true),
            user.link(),
            user.full_name()
        )))
        .await?;
    }

    Ok(())
}

pub async fn up(
    bot: Bot,
    message: Message,
    db: Extension<DbConn>,
    arch: Extension<AttachResult>,
    result: Extension<CommandResult>,
) -> HandlerResult<()> {
    up_core(bot, message, db, arch, result, 0).await
}

pub async fn up_1(
    bot: Bot,
    message: Message,
    db: Extension<DbConn>,
    arch: Extension<AttachResult>,
    result: Extension<CommandResult>,
) -> HandlerResult<()> {
    up_core(bot, message, db, arch, result, 1).await
}

pub async fn up_2(
    bot: Bot,
    message: Message,
    db: Extension<DbConn>,
    arch: Extension<AttachResult>,
    result: Extension<CommandResult>,
) -> HandlerResult<()> {
    up_core(bot, message, db, arch, result, 2).await
}

pub async fn up_3(
    bot: Bot,
    message: Message,
    db: Extension<DbConn>,
    arch: Extension<AttachResult>,
    result: Extension<CommandResult>,
) -> HandlerResult<()> {
    up_core(bot, message, db, arch, result, 3).await
}

pub async fn up_4(
    bot: Bot,
    message: Message,
    db: Extension<DbConn>,
    arch: Extension<AttachResult>,
    result: Extension<CommandResult>,
) -> HandlerResult<()> {
    up_core(bot, message, db, arch, result, 4).await
}

pub async fn down(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(arch): Extension<AttachResult>,
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
    let chat_ids = arch.chat_ids;

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

    let true = module
        .check::<8>(ModuleAccess::CustomM(&message, chat_ids.0))
        .await
    else {
        return Ok(());
    };

    // SAFETY: TBA will never return None in message.from().
    let my_ids = unsafe {
        message
            .from()
            .unwrap_unchecked()
    }
    .id;
    let sms_ids = message.message_id();

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

    let affected = unsafe {
        row.try_get::<i16>("", "affected")
            .unwrap_unchecked()
    };

    if affected == -1 {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} <a href='{1}'>{2}</a> не является модератором.",
            smail_pensil(true),
            user.link(),
            user.full_name()
        )))
        .await?;
    } else if affected == 0 {
        let g1 = gender!(user.gender => ["а", ""]);

        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Модератор <a href='{1}'>{2}</a> разжалован{g1}",
            smail_cross(true),
            user.link(),
            user.full_name()
        )))
        .await?;
    } else {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Модератору <a href='{1}'>{2}</a> понижен ранг",
            smail_tick(true),
            user.link(),
            user.full_name()
        )))
        .await?;
    }

    Ok(())
}

pub async fn remove(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(arch): Extension<AttachResult>,
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
    let chat_ids = arch.chat_ids;

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

    let true = module
        .check::<8>(ModuleAccess::CustomM(&message, chat_ids.0))
        .await
    else {
        return Ok(());
    };

    // SAFETY: TBA will never return None in message.from().
    let my_ids = unsafe {
        message
            .from()
            .unwrap_unchecked()
    }
    .id;

    let Ok(Some(row)) = db
        .query_one_raw(raw_sql!(
            Postgres,
            r#"
            WITH result AS (
                DELETE FROM c4
                WHERE user_ids = {user.ids}
                    AND chat_ids = {chat_ids}
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
                RETURNING 1
            )
            SELECT EXISTS (SELECT 1 FROM result) AS affected
            WHERE EXISTS (SELECT 1 FROM result)
                OR NOT EXISTS (
                    SELECT 1 FROM c4
                    WHERE user_ids = {user.ids}
                        AND chat_ids = {chat_ids}
                );
            "#
        ))
        .await
    else {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Вы не можете разжаловать равного или выше себя рангом — одного желания для этого \
             маловато.",
            smail_pensil(true)
        )))
        .await?;
        return Ok(());
    };

    let affected = unsafe {
        row.try_get::<bool>("", "affected")
            .unwrap_unchecked()
    };

    if affected {
        let g1 = gender!(user.gender => ["а", ""]);

        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Модератор <a href='{1}'>{2}</a> разжалован{g1}",
            smail_cross(true),
            user.link(),
            user.full_name()
        )))
        .await?;
    } else {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} <a href='{1}'>{2}</a> не является модератором.",
            smail_pensil(true),
            user.link(),
            user.full_name()
        )))
        .await?;
    }

    Ok(())
}
