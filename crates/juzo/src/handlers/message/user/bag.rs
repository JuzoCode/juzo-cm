use core::fmt::Write;

use juzo_core::{
    application::{JuzoAnswer, UserIndex, UserModel},
    common::{
        emojis::{
            smail_bag, smail_gold, smail_jcoin, smail_pensil, smail_score, smail_stars,
            smail_sweets,
        },
        inflection::{GOLD_TEXT, JUZO_COIN_TEXT},
        tools::time::holiday_choice,
    },
    db::user::{balance, prelude::UserBalance},
};
use sea_orm::{DbConn, EntityTrait};

use super::super::*;

/// добавить в бизнес мод
pub async fn show(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    let user_ind = UserIndex::new(&bot, &db);

    // SAFETY: The Command filter will not allow processing of a "None" value.
    let text = unsafe {
        message
            .text()
            .or_else(|| message.caption())
            .unwrap_unchecked()
    };
    let args = result.args::<1>(text);

    let user: UserModel = match args {
        Some([a1]) => {
            let Ok(found_user) = user_ind
                .search_user(&text[a1])
                .await
            else {
                return Ok(());
            };
            found_user
        }
        None => unsafe {
            if let Some(r) = message.reply_to_message() {
                r.from()
                    .unwrap_unchecked()
                    .into()
            } else if message
                .business_connection_id()
                .is_some()
            {
                message.chat().into()
            } else {
                message
                    .from()
                    .unwrap_unchecked()
                    .into()
            }
        },
    };

    let balance = UserBalance::find_by_id(user.ids)
        .one(&db)
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| balance::Model::new(user.ids));

    if !balance.show {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} {{}} скрыл мешок.<blockquote>Заслужите его милость, и тогда <b>просите открыть \
             его!</b> =)</blockquote>",
            smail_pensil(true)
        )))
        .await?;
        return Ok(());
    }

    let mut text = String::new();

    let _ = writeln!(text, "{0} <b>В мешке {{}}</b>.<blockquote expandable>", smail_bag(true));

    let _ = writeln!(
        text,
        "{0} {1} {2} {3} {4}\n{5} {6} {7} {8} {9}",
        smail_sweets(true),
        balance.sweets,
        smail_gold(true),
        balance.gold,
        GOLD_TEXT,
        smail_jcoin(true),
        balance.coins,
        JUZO_COIN_TEXT,
        smail_stars(true),
        balance.asterisks,
    );

    if balance.score > 0 {
        let _ = writeln!(text, "{0} {1}", smail_score(true), balance.score);
    }

    let _ = write!(
        text,
        "</blockquote>\n💬 Запасы {0} можно пополнить, введя команду <code>купить {{число}}</code>",
        holiday_choice(&"леденцов", &"мандаринок", &"тыковок")
    );

    bot.send(JuzoAnswer::message(&message).text(text))
        .await?;

    Ok(())
}
