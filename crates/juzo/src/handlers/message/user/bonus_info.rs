use core::fmt::Write;

use chrono::Utc;
use juzo_core::{
    application::{BonusResult, JuzoBonus},
    common::{
        emojis::{smail_pensil, smail_premium, smail_sweets, smail_tick},
        inflection::{plur_sweets, plur_sweets_y},
    },
    config::PRICE_PREMIUM,
    db::chat::{prelude::ChatSetting, setting},
    domain::{AttachResult, TimeFormatted, enums::BonusLog},
    payloads::{
        base::PayloadType,
        callback::{Callback, CallbackKind},
    },
};
use sea_orm::{ActiveValue::Set, EntityTrait, sea_query::OnConflict};
use telers::{
    methods::{SendMessage, SendRichMessage},
    types::{InlineKeyboardButton, InlineKeyboardMarkup, InputRichMessage},
};

use super::super::*;

async fn vip_core(
    bot: Bot,
    message: Message,
    db: DbConn,
    result: CommandResult,
) -> HandlerResult<()> {
    let module = ModuleChecker::new(&bot, &db);

    // SAFETY: The Command filter will not allow processing of a "None" value.
    let text = unsafe {
        message
            .text()
            .or_else(|| message.caption())
            .unwrap_unchecked()
    };
    let args = result.args::<1>(text);

    let value: u8 = match args {
        ArgsResult::Some([a1], _) => {
            let Ok(value) = text[a1].parse() else {
                return Ok(());
            };
            value
        }
        ArgsResult::None => unsafe { core::intrinsics::unreachable() },
        ArgsResult::Unk => return Ok(()),
    };

    let true = module
        .check::<49>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    if value < 10 {
        bot.send(JuzoAnswer::<SendMessage>::to(&message).text(format!(
            "{0} Цена VIP-статуса не должна быть ниже {1}",
            smail_pensil(true),
            plur_sweets(10u8),
        )))
        .await?;
        return Ok(());
    }

    let model = setting::ActiveModel {
        chat_ids: Set(message
            .chat()
            .id()
            .into()),
        vip_price: Set(value as i16),
        ..Default::default()
    };

    let _ = ChatSetting::insert(model)
        .on_conflict(
            OnConflict::column(setting::Column::ChatIds)
                .update_column(setting::Column::VipPrice)
                .to_owned(),
        )
        .exec(&db)
        .await;

    let vibe = match value {
        0..=90 => "Раз VIP-статус для нормальных цен, он теперь составляет",
        91..=170 => "Ладно, золотая середина для VIP-статуса, теперь цена составляет",
        171..=255 => "<b>Тут типа богатые сидят?</b> Ну хорошо, цена VIP-статуса составляет",
    };

    bot.send(JuzoAnswer::<SendMessage>::to(&message).text(format!(
        "{0} {vibe} {1} {2}",
        smail_tick(true),
        smail_sweets(true),
        plur_sweets_y(value),
    )))
    .await?;

    Ok(())
}

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
        Callback::new(CallbackKind::PremiumBuy1).encode(
            message
                .from()
                .unwrap_unchecked()
                .id as u64,
        )
    };

    let mut button = InlineKeyboardButton::new("").callback_data(data);

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

pub async fn vip(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return vip_core(bot, message, db, result).await;
    }

    let module = ModuleChecker::new(&bot, &db);
    let bonus = JuzoBonus::new(&db);

    let true = module
        .check::<50>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    // let (smail, name, removed) = match bonus
    let _ = match bonus
        .chat_ids(message.chat().id())
        .one(BonusLog::Vip)
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

    Ok(())
}
