use juzo_core::db::chat::{chat_module, prelude::ChatModule};
use sea_orm::{DbConn, EntityTrait, Set, sea_query::OnConflict};

use super::super::*;

pub async fn add(
    _bot: Bot,
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
    let Some([a1, a2]) = result.args::<2>(text) else {
        return Ok(());
    };

    let Ok(rank) = text[a2].parse::<u8>() else {
        return Ok(());
    };

    let Ok(ids) = text[a1].parse::<i16>() else {
        return Ok(());
    };

    let model = chat_module::ActiveModel {
        chat_ids: Set(message
            .chat()
            .id()
            .into()),
        module_ids: Set(ids),
        rank: Set(rank),
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
    _bot: Bot,
    message: Message,
    Extension(_db): Extension<DbConn>,
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
    let Some([a1]) = result.args::<1>(text) else {
        return Ok(());
    };

    let Ok(ids) = text[a1].parse::<i16>() else {
        return Ok(());
    };

    Ok(())
}

// Через два месяца все сделаю, наверно =))) И то, что открытие 28 августа, меня не волнует
