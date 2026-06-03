use juzo_core::{
    application::{UserIds, UserIndex, UserModel},
    common::emojis::{smail_pensil, smail_tick},
    db::agent::{
        block_system,
        prelude::{Agent, BlockSystem},
    },
    domain::enums::BlockFunc,
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

    let model = block_system::ActiveModel {
        user_ids: Set(user.ids),
        function: Set(BlockFunc::AntiSpam),
        agents_ids: Set(my_ids),
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

    bot.send(
        JuzoAnswer::message(&message)
            .text(format!("{0} {{}} занесён в «Juzo | Anti-Spam»", smail_tick(true))),
    )
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

    let res = BlockSystem::delete_by_id((user.ids, BlockFunc::AntiSpam))
        .exec(&db)
        .await;

    match res {
        Ok(r) if r.rows_affected > 0 => {
            bot.send(JuzoAnswer::message(&message).text(format!(
                "{0} {{}} вынесен из «Juzo | Anti-Spam»{1}",
                smail_tick(true),
                ["", " без пометки о выносе"][takeaway as usize]
            )))
            .await?;
        }
        _ => {
            bot.send(
                JuzoAnswer::message(&message)
                    .text(format!("{0} {{}} не находится в базе спама.", smail_pensil(true))),
            )
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
