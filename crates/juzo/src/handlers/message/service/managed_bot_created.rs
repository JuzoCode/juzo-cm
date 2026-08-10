use juzo_core::{application::FullName, common::emojis::smail_tick};
use telers::types::MessageManagedBotCreated;

use super::super::{Bot, HandlerResult, JuzoAnswer};

pub async fn send(bot: Bot, message: MessageManagedBotCreated) -> HandlerResult<()> {
    let managed = &message
        .managed_bot_created
        .bot;

    let full_name = FullName::new(
        &managed.first_name,
        managed
            .last_name
            .as_deref(),
    );

    let text = format!(
        "{0} <a href='https://t.me/{1}'>{2}</a> успешно установлен как несанкционированный клон \
         Джузо.\n<blockquote>Устанавливать клона в чаты можете только Вы</blockquote>",
        smail_tick(true),
        // SAFETY: bots always have a username when creating
        unsafe {
            managed
                .username
                .as_deref()
                .unwrap_unchecked()
        },
        full_name.as_str()
    );

    bot.send(JuzoAnswer::message(&message.into()).text(text))
        .await?;

    Ok(())
}
