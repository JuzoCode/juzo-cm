use juzo_core::{
    common::emojis::{smail_pensil, smail_tick},
    domain::UserIds,
};
use telers::{
    methods::{SendMessage, SetMyProfilePhoto},
    types::{InputFile, InputProfilePhotoStatic},
};

use super::super::*;

pub async fn yes(
    bot: Bot,
    message: Message,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    // SAFETY: TBA will never return None in message.from().
    let my_ids: UserIds = unsafe {
        message
            .from()
            .unwrap_unchecked()
    }
    .id
    .into();

    if !my_ids.is_creator_bot() {
        return Ok(());
    }

    let Some(photo) = message
        .photo()
        .and_then(|p| p.last())
    else {
        bot.send(JuzoAnswer::<SendMessage>::to(&message).text(format!(
            "{0} Ради приличия прикрепите к сообщению какую-нибудь фотографию.",
            smail_pensil(true)
        )))
        .await?;
        return Ok(());
    };

    let bytes = bot
        .download(photo)
        .await?
        .bytes()
        .await?;

    bot.send(SetMyProfilePhoto::new(InputProfilePhotoStatic::new(
        InputFile::buffered(bytes),
    )))
    .await?;

    bot.send(JuzoAnswer::<SendMessage>::to(&message).text(format!(
        "{0} У Джузо была сменена аватарка в профиле",
        smail_tick(true),
    )))
    .await?;

    Ok(())
}
