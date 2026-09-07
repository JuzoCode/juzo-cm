use juzo_core::{
    common::emojis::{smail_cross, smail_tick},
    db::chat::{
        chat_module,
        prelude::{ChatModule, ChatSetting},
        setting,
    },
    domain::AttachResult,
};
use sea_orm::{EntityTrait, Set, sea_query::OnConflict};

use super::super::*;

pub async fn _add(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if result.args.is_empty() {
        return Ok(());
    }

    // SAFETY: The Command filter will not allow processing of a "None" value.
    let text = unsafe {
        message
            .text()
            .or_else(|| message.caption())
            .unwrap_unchecked()
    };
    let ArgsResult::Some([a1, a2], _) = result.args::<2>(text) else {
        return Ok(());
    };

    let Ok(rank) = text[a2].parse::<u8>() else {
        return Ok(());
    };

    let Ok(ids) = text[a1].parse::<i16>() else {
        return Ok(());
    };

    let module = ModuleChecker::new(&bot, &db);
    let true = module
        .check::<25>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    let model = chat_module::ActiveModel {
        chat_ids: Set(message
            .chat()
            .id()
            .into()),
        module_ids: Set(ids),
        rank: Set(rank as i16),
        ..Default::default()
    };

    let _ = ChatModule::insert(model)
        .on_conflict(
            OnConflict::columns([
                chat_module::Column::ChatIds,
                chat_module::Column::UserIds,
                chat_module::Column::ModuleIds,
            ])
            .do_nothing()
            .to_owned(),
        )
        .exec(&db)
        .await;

    Ok(())
}

pub async fn _delete(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if result.args.is_empty() {
        return Ok(());
    }

    // SAFETY: The Command filter will not allow processing of a "None" value.
    let text = unsafe {
        message
            .text()
            .or_else(|| message.caption())
            .unwrap_unchecked()
    };
    let ArgsResult::Some([a1], _) = result.args::<1>(text) else {
        return Ok(());
    };

    let Ok(_ids) = text[a1].parse::<i16>() else {
        return Ok(());
    };

    let module = ModuleChecker::new(&bot, &db);
    let true = module
        .check::<25>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    Ok(())
}

async fn edit_show_core(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(arch): Extension<AttachResult>,
    Extension(result): Extension<CommandResult>,
    show: bool,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    let module = ModuleChecker::new(&bot, &db);
    let chat_ids = arch.chat_ids;

    let true = module
        .check::<22>(ModuleAccess::CustomM(&message, chat_ids.0))
        .await
    else {
        return Ok(());
    };

    let model = setting::ActiveModel {
        chat_ids: Set(chat_ids),
        access_command: Set(show),
        ..Default::default()
    };

    let _ = ChatSetting::insert(model)
        .on_conflict(
            OnConflict::column(setting::Column::ChatIds)
                .update_column(setting::Column::AccessCommand)
                .to_owned(),
        )
        .exec(&db)
        .await;

    if show {
        bot.send(
            JuzoAnswer::message(&message)
                .text(format!("{0} Оповещения о доступности модулей включены", smail_tick(true))),
        )
        .await?;
    } else {
        bot.send(
            JuzoAnswer::message(&message)
                .text(format!("{0} Оповещения о доступности модулей отключены", smail_cross(true))),
        )
        .await?;
    }

    Ok(())
}

pub async fn edit_show_true(
    bot: Bot,
    message: Message,
    db: Extension<DbConn>,
    arch: Extension<AttachResult>,
    result: Extension<CommandResult>,
) -> HandlerResult<()> {
    edit_show_core(bot, message, db, arch, result, true).await
}

pub async fn edit_show_false(
    bot: Bot,
    message: Message,
    db: Extension<DbConn>,
    arch: Extension<AttachResult>,
    result: Extension<CommandResult>,
) -> HandlerResult<()> {
    edit_show_core(bot, message, db, arch, result, false).await
}
