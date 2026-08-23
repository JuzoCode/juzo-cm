use core::fmt::Write;

use juzo_core::{
    application::{UserIndex, UserModel},
    common::{
        emojis::{
            smail_asterisks, smail_bag, smail_gold, smail_jcoin, smail_pensil, smail_score,
            smail_sweets,
        },
        inflection::{plur_asterisks, plur_gold, plur_jcoin, plur_score, plur_sweets},
        // tools::time::holiday_choice,
    },
    db::user::{balance, prelude::UserBalance},
};
use sea_orm::{EntityTrait, raw_sql, sea_query::prelude::rust_decimal::prelude::ToPrimitive};

use super::super::*;

/// добавить в бизнес мод
pub async fn show(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    let user_ind = UserIndex::new(&bot, &db);
    let module = ModuleChecker::new(&bot, &db);

    // SAFETY: The Command filter will not allow processing of a "None" value.
    let text = unsafe {
        message
            .text()
            .or_else(|| message.caption())
            .unwrap_unchecked()
    };
    let args = result.args::<1>(text);

    let user: UserModel = match args {
        ArgsResult::Some([a1], _) => {
            let Ok(found_user) = user_ind
                .search_user(&text[a1])
                .await
            else {
                return Ok(());
            };
            found_user
        }
        // SAFETY: TBA will never return None in message.from().
        ArgsResult::None => unsafe {
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
        ArgsResult::Unk => return Ok(()),
    };

    let access = module
        .check::<31>(ModuleAccess::M(&message))
        .await;
    if !access {
        return Ok(());
    }

    // SAFETY: TBA will never return None in message.from().
    let my_ids = unsafe {
        message
            .from()
            .unwrap_unchecked()
    }
    .id;

    let balance = UserBalance::find()
        .from_raw_sql(raw_sql!(
            Postgres,
            r#"
            SELECT
                user_ids,
                CASE
                    WHEN user_ids = {my_ids} THEN true
                    ELSE show
                END AS show,
                asterisks,
                coins,
                sweets,
                CASE
                    WHEN user_ids = {my_ids} THEN score
                    ELSE 0
                END AS score,
                gold
            FROM u2
            WHERE user_ids = {user.ids};
            "#
        ))
        .one(&db)
        .await
        .unwrap()
        .unwrap_or_else(|| balance::Model::new(user.ids));

    if !balance.show {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} <b><a href='{1}'>{2}</a></b> скрыл мешок.<blockquote>Заслужите его милость, и \
             тогда <b>просите открыть его!</b> =)</blockquote>",
            smail_pensil(true),
            user.link(),
            user.full_name(),
        )))
        .await?;
        return Ok(());
    }

    if balance.is_empty() {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} <b>В мешке <a href='{1}'>{2}</a></b></b> пустеет так, что не осталось даже пыли",
            smail_bag(true),
            user.link(),
            user.full_name(),
        )))
        .await?;
        return Ok(());
    }

    let mut text = format!(
        "{0} <b>В мешке <a href='{1}'>{2}</a></b>.<blockquote expandable>",
        smail_bag(true),
        user.link(),
        user.full_name(),
    );

    let _ = writeln!(
        text,
        "{0} {1} {2} {3}\n{4} {5} {6} {7}",
        smail_sweets(true),
        unsafe {
            plur_sweets(
                balance
                    .sweets
                    .to_u32()
                    .unwrap_unchecked(),
            )
        },
        smail_gold(true),
        plur_gold(balance.gold),
        smail_jcoin(true),
        plur_jcoin(balance.coins),
        smail_asterisks(true),
        plur_asterisks(balance.asterisks)
    );

    if balance.score > 0 {
        let _ = writeln!(text, "{0} {1}", smail_score(true), plur_score(balance.score));
    }

    text.push_str("</blockquote>");

    // let _ = write!(
    //     text,
    //     "\n💬 Запасы {0} можно пополнить, введя команду <code>купить {{число}}</code>",
    //     holiday_choice(&"леденцов", &"мандаринок", &"тыковок")
    // );

    bot.send(JuzoAnswer::message(&message).text(text))
        .await?;

    Ok(())
}
