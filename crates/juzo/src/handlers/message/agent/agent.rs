use juzo_core::{
    application::{UserIds, UserIndex, UserModel},
    common::emojis::smail_tick,
    db::agent::{agent, prelude::Agent},
};
use sea_orm::{EntityTrait, QuerySelect, Set, sea_query::OnConflict};
use telers::types::ReplyParameters;

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
    .id;

    let Ok(Some(true)) = Agent::find_by_id(my_ids)
        .select_only()
        .column(agent::Column::AddAgent)
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

    let model = agent::ActiveModel {
        user_ids: Set(user.ids),
        agent: Set(true),
        ..Default::default()
    };

    let _ = Agent::insert(model)
        .on_conflict(
            OnConflict::column(agent::Column::UserIds)
                .update_column(agent::Column::Agent)
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
                .text(format!("👨‍💻 Вы были назначены агентом поддержки «Juzo | Чат-Менеджер»"))
                .chat_id(user.ids.0)
                .business_connection_id_option::<&str>(None)
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

    let Ok(Some(true)) = Agent::find_by_id(my_ids)
        .select_only()
        .column(agent::Column::AddAgent)
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

    let model = agent::ActiveModel {
        user_ids: Set(user.ids),
        spam: Set(true),
        ..Default::default()
    };

    let _ = Agent::insert(model)
        .on_conflict(
            OnConflict::column(agent::Column::UserIds)
                .update_column(agent::Column::Spam)
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
                .business_connection_id_option::<&str>(None)
                .reply_parameters_option::<ReplyParameters>(None),
        )
        .await;

    Ok(())
}

pub async fn add_main(
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
        add_agent: Set(true),
        ..Default::default()
    };

    let _ = Agent::insert(model)
        .on_conflict(
            OnConflict::column(agent::Column::UserIds)
                .update_column(agent::Column::AddAgent)
                .to_owned(),
        )
        .exec(&db)
        .await;

    bot.send(JuzoAnswer::message(&message).text(format!(
        "{0} Гл. агент <a href='{1}'>{2}</a> назначен",
        smail_tick(true),
        user.link(),
        user.full_name(),
    )))
    .await?;

    let _ = bot
        .send(
            JuzoAnswer::message(&message)
                .text(format!(
                    "👨‍💻 Вы были назначены главный агентом поддержки «Juzo | Чат-Менеджер»"
                ))
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

    let Ok(Some(true)) = Agent::find_by_id(my_ids)
        .select_only()
        .column(agent::Column::AddAgent)
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

    let model = agent::ActiveModel {
        user_ids: Set(user.ids),
        agent: Set(false),
        ..Default::default()
    };

    let _ = Agent::insert(model)
        .on_conflict(
            OnConflict::column(agent::Column::UserIds)
                .update_column(agent::Column::Agent)
                .to_owned(),
        )
        .exec(&db)
        .await;

    bot.send(JuzoAnswer::message(&message).text(format!(
        "{0} Агент <a href='{1}'>{2}</a> разжалован",
        smail_tick(true),
        user.link(),
        user.full_name(),
    )))
    .await?;

    Ok(())
}

pub async fn delete_spam(
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

    let Ok(Some(true)) = Agent::find_by_id(my_ids)
        .select_only()
        .column(agent::Column::AddAgent)
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

    let model = agent::ActiveModel {
        user_ids: Set(user.ids),
        spam: Set(false),
        ..Default::default()
    };

    let _ = Agent::insert(model)
        .on_conflict(
            OnConflict::column(agent::Column::UserIds)
                .update_column(agent::Column::Spam)
                .to_owned(),
        )
        .exec(&db)
        .await;

    bot.send(JuzoAnswer::message(&message).text(format!(
        "{0} Агент антиспама <a href='{1}'>{2}</a> разжалован",
        smail_tick(true),
        user.link(),
        user.full_name(),
    )))
    .await?;

    Ok(())
}

pub async fn delete_main(
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
        add_agent: Set(false),
        ..Default::default()
    };

    let _ = Agent::insert(model)
        .on_conflict(
            OnConflict::column(agent::Column::UserIds)
                .update_column(agent::Column::AddAgent)
                .to_owned(),
        )
        .exec(&db)
        .await;

    bot.send(JuzoAnswer::message(&message).text(format!(
        "{0} Гл. агент <a href='{1}'>{2}</a> разжалован",
        smail_tick(true),
        user.link(),
        user.full_name(),
    )))
    .await?;

    Ok(())
}
