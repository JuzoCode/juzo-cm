use core::fmt::Write;

use chrono::Utc;
use juzo_core::{
    application::{BonusResult, JuzoBonus},
    common::{
        emojis::{smail_pensil, smail_premium, smail_sweets},
        inflection::plur_sweets,
    },
    config::PRICE_PREMIUM,
    domain::{AttachResult, TimeFormatted, enums::BonusLog},
    payloads::{
        base::PackedPayload,
        callback::{Callback, CallbackKind},
    },
};
use telers::{
    methods::SendRichMessage,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, InputRichMessage},
};

// use sea_orm::{ActiveValue::Set, EntityTrait, sea_query::OnConflict};
use super::super::*;

// async fn info_core(
//     bot: Bot,
//     message: Message,
//     Extension(db): Extension<DbConn>,
//     Extension(result): Extension<CommandResult>,
//     bonus: BonusLog,
// ) -> HandlerResult<()> {

//     Ok(())
// }

pub async fn premium(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(arch): Extension<AttachResult>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    let module = ModuleChecker::new(&bot, &db);
    let bonus = JuzoBonus::new(&db);

    let chat_ids = arch.chat_ids.0;

    let true = module
        .check::<47>(ModuleAccess::CustomM(&message, chat_ids))
        .await
    else {
        return Ok(());
    };

    let (smail, name, removed) = match bonus
        .chat_ids(chat_ids)
        .one(BonusLog::Premium)
        .await
    {
        BonusResult::Expired(removed) => (
            smail_pensil(true),
            "'>Истёкший Premium",
            Some(("Истёк в", TimeFormatted::<Utc>::new(removed).from())),
        ),
        BonusResult::Active(removed) => (
            smail_premium(true),
            "success'>Premium",
            Some(("Действует до", TimeFormatted::<Utc>::new(removed).until())),
        ),
        BonusResult::Unk => ("🗓", "danger'>Обычный", None),
    };

    let mut text = format!(
        "{smail} Статус чата: <tg-button type='url' url='https://teletype.in/@juzo_cm/premium' \
         style='{name}</tg-button>"
    );

    // SAFETY: TBA will never return None in message.from().
    let data = unsafe {
        PackedPayload::new(
            message
                .from()
                .unwrap_unchecked()
                .id as u64,
            Callback::new(CallbackKind::PremiumBuy1),
        )
    };

    let mut button = InlineKeyboardButton::new("").callback_data(data.encode().as_str());

    if let Some((label, time)) = removed {
        let _ = write!(
            text,
            "<br><b>{label}</b> {0} ({time})<br><br>💬 Продлить на месяц, заплатив <b>{1}</b>, \
             можно командой <code>купить премиум</code>",
            time.date()
                .with_timezone(&chrono::Local)
                .format("%d.%m.%Y, %H:%M"),
            plur_sweets(PRICE_PREMIUM),
        );
        button = button
            .text(format!("{0} Продлить Premium-статус", smail_premium(false)))
            .style("primary");
    } else {
        let _ = write!(
            text,
            "<br><br>💬 Повысить до уровня <tg-button type='url' url='https://teletype.in/@juzo_cm/premium' \
             style='success'>Premium</tg-button> можно командой <code>купить премиум</code>. \
             <br>{0} <b>Цена: </b>{1} в месяц",
            smail_sweets(true),
            plur_sweets(PRICE_PREMIUM),
        );
        button = button
            .text(format!("{0} Купить Premium-статус", smail_premium(false)))
            .style("success");
    }

    bot.send(
        JuzoAnswer::<SendRichMessage>::to(&message)
            .rich_message(InputRichMessage::new().html(text))
            .reply_markup(InlineKeyboardMarkup::new([[button]])),
    )
    .await?;

    Ok(())
}
