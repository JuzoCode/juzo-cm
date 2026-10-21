use telers::{methods::SendMessage, types::MessageChatOwnerChanged};

use super::super::{Bot, HandlerResult, JuzoAnswer};

pub async fn changed(bot: Bot, message: MessageChatOwnerChanged) -> HandlerResult<()> {
    bot.send(JuzoAnswer::<SendMessage>::to(&message.into()).text("Meow"))
        .await?;

    Ok(())
}
