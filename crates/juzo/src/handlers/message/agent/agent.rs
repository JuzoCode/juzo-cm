use juzo_core::{
    application::{UserIds, UserIndex, UserModel},
    common::emojis::{smail_pensil, smail_tick},
    db::agent::{agent, prelude::Agent},
};
use sea_orm::{DbConn, EntityTrait, SelectExt, Set, sea_query::OnConflict};
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

    if !my_ids.is_creator_bot() {
        let Ok(exists) = Agent::find_by_id(my_ids)
            .exists(&db)
            .await
        else {
            return Ok(());
        };
        if !exists {
            return Ok(());
        }
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

    let model = agent::ActiveModel {
        user_ids: Set(user.ids),
        add_ids: Set(my_ids),
        ..Default::default()
    };

    let _ = Agent::insert(model)
        .on_conflict(
            OnConflict::column(agent::Column::UserIds)
                .do_nothing()
                .to_owned(),
        )
        .exec(&db)
        .await;

    bot.send(JuzoAnswer::message(&message).text(format!(
        "{0} Агент <a href='{1}'>{2}</a> назначен",
        smail_tick(true),
        user.link(),
        user.full_name(),
    )))
    .await?;
    let _ = bot
        .send(
            JuzoAnswer::message(&message)
                .text(format!("👨‍💻 Вы были назначены агентом поддержи «Juzo | Чат-Менеджер»"))
                .chat_id(user.ids.0)
                .reply_parameters_option::<ReplyParameters>(None),
        )
        .await;

    Ok(())
}

pub async fn add_spam(
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

    let model = agent::ActiveModel {
        user_ids: Set(user.ids),
        antispam: Set(true),
        add_ids: Set(my_ids),
        ..Default::default()
    };

    let _ = Agent::insert(model)
        .on_conflict(
            OnConflict::column(agent::Column::UserIds)
                .do_nothing()
                .to_owned(),
        )
        .exec(&db)
        .await;

    bot.send(JuzoAnswer::message(&message).text(format!(
        "{0} Агент антиспама <a href='{1}'>{2}</a> назначен",
        smail_tick(true),
        user.link(),
        user.full_name(),
    )))
    .await?;
    let _ = bot
        .send(
            JuzoAnswer::message(&message)
                .text(format!("👨‍💻 Вы были назначены агентом антиспама «Juzo | Чат-Менеджер»"))
                .chat_id(user.ids.0)
                .reply_parameters_option::<ReplyParameters>(None),
        )
        .await;

    Ok(())
}

pub async fn delete(
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

    let res = Agent::delete_by_id(user.ids)
        .exec(&db)
        .await;

    match res {
        Ok(r) if r.rows_affected > 0 => {
            bot.send(JuzoAnswer::message(&message).text(format!(
                "{0} Агент <a href='{1}'>{2}</a> разжалован",
                smail_tick(true),
                user.link(),
                user.full_name(),
            )))
            .await?;
        }
        _ => {
            bot.send(JuzoAnswer::message(&message).text(format!(
                "{0} <a href='{1}'>{2}</a> не является агентом.",
                smail_pensil(true),
                user.link(),
                user.full_name(),
            )))
            .await?;
        }
    }

    Ok(())
}
