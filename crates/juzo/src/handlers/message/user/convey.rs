use juzo_core::{
    application::{UserIndex, UserModel},
    common::{
        emojis::{smail_gold, smail_pensil, smail_score, smail_sweets, smail_warning},
        inflection::{plur_gold, plur_score, plur_sweets},
        tools::time::holiday_choice,
    },
    db::user::{balance, prelude::UserBalance},
};
use sea_orm::{ConnectionTrait, DbConn, EntityTrait, QuerySelect, raw_sql, sea_query::Expr};
use telers::types::ReplyParameters;

use super::super::*;

pub async fn sweets(
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
    let args = result.args::<2>(text);
    let comment = &text[result.first_line];

    let (user, value): (UserModel, u32) = match args {
        // SAFETY: TBA will never return None in message.from().
        Some([a1, a2]) if a2.is_empty() => unsafe {
            let found_user = if let Some(r) = message.reply_to_message() {
                r.from()
                    .unwrap_unchecked()
                    .into()
            } else {
                return Ok(());
            };

            let Ok(value) = text[a1].parse() else {
                return Ok(());
            };

            (found_user, value)
        },
        Some([a1, a2]) => {
            let Ok(found_user) = user_ind
                .search_user(&text[a2])
                .await
            else {
                return Ok(());
            };

            let Ok(value) = text[a1].parse() else {
                return Ok(());
            };

            (found_user, value)
        }
        None => return Ok(()),
    };

    if !user.is_user {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} {1} передаются только избранным.",
            smail_pensil(true),
            holiday_choice(&"Леденцы", &"Мандаринки", &"Тыковки")
        )))
        .await?;
        return Ok(());
    } else if value == 0 {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Даже ваш мешок знает, что ноль — это не перевод.",
            smail_pensil(true)
        )))
        .await?;
        return Ok(());
    } else if comment
        .chars()
        .nth(128)
        .is_some()
    {
        bot.send(
            JuzoAnswer::message(&message)
                .text(format!("{0} Длина текста превышает 128 символов.", smail_pensil(true))),
        )
        .await?;
        return Ok(());
    }

    // SAFETY: TBA will never return None in message.from().
    let iam: UserModel = unsafe {
        message
            .from()
            .unwrap_unchecked()
            .into()
    };

    let bag = UserBalance::find_by_id(iam.ids)
        .select_only()
        .column_as(Expr::cust("TRUNC(sweets)::int"), "sweets")
        .into_tuple::<u32>()
        .one(&db)
        .await
        .ok()
        .flatten()
        .unwrap_or_default();

    if value > bag {
        bot.send(
            JuzoAnswer::message(&message)
                .text(format!("{0} Ваш мешок не согласен с таким переводом.", smail_pensil(true))),
        )
        .await?;
        return Ok(());
    }

    let Ok(_) = db
        .execute_raw(raw_sql!(
            Postgres,
            r#"
            WITH moved AS (
                UPDATE u2
                SET sweets = sweets - {value}
                WHERE user_ids = {iam.ids}
                RETURNING user_ids
            )
            INSERT INTO u2(user_ids, sweets)
            SELECT {user.ids}, {value}
            FROM moved
            ON CONFLICT (user_ids)
            DO UPDATE SET sweets = u2.sweets + EXCLUDED.sweets
            "#
        ))
        .await
    else {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} <b>Перевод получился неудачным.</b> Не бойтесь, ваши {1} в безопасности =)",
            smail_warning(true),
            holiday_choice(&"леденцы", &"мандаринки", &"тыковки")
        )))
        .await?;
        return Ok(());
    };

    let smail_sweets = smail_sweets(true);
    let sweets_name = plur_sweets(value);

    let mut text = format!(
        "{smail_sweets} <a href='{0}'>{1}</a> получил {sweets_name}",
        user.link(),
        user.full_name(),
    );

    if !comment.is_empty() {
        text.push_str(".<blockquote expandable><b>💬 Подпись к переводу:</b> ");
        text.push_str(comment);
        text.push_str("</blockquote>");
    }

    let mut text_send = format!(
        "{sweets_name} Вам перевели {sweets_name}.<blockquote expandable><b>👤 Отправитель:</b> \
         <a href='{0}'>{1}</a>",
        iam.link(),
        iam.full_name(),
    );

    if !comment.is_empty() {
        text_send.push_str("\n<b>💬 Подпись к переводу:</b> ");
        text_send.push_str(comment);
    }

    text_send.push_str("</blockquote>");

    bot.send(JuzoAnswer::message(&message).text(text))
        .await?;
    let _ = bot
        .send(
            JuzoAnswer::message(&message)
                .text(text_send)
                .chat_id(user.ids.0)
                .reply_parameters_option::<ReplyParameters>(None),
        )
        .await;

    Ok(())
}

