use juzo_core::common::emojis::smail_pensil;
use telers::methods::SetChatTitle;

use super::super::*;

pub async fn set(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if result.args.is_empty() {
        return Ok(());
    }

    let module = ModuleChecker::new(&bot, &db);
    let true = module
        .check::<22>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    // SAFETY: The Command filter will not allow processing of a "None" value.
    let title = &unsafe {
        message
            .text()
            .or_else(|| message.caption())
            .unwrap_unchecked()
    }[result.args];

    if title
        .chars()
        .nth(128)
        .is_some()
    {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Название чата превышает допустимый лимит в 128 символов.",
            smail_pensil(true)
        )))
        .await?;
        return Ok(());
    };

    bot.send(SetChatTitle::new(message.chat().id(), title))
        .await?;

    Ok(())
}
