use juzo_core::{
    common::{
        emojis::smail_pensil,
        tools::time::{HOLIDAY, holiday_choice},
    },
    domain::UserIds,
};
use telers::methods::SendMessage;

use super::super::*;

pub async fn yes(
    bot: Bot,
    message: Message,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if result.args.is_empty() {
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

    // SAFETY: The Command filter will not allow processing of a "None" value.
    let text = unsafe {
        message
            .text()
            .or_else(|| message.caption())
            .unwrap_unchecked()
    };
    let ArgsResult::Some([a1], _) = result.args::<1>(text) else {
        return Ok(());
    };

    let Ok(value) = u8::from_str_radix(&text[a1], 2) else {
        return Ok(());
    };

    if !matches!(value, 0 | 0b100 | 0b101 | 0b110) {
        bot.send(JuzoAnswer::<SendMessage>::to(&message).text(format!(
            "{0} 0b<code>100</code>, 0b<code>101</code>, 0b<code>110</code>",
            smail_pensil(true)
        )))
        .await?;
        return Ok(());
    }

    unsafe { core::intrinsics::volatile_store(&raw mut HOLIDAY, value) }

    let (smail, day) = holiday_choice(
        &(
            "<tg-emoji emoji-id='5449389114287765641'>📝</tg-emoji>",
            "ничего?...",
        ),
        &(
            "<tg-emoji emoji-id='5447223681971430605'>🎄</tg-emoji>",
            "предвкушение Нового года!",
        ),
        &(
            "<tg-emoji emoji-id='5447113928377152650'>🎃</tg-emoji>",
            "Хэллоуин!",
        ),
    );

    bot.send(
        JuzoAnswer::<SendMessage>::to(&message)
            .text(format!("{smail} Джузо будет отмечать <b>{day}</b>",)),
    )
    .await?;

    Ok(())
}
