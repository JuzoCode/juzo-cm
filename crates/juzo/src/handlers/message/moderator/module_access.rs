use juzo_core::db::chat::{chat_module, prelude::ChatModule};
use sea_orm::{EntityTrait, Set, sea_query::OnConflict};

use super::super::*;

pub async fn add(
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

pub async fn delete(
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

// Через два месяца все сделаю, наверно =))) И то, что открытие 28 августа, меня не волнует
