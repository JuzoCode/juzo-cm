use juzo_core::{
    common::emojis::{smail_pensil, smail_tick},
    db::user::{anketa, prelude::UserAnketa},
    domain::UserModel,
    gender,
};
use sea_orm::{ActiveValue::Set, EntityTrait, sea_query::OnConflict};
use telers::{
    methods::{SendMessage, SendRichMessage},
    types::InputRichMessage,
};

use super::super::*;

pub async fn edit(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if result.args.is_empty() {
        return Ok(());
    };

    let module = ModuleChecker::new(&bot, &db);

    let text = unsafe {
        message
            .text()
            .or_else(|| message.caption())
            .unwrap_unchecked()
    };
    let ArgsResult::Some([a1], _) = result.args::<1>(text) else {
        return Ok(());
    };

    if message
        .chat()
        .title()
        .is_some()
    {
        let true = module
            .check::<28>(ModuleAccess::M(&message))
            .await
        else {
            return Ok(());
        };
    }

    let system = text[a1].as_bytes();
    let len = system.len();

    if !matches!(len, 6 | 12 | 14) {
        return Ok(());
    }

    // SAFETY: TBA will never return None in message.from().
    let iam: UserModel = unsafe {
        message
            .from()
            .unwrap_unchecked()
    }
    .into();

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

    let Some(g) = GENDERS
        .iter()
        .find_map(|&(name, gender)| (name == &*out).then_some(gender))
    else {
        bot.send(JuzoAnswer::<SendRichMessage>::to(&message).rich_message(
            InputRichMessage::new().html(format!(
                "{0} Увы, выбора всего два — <tg-button type='copy_text' text='.мой пол мужской' \
                 >мужской</tg-button> и <tg-button type='copy_text' text='.мой пол женский' \
                 style='danger'>женский</tg-button>. Не определились? Берите <tg-button \
                 type='copy_text' text='.мой пол другой' style='success'>другой</tg-button>, \
                 почти как ламинат.",
                smail_pensil(true),
            )),
        ))
        .await?;
        return Ok(());
    };

    UserModel::edit_gender(iam.ids, g);

    let model = anketa::ActiveModel {
        user_ids: Set(iam.ids),
        gender: Set(g),
        is_user: Set(iam.is_user),
        ..Default::default()
    };

    let Ok(_) = UserAnketa::insert(model)
        .on_conflict(
            OnConflict::column(anketa::Column::UserIds)
                .update_column(anketa::Column::Gender)
                .to_owned(),
        )
        .exec(&db)
        .await
    else {
        bot.send(JuzoAnswer::<SendMessage>::to(&message).text(format!(
            "{0} Пол не был изменён, но он будет, как минимум, закэшированным.",
            smail_pensil(true)
        )))
        .await?;
        return Ok(());
    };

    let g1 = gender!(g => [
        "женский' style='danger'>женский",
        "мужской'>мужской",
        "другой' style='success'>другой"
    ]);

    bot.send(JuzoAnswer::<SendRichMessage>::to(&message).rich_message(
        InputRichMessage::new().html(format!(
            "{0} <a href='{1}'>Вы</a> изменили свой пол на <tg-button type='copy_text' text='.мой \
             пол {g1}</tg-button>",
            smail_tick(true),
            iam.link(),
        )),
    ))
    .await?;

    Ok(())
}

const GENDERS: [(&[u8], i16); 5] = [
    ("муж".as_bytes(), 1),
    ("жен".as_bytes(), 0),
    ("мужской".as_bytes(), 1),
    ("женский".as_bytes(), 0),
    ("другой".as_bytes(), 2),
];
