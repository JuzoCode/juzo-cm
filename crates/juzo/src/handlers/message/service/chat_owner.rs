use telers::types::MessageChatOwnerLeft;

use super::super::{Bot, HandlerResult, JuzoAnswer};

pub async fn left(bot: Bot, message: MessageChatOwnerLeft) -> HandlerResult<()> {
    bot.send(JuzoAnswer::message(&message.into()).text("Meow"))
        .await?;

    Ok(())
}
