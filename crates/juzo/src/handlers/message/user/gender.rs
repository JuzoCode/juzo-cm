use juzo_core::common::emojis::smail_tick;
use telers::methods::SendMessage;

use super::super::*;

pub async fn edit(
    bot: Bot,
    message: Message,
    Extension(_db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if result.args.is_empty() {
        return Ok(());
    };

    let text = unsafe {
        message
            .text()
            .or_else(|| message.caption())
            .unwrap_unchecked()
    };
    let ArgsResult::Some([a1], _) = result.args::<1>(text) else {
        return Ok(());
    };

    let system = text[a1].as_bytes();
    let len = system.len();

    if !matches!(len, 6 | 12 | 14) {
        return Ok(());
    }

    let mut buf = [0u8; 14];
    let out = &mut buf[..len];
    let mut i = 0;
    while i < len {
        let (b0, b1) = (system[i], system[i + 1]);
        (out[i], out[i + 1]) = match (b0, b1) {
            (0xD0, 0x90..=0x9F) => (0xD0, b1 + 0x20),
            (0xD0, 0xA0..=0xAF) => (0xD1, b1 - 0x20),
            _ => (b0, b1),
        };
        i += 2;
    }

    let Some(_gender) = GENDERS
        .iter()
        .find_map(|&(name, gender)| (name == &*out).then_some(gender))
    else {
        return Ok(());
    };

    bot.send(JuzoAnswer::<SendMessage>::to(&message).text(format!("{0} Был сменен на ", smail_tick(true))))
        .await?;

    Ok(())
}

const GENDERS: [(&[u8], u8); 5] = [
    ("муж".as_bytes(), 1),
    ("жен".as_bytes(), 0),
    ("мужской".as_bytes(), 1),
    ("женский".as_bytes(), 0),
    ("другой".as_bytes(), 2),
];
