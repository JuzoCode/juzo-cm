use juzo_core::{
    application::{UserIds, UserIndex, UserModel},
    common::emojis::{smail_pensil, smail_tick},
    db::agent::{agent, prelude::Agent},
};
use sea_orm::{DbConn, EntityTrait, SelectExt, Set, sea_query::OnConflict};

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
            } else {
                return Ok(());
            }
        },
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
        "{0} Агент {1} назначен",
        smail_tick(true),
        user.ids
    )))
    .await?;

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
            } else {
                return Ok(());
            }
        },
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
        "{0} Агент антиспама {1} назначен",
        smail_tick(true),
        user.ids
    )))
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
            } else {
                return Ok(());
            }
        },
    };

    let res = Agent::delete_by_id(user.ids)
        .exec(&db)
        .await;

    match res {
        Ok(r) if r.rows_affected > 0 => {
            bot.send(JuzoAnswer::message(&message).text(format!(
                "{0} Агент {1} разжалован",
                smail_tick(true),
                user.ids
            )))
            .await?;
        }
        _ => {
            bot.send(JuzoAnswer::message(&message).text(format!(
                "{0} {1} не является агентом.",
                smail_pensil(true),
                user.ids
            )))
            .await?;
        }
    }

    Ok(())
}
