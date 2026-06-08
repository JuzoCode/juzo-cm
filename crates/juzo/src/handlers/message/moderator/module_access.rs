// use juzo_core::common::emojis::smail_pensil;
use sea_orm::DbConn;

use super::super::*;

pub async fn add(
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
    let _args = result.args::<1>(text);

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
    let _args = result.args::<1>(text);

    Ok(())
}

// Через два месяца все сделаю, наверно =))) И то, что открытие 28 августа, меня не волнует
