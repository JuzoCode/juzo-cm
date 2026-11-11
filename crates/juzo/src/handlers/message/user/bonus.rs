use core::fmt::Write;

use chrono::{Timelike, Utc};
use juzo_core::{
    application::{BonusResult, JuzoBonus, ParseTgLink, UserIndex, UserModel},
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
    methods::{SendMessage, SendRichMessage},
    types::{InlineKeyboardButton, InlineKeyboardMarkup, InputRichMessage},
};

// use sea_orm::{ActiveValue::Set, EntityTrait, sea_query::OnConflict};
use super::super::*;

async fn buy_core<'a>(
    message: &'a Message,
    result: CommandResult,
    user_ind: UserIndex<'a>,
) -> Option<(u8, UserModel)> {
    let text = unsafe {
        message
            .text()
            .or_else(|| message.caption())
            .unwrap_unchecked()
    };
    let args = result.args::<2>(text);

    match args {
        ArgsResult::Some([a1, a2], 2) => {
            let Ok(value) = text[a1].parse() else {
                return None;
            };
            let Ok(found_user) = user_ind
                .search_user(&text[a2])
                .await
            else {
                return None;
            };
            Some((value, found_user))
        }
        ArgsResult::Some([a1, _], 1) => unsafe {
            if let Some(link) = ParseTgLink::new(&text[a1]) {
                let Ok(found_user) = user_ind
                    .fetch_user(link)
                    .await
                else {
                    return None;
                };
                Some((1, found_user))
            } else if !a1.is_empty() {
                let Ok(value) = text[a1].parse() else {
                    return None;
                };
                let found_user = if let Some(r) = message.reply_to_message() {
                    // SAFETY: TBA will never return None in message.from().
                    r.from()
                        .unwrap_unchecked()
                        .into()
                } else {
                    message
                        .from()
                        .unwrap_unchecked()
                        .into()
                };
                Some((value, found_user))
            } else {
                let found_user = if let Some(r) = message.reply_to_message() {
                    // SAFETY: TBA will never return None in message.from().
                    r.from()
                        .unwrap_unchecked()
                        .into()
                } else {
                    message
                        .from()
                        .unwrap_unchecked()
                        .into()
                };
                Some((1, found_user))
            }
        },
        // SAFETY: TBA will never return None in message.from().
        ArgsResult::None => unsafe {
            let found_user = if let Some(r) = message.reply_to_message() {
                r.from()
                    .unwrap_unchecked()
                    .into()
            } else {
                message
                    .from()
                    .unwrap_unchecked()
                    .into()
            };
            Some((1, found_user))
        },
        _ => return None,
    }
}

pub async fn buy_premium(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    let user_ind = UserIndex::new(&bot, &db);
    let module = ModuleChecker::new(&bot, &db);

    let Some((value, _)) = buy_core(&message, result, user_ind).await else {
        return Ok(());
    };

    let true = module
        .check::<47>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    bot.send(JuzoAnswer::<SendMessage>::to(&message).text(format!("{value}",)))
        .await?;

    Ok(())
}

pub async fn buy_vip(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    let user_ind = UserIndex::new(&bot, &db);
    let module = ModuleChecker::new(&bot, &db);

    let Some((_value, _user)) = buy_core(&message, result, user_ind).await else {
        return Ok(());
    };

    let true = module
        .check::<50>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    Ok(())
}

// async fn info_core(
//     bot: Bot,
//     message: Message,
//     Extension(db): Extension<DbConn>,
//     Extension(result): Extension<CommandResult>,
//     bonus: BonusLog,
// ) -> HandlerResult<()> {

//     Ok(())
// }

pub async fn info_premium(
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

    // SAFETY: ¯\_(ツ)_/¯
    let now = unsafe {
        Utc::now()
            .with_nanosecond(0)
            .unwrap_unchecked()
    };

    let (smail, name, removed) = match bonus
        .chat_ids(chat_ids)
        .one(BonusLog::Premium)
        .await
    {
        BonusResult::Expired(removed) => (
            smail_pensil(true),
            "'>Истёкший Premium",
            Some(("Истёк в", TimeFormatted::from(removed, now))),
        ),
        BonusResult::Active(removed) => (
            smail_premium(true),
            "success'>Premium",
            Some(("Действует до", TimeFormatted::until(removed, now))),
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
            "<br><b>{label}</b> {0} ({time})<br><br>💬 Продлить на месяц, заплати в <b>{1}</b>, \
             можно командой <code>купить премиум</code>",
            time.datetime
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
