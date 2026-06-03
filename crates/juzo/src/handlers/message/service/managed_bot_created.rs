use core::fmt::Write;

use juzo_core::{application::FullName, common::emojis::smail_tick};
use telers::types::MessageManagedBotCreated;

use super::super::{Bot, HandlerResult, JuzoAnswer};

pub async fn send(bot: Bot, message: MessageManagedBotCreated) -> HandlerResult<()> {
    let managed = &message
        .managed_bot_created
        .bot;

    let mut text = FullName::new(
        &managed.first_name,
        managed
            .last_name
            .as_deref(),
    )
    // SAFETY: bots always have a username when creating.
    .with_name(|full_name| unsafe {
        format!(
            "{0} <a href='https://t.me/{1}'>{full_name}</a> успешно установлен как \
             несанкционированный клон Джузо.\n",
            smail_tick(true),
            managed
                .username
                .as_deref()
                .unwrap_unchecked(),
        )
    });
    let _ = write!(text, "<blockquote>Устанавливать клона в чаты можете только Вы</blockquote>");

    bot.send(JuzoAnswer::message(&message.into()).text(text))
        .await?;

    Ok(())
}
