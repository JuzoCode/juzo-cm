use core::fmt::Write;

use juzo_core::{
    application::{UserIds, UserIndex, UserModel},
    common::emojis::{smail_pensil, smail_tick},
    db::{
        agent::prelude::Agent,
        bot::{managed, prelude::ManagedBot},
    },
    payloads::base::encode_token,
};
use sea_orm::{
    ColumnTrait, Condition, DbConn, EntityTrait, QueryFilter, QuerySelect, SelectExt, Set,
    sea_query::OnConflict,
};
use telers::utils::token::extract_bot_id;

use super::super::*;

pub async fn info(
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
            .id
            .into()
    };

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
        Some([a1]) => {
            let Ok(found_user) = user_ind
                .search_user(&text[a1])
                .await
            else {
                return Ok(());
            };
            found_user
        }
        None => unsafe {
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
    };

    let Ok(data) = ManagedBot::find_by_id(user.ids)
        .select_only()
        .columns([
            managed::Column::ManagedBot,
            managed::Column::CreatorIds,
            managed::Column::IsOfficial,
        ])
        .into_tuple::<(i64, i64, bool)>()
        .one(&db)
        .await
    else {
        return Ok(());
    };

    if data.is_none() {
        bot.send(
            JuzoAnswer::message(&message)
                .text(format!("{0} Данный бот не поддерживается Джузом.", smail_pensil(true))),
        )
        .await?;
        return Ok(());
    }
    let data = data.unwrap();

    let mut text = format!(
        "<tg-emoji emoji-id='5339267587337370029'>🤖</tg-emoji> {0} создан и поддерживается {1}",
        user.ids, data.0,
    );
    if !data.2 {
        let _ = write!(text, ".\n&lt;/&gt; <b>Создатель:</b> {0}", data.1);
    }

    bot.send(JuzoAnswer::message(&message).text(text))
        .await?;

    Ok(())
}

pub async fn set_official(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if result.args.is_empty() {
        return Ok(());
    }

    // SAFETY: TBA will never return None in message.from().
    let my_ids: UserIds = unsafe {
        message
            .from()
            .unwrap_unchecked()
            .id
            .into()
    };
    if !my_ids.is_creator_bot() {
        return Ok(());
    }

    // SAFETY: The Command filter will not allow processing of a "None" value.
    let text = unsafe {
        message
            .text()
            .or_else(|| message.caption())
            .unwrap_unchecked()
    };
    let Some([a1]) = result.args::<1>(text) else {
        return Ok(());
    };

    let token = &text[a1];
    let Some(bot_ids) = extract_bot_id(token) else {
        bot.send(
            JuzoAnswer::message(&message)
                .text(format!("{0} Токен не является валидным.", smail_pensil(true))),
        )
        .await?;
        return Ok(());
    };

    let secret_token = unsafe { encode_token(token.as_bytes()) };

    let model = managed::ActiveModel {
        bot_ids: Set(bot_ids.into()),
        creator_ids: Set(my_ids.into()),
        managed_bot: Set(392851555.into()),
        is_official: Set(true),
        token: Set(token.into()),
        secret_token: unsafe { Set(str::from_utf8_unchecked(&secret_token).into()) },
    };

    let _ = ManagedBot::insert(model)
        .on_conflict(
            OnConflict::column(managed::Column::BotIds)
                .update_columns([managed::Column::Token, managed::Column::SecretToken])
                .to_owned(),
        )
        .exec(&db)
        .await;

    // bot.send(
    //     SetWebhook::new("JUZO MEOW")
    //         .drop_pending_updates(true)
    //         .allowed_updates(ALLOWEDS_UPDATES)
    //         .secret_token(secret_token)
    // ).await?;

    bot.send(
        JuzoAnswer::message(&message)
            .text(format!("{0} Был создан санкционированный клон Джузо", smail_tick(true))),
    )
    .await?;

    Ok(())
}

pub async fn delete(
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
            .id
            .into()
    };

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
        Some([a1]) => {
            let Ok(found_user) = user_ind
                .search_user(&text[a1])
                .await
            else {
                return Ok(());
            };
            found_user
        }
        None => unsafe {
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
    };

    let delete_condition = Condition::all()
        .add_option((!my_ids.is_creator_bot()).then(|| managed::Column::IsOfficial.eq(false)));

    let res = ManagedBot::delete_by_id(user.ids)
        .filter(delete_condition)
        .exec(&db)
        .await;

    match res {
        Ok(r) if r.rows_affected > 0 => {
            bot.send(JuzoAnswer::message(&message).text(format!(
                "{0} Клон {1} удалён",
                smail_tick(true),
                user.ids
            )))
            .await?;
        }
        _ => {
            bot.send(JuzoAnswer::message(&message).text(format!(
                "{0} {1} является санкционированным клоном или вовсе не является клоном.",
                smail_pensil(true),
                user.ids
            )))
            .await?;
        }
    }

    Ok(())
}

/// добавить в бизнес мод и аргию команды тоже
pub async fn changing_creator(
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
            .id
            .into()
    };

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

    // тут два аргумента [a1: пользователь, a2: бот] или [a1: пользователь] и реплай бот

    let user: UserModel = match args {
        Some([a1]) => {
            let Ok(found_user) = user_ind
                .search_user(&text[a1])
                .await
            else {
                return Ok(());
            };
            found_user
        }
        None => unsafe {
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
    };

    let res = ManagedBot::update(managed::ActiveModel {
        bot_ids: Set(123456789.into()),
        creator_ids: Set(user.ids),
        ..Default::default()
    })
    .exec(&db)
    .await;

    match res {
        Ok(_) => {
            bot.send(
                JuzoAnswer::message(&message)
                    .text(format!("{0} У {{}} сменился создатель", smail_tick(true))),
            )
            .await?;
        }
        _ => {
            bot.send(
                JuzoAnswer::message(&message)
                    .text(format!("{0} {{}} не является клоном.", smail_pensil(true))),
            )
            .await?;
        }
    }

    Ok(())
}
