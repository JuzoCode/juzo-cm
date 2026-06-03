use telers::types::MessageChatOwnerLeft;

use super::super::{Bot, HandlerResult, JuzoAnswer};

pub async fn send(bot: Bot, message: MessageChatOwnerLeft) -> HandlerResult<()> {
    bot.send(JuzoAnswer::message(&message.into()))
        .await?;

    Ok(())
}
