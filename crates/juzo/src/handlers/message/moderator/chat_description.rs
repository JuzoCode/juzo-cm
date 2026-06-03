use juzo_core::common::emojis::{smail_pensil, smail_tick};
use telers::methods::SetChatDescription;

use super::super::*;

pub async fn set(
    bot: Bot,
    message: Message,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    let first_line = result.first_line;
    if first_line.is_empty() {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Описание чата не указано. <blockquote>💬 Если Вы хотите удалить описание, \
             введите команду <code>-описание чата</code></blockquote>",
            smail_pensil(true)
        )))
        .await?;
        return Ok(());
    }

    let text = unsafe {
        &message
            .text()
            .unwrap_unchecked()[first_line]
    };

    if text
        .chars()
        .nth(255)
        .is_some()
    {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Описание превышает допустимый лимит 255 символов.",
            smail_pensil(true)
        )))
        .await?;
        return Ok(());
    }

    bot.send(SetChatDescription::new(message.chat().id()).description(text))
        .await?;

    bot.send(
        JuzoAnswer::message(&message)
            .text(format!("{0} Описание успешно изменено", smail_tick(true))),
    )
    .await?;

    Ok(())
}

pub async fn delete(
    bot: Bot,
    message: Message,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    bot.send(SetChatDescription::new(message.chat().id()).description_option::<&str>(None))
        .await?;

    bot.send(
        JuzoAnswer::message(&message)
            .text(format!("{0} Описание успешно удалено", smail_tick(true))),
    )
    .await?;

    Ok(())
}