pub async fn gold(
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
    let args = result.args::<2>(text);
    let comment = &text[result.first_line];

    let (user, value): (UserModel, u32) = match args {
        // SAFETY: TBA will never return None in message.from().
        Some([a1, a2]) if a2.is_empty() => unsafe {
            let found_user = if let Some(r) = message.reply_to_message() {
                r.from()
                    .unwrap_unchecked()
                    .into()
            } else {
                return Ok(());
            };

            let Ok(value) = text[a1].parse() else {
                return Ok(());
            };

            (found_user, value)
        },
        Some([a1, a2]) => {
            let Ok(found_user) = user_ind
                .search_user(&text[a2])
                .await
            else {
                return Ok(());
            };

            let Ok(value) = text[a1].parse() else {
                return Ok(());
            };

            (found_user, value)
        }
        None => return Ok(()),
    };

    if !user.is_user {
        bot.send(
            JuzoAnswer::message(&message).text(format!(
                "{0} Золотые леденцы передаются только избранным.",
                smail_pensil(true),
            )),
        )
        .await?;
        return Ok(());
    } else if value == 0 {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Даже ваш мешок знает, что ноль — это не перевод.",
            smail_pensil(true),
        )))
        .await?;
        return Ok(());
    } else if comment
        .chars()
        .nth(128)
        .is_some()
    {
        bot.send(
            JuzoAnswer::message(&message)
                .text(format!("{0} Длина текста превышает 128 символов.", smail_pensil(true))),
        )
        .await?;
        return Ok(());
    }

    // SAFETY: TBA will never return None in message.from().
    let iam: UserModel = unsafe {
        message
            .from()
            .unwrap_unchecked()
            .into()
    };

    let bag = UserBalance::find_by_id(iam.ids)
        .select_only()
        .column(balance::Column::Gold)
        .into_tuple::<u32>()
        .one(&db)
        .await
        .ok()
        .flatten()
        .unwrap_or_default();

    if value > bag {
        bot.send(
            JuzoAnswer::message(&message)
                .text(format!("{0} Ваш мешок не согласен с таким переводом.", smail_pensil(true))),
        )
        .await?;
        return Ok(());
    }

    let Ok(_) = db
        .execute_raw(raw_sql!(
            Postgres,
            r#"
            WITH moved AS (
                UPDATE u2
                SET gold = gold::bigint - {value}
                WHERE user_ids = {iam.ids}
                RETURNING user_ids
            )
            INSERT INTO u2(user_ids, gold)
            SELECT {user.ids}, {value}
            FROM moved
            ON CONFLICT (user_ids)
            DO UPDATE SET gold = u2.gold::bigint + EXCLUDED.gold::bigint
            "#
        ))
        .await
    else {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} <b>Перевод получился неудачным.</b> Не бойтесь, ваши золотые леденцы в \
             безопасности =)",
            smail_warning(true)
        )))
        .await?;
        return Ok(());
    };

    let mut text = format!(
        "{0} <a href='{1}'>{2}</a> получил {3}",
        smail_gold(true),
        user.link(),
        user.full_name(),
        plur_gold(value)
    );

    if !comment.is_empty() {
        text.push_str(".<blockquote expandable><b>💬 Подпись к переводу:</b> ");
        text.push_str(comment);
        text.push_str("</blockquote>");
    }

    let mut text_send = format!(
        "{0} Вам перевели {1}.<blockquote expandable><b>👤 Отправитель:</b> <a href='{2}'>{3}</a>",
        smail_gold(true),
        plur_gold(value),
        iam.link(),
        iam.full_name(),
    );

    if !comment.is_empty() {
        text_send.push_str("\n<b>💬 Подпись к переводу:</b> ");
        text_send.push_str(comment);
    }

    text_send.push_str("</blockquote>");

    bot.send(JuzoAnswer::message(&message).text(text))
        .await?;
    let _ = bot
        .send(
            JuzoAnswer::message(&message)
                .text(text_send)
                .chat_id(user.ids.0)
                .reply_parameters_option::<ReplyParameters>(None),
        )
        .await;

    Ok(())
}

