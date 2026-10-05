use juzo_core::{
    common::emojis::smail_tick,
    db::user::{prelude::UserSetting, setting},
    domain::UserModel,
};
use sea_orm::{EntityTrait, Set, sea_query::OnConflict};
use telers::types::ReplyParameters;

use super::super::*;

pub async fn yes(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    let module = ModuleChecker::new(&bot, &db);
    let true = module
        .check::<45>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    let iam: UserModel = unsafe {
        message
            .from()
            .unwrap_unchecked()
    }
    .into();

    if !iam.is_user {
        return Ok(());
    }

    let model = setting::ActiveModel {
        user_ids: Set(iam.ids.into()),
        contact_chat: Set(message
            .chat()
            .id()
            .into()),
    };

    let _ = UserSetting::insert(model)
        .on_conflict(
            OnConflict::column(setting::Column::UserIds)
                .update_column(setting::Column::ContactChat)
                .to_owned(),
        )
        .exec(&db)
        .await;

    let _ = bot
        .send(
            JuzoAnswer::message(&message)
                .text(format!("{0} Чат был привязан", smail_tick(true)))
                .chat_id(iam.ids.0)
                .business_connection_id_option::<&str>(None)
                .reply_parameters_option::<ReplyParameters>(None),
        )
        .await;

    Ok(())
}

pub async fn no(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    let my_ids = unsafe {
        message
            .from()
            .unwrap_unchecked()
    }
    .id;

    let model = setting::ActiveModel {
        user_ids: Set(my_ids.into()),
        contact_chat: Set(0.into()),
    };

    let _ = UserSetting::insert(model)
        .on_conflict(
            OnConflict::column(setting::Column::UserIds)
                .update_column(setting::Column::ContactChat)
                .to_owned(),
        )
        .exec(&db)
        .await;

    bot.send(JuzoAnswer::message(&message).text(format!("{0} Чат был отвязан", smail_tick(true))))
        .await?;

    Ok(())
}

// pub async fn no(
//     bot: Bot,
//     message: Message,
//     Extension(db): Extension<DbConn>,
//     Extension(result): Extension<CommandResult>,
// ) -> HandlerResult<()> {
//     if !result.args.is_empty() {
//         return Ok(());
//     }

//     Ok(())
// }
