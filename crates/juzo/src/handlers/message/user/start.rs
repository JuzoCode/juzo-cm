use core::fmt::Write;

use sea_orm::{ConnectionTrait, raw_sql};
use telers::types::{Chat, InlineKeyboardButton, InlineKeyboardMarkup};

use super::super::*;

/// дописать старт
pub async fn yes(
    _bot: Bot,
    _message: Message,
    Extension(_db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    Ok(())
}

/// добавить агентов
pub async fn help(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    if let Chat::Private(_) = message.chat() {
        return yes(bot, message, Extension(db), Extension(result)).await;
    }

    let module = ModuleChecker::new(&bot, &db);
    let true = module
        .check::<35>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    let Ok(rows) = db
        .query_all_raw(raw_sql!(
            Postgres,
            r#"
            SELECT
                u.full_name,
                'https://t.me/' || u.username AS link
            FROM a
            INNER JOIN u
                ON u.user_ids = a.user_ids
               AND u.username IS NOT NULL
            WHERE a.show AND (a.agent OR a.spam);
            "#
        ))
        .await
    else {
        return Ok(());
    };

    let mut first = true;
    let mut text = String::from(
        "<tg-emoji emoji-id='5471999544415783597'>📖</tg-emoji> Помощь по боту<b> \
         <a href='https://juzo_cm_bot.t.me'>Juzo | Чат-менеджер</a></b>",
    );

    for row in rows {
        let (Ok(full_name), Ok(link)) =
            (row.try_get::<String>("", "full_name"), row.try_get::<String>("", "link"))
        else {
            continue;
        };

        if first {
            text.push_str("\n\n<b>Агенты поддержки:</b> (им можно задать вопросы)");
            first = false;
        }

        let _ = write!(text, "\n<a href='{link}'>{full_name}</a>");
    }

    text.push_str(
        "\n\n\
         <tg-emoji emoji-id='5431736674147114227'>🗂</tg-emoji> Список всех команд \
         <a href='https://teletype.in/@juzo_cm/commands'>с их описанием</a>.\n\
         <tg-emoji emoji-id='5258513401784573443'>👥</tg-emoji> Официальный \
         <a href='https://juzo_cm_chat.t.me'>чат поддержки бота</a>.\n\
         <tg-emoji emoji-id='5260268501515377807'>📢</tg-emoji> <a href='https://juzo_cm.t.me'>Канал</a> \
         с важными новостями.\n\
         <tg-emoji emoji-id='5373098009640836781'>📚</tg-emoji> <a href='https://juzosup.t.me'>Канал</a> \
         с полезными статьями.",
    );

    let keyboard = InlineKeyboardMarkup::new([
        [
            InlineKeyboardButton::new("🗂 Команды").url("https://teletype.in/@juzo_cm/commands"),
            InlineKeyboardButton::new("👥 Чат поддержки").url("https://juzo_cm_chat.t.me"),
        ],
        [
            InlineKeyboardButton::new("📚 Полезные статьи").url("https://juzosup.t.me"),
            InlineKeyboardButton::new("📢 Канал бота").url("https://juzo_cm.t.me"),
        ],
    ]);

    bot.send(
        JuzoAnswer::message(&message)
            .text(text)
            .reply_markup(keyboard),
    )
    .await?;

    Ok(())
}