pub async fn score(
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
    let args = result.args::<2>(text);
    let comment = &text[result.first_line];

    let (user, value): (UserModel, u32) = match args {
        // SAFETY: TBA will never return None in message.from().
        Some([a1, a2]) if a2.is_empty() => unsafe {
            let found_user = if let Some(r) = message.reply_to_message() {
                r.from()
                    .unwrap_unchecked()
                    .into()
            } else {
                return Ok(());
            };

            let Ok(value) = text[a1].parse() else {
                return Ok(());
            };

            (found_user, value)
        },
        Some([a1, a2]) => {
            let Ok(found_user) = user_ind
                .search_user(&text[a2])
                .await
            else {
                return Ok(());
            };

            let Ok(value) = text[a1].parse() else {
                return Ok(());
            };

            (found_user, value)
        }
        None => return Ok(()),
    };

    if !user.is_user {
        bot.send(
            JuzoAnswer::message(&message)
                .text(format!("{0} Очки доната передаются только избранным.", smail_pensil(true),)),
        )
        .await?;
        return Ok(());
    } else if value == 0 {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Даже ваш мешок знает, что ноль — это не перевод.",
            smail_pensil(true),
        )))
        .await?;
        return Ok(());
    } else if comment
        .chars()
        .nth(128)
        .is_some()
    {
        bot.send(
            JuzoAnswer::message(&message)
                .text(format!("{0} Длина текста превышает 128 символов.", smail_pensil(true))),
        )
        .await?;
        return Ok(());
    }

    // SAFETY: TBA will never return None in message.from().
    let iam: UserModel = unsafe {
        message
            .from()
            .unwrap_unchecked()
            .into()
    };

    let bag = UserBalance::find_by_id(iam.ids)
        .select_only()
        .column(balance::Column::Gold)
        .into_tuple::<u32>()
        .one(&db)
        .await
        .ok()
        .flatten()
        .unwrap_or_default();

    if value > bag {
        bot.send(
            JuzoAnswer::message(&message)
                .text(format!("{0} Ваш мешок не согласен с таким переводом.", smail_pensil(true))),
        )
        .await?;
        return Ok(());
    }

    let Ok(_) = db
        .execute_raw(raw_sql!(
            Postgres,
            r#"
            WITH moved AS (
                UPDATE u2
                SET score = score::bigint - {value}
                WHERE user_ids = {iam.ids}
                RETURNING user_ids
            )
            INSERT INTO u2(user_ids, score)
            SELECT {user.ids}, {value}
            FROM moved
            ON CONFLICT (user_ids)
            DO UPDATE SET score = u2.score::bigint + EXCLUDED.score::bigint
            "#
        ))
        .await
    else {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} <b>Перевод получился неудачным.</b> Не бойтесь, ваши очки доната в безопасности \
             =)",
            smail_warning(true)
        )))
        .await?;
        return Ok(());
    };

    let mut text = format!(
        "{0} <a href='{1}'>{2}</a> получил {3}",
        smail_score(true),
        user.link(),
        user.full_name(),
        plur_score(value)
    );

    if !comment.is_empty() {
        text.push_str(".<blockquote expandable><b>💬 Подпись к переводу:</b> ");
        text.push_str(comment);
        text.push_str("</blockquote>");
    }

    let mut text_send = format!(
        "{0} Вам перевели {1}.<blockquote expandable><b>👤 Отправитель:</b> <a href='{2}'>{3}</a>",
        smail_score(true),
        plur_score(value),
        iam.link(),
        iam.full_name(),
    );

    if !comment.is_empty() {
        text_send.push_str("\n<b>💬 Подпись к переводу:</b> ");
        text_send.push_str(comment);
    }

    text_send.push_str("</blockquote>");

    bot.send(JuzoAnswer::message(&message).text(text))
        .await?;
    let _ = bot
        .send(
            JuzoAnswer::message(&message)
                .text(text_send)
                .chat_id(user.ids.0)
                .reply_parameters_option::<ReplyParameters>(None),
        )
        .await;

    Ok(())
}
