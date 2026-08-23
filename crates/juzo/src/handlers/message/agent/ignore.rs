use juzo_core::{
    application::{UserIndex, UserModel},
    common::emojis::{smail_pensil, smail_tick},
    db::agent::{
        BlockFunc, block_system,
        prelude::{Agent, BlockSystem},
    },
};
use sea_orm::{ConnectionTrait, EntityTrait, SelectExt, Set, raw_sql, sea_query::OnConflict};

use super::super::*;

pub async fn add(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    // SAFETY: TBA will never return None in message.from().
    let my_ids = unsafe {
        message
            .from()
            .unwrap_unchecked()
    }
    .id
    .into();

    let Ok(exists) = Agent::find_by_id(my_ids)
        .exists(&db)
        .await
    else {
        return Ok(());
    };
    if !exists {
        return Ok(());
    }

    let user_ind = UserIndex::new(&bot, &db);

    // SAFETY: The Command filter will not allow processing of a "None" value.
    let text = unsafe {
        message
            .text()
            .or_else(|| message.caption())
            .unwrap_unchecked()
    };
    let args = result.args::<1>(text);
    let comment = &text[result.first_line];

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

    let model = block_system::ActiveModel {
        user_ids: Set(user.ids),
        function: Set(BlockFunc::Ignore),
        agents_ids: Set(my_ids),
        reason: Set(comment.into()),
        ..Default::default()
    };

    let _ = BlockSystem::insert(model)
        .on_conflict(
            OnConflict::columns([block_system::Column::UserIds, block_system::Column::Function])
                .do_nothing()
                .to_owned(),
        )
        .exec(&db)
        .await;

    bot.send(JuzoAnswer::message(&message).text(format!(
        "{0} <a href='{1}'>{2}</a> занесён в «Juzo | Ignore System»",
        smail_tick(true),
        user.link(),
        user.full_name(),
    )))
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

    let Ok(exists) = Agent::find_by_id(my_ids)
        .exists(&db)
        .await
    else {
        return Ok(());
    };
    if !exists {
        return Ok(());
    }

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
                AND function = 1
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
                    VALUES ({user.ids}, {my_ids}, {r}, 1)
                    "#
                ))
                .await;

            bot.send(JuzoAnswer::message(&message).text(format!(
                "{0} <a href='{1}'>{2}</a> вынесен из «Juzo | Ignore System»",
                smail_tick(true),
                user.link(),
                user.full_name()
            )))
            .await?;
        }
        Ok(Some(_)) => {
            bot.send(JuzoAnswer::message(&message).text(format!(
                "{0} <a href='{1}'>{2}</a> вынесен из «Juzo | Ignore System» без пометки о выносе",
                smail_tick(true),
                user.link(),
                user.full_name()
            )))
            .await?;
        }
        _ => {
            bot.send(JuzoAnswer::message(&message).text(format!(
                "{0} <a href='{1}'>{2}</a> не находится в базе игнора.",
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

pub async fn take_delete(
    _bot: Bot,
    _message: Message,
    Extension(_db): Extension<DbConn>,
    Extension(_result): Extension<CommandResult>,
) -> HandlerResult<()> {
    // удаление пометки выноса

    Ok(())
}
