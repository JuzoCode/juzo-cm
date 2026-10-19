use core::fmt::Write;

use chrono::Utc;
use juzo_core::{
    application::{UserIndex, UserModel},
    common::emojis::{smail_cross, smail_pensil, smail_tick},
    db::user::{anketa, prelude::UserAnketa},
    domain::TimeFormatted,
    gender,
};
use sea_orm::{ConnectionTrait, EntityTrait, QuerySelect, Set, raw_sql, sea_query::OnConflict};

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

    // SAFETY: TBA will never return None in message.from().
    let my_ids = unsafe {
        message
            .from()
            .unwrap_unchecked()
    }
    .id;

    let Ok(Some(row)) = db
        .query_one_raw(raw_sql!(
            Postgres,
            r#"
            SELECT
                (u.user_ids = {my_ids} OR u.show) AS show,
                u.gender,
                u.added,
                COALESCE(u1.brith, '') AS brith
            FROM u
            LEFT JOIN u1
                ON u1.user_ids = u.user_ids
            WHERE u.user_ids = {user.ids}
            "#
        ))
        .await
    else {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} О данном пользователе ни слуху, ни духу.",
            smail_pensil(true)
        )))
        .await?;
        return Ok(());
    };

    let (show, gender, added, _) = unsafe {
        (
            row.try_get::<bool>("", "show")
                .unwrap_unchecked(),
            row.try_get::<i16>("", "gender")
                .unwrap_unchecked(),
            row.try_get::<i64>("", "added")
                .unwrap_unchecked(),
            row.try_get::<String>("", "brith")
                .unwrap_unchecked(),
        )
    };

    if !show {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Анкета же скрыта от незрелых глазок.",
            smail_pensil(true)
        )))
        .await?;
        return Ok(());
    }

    let mut text = if user.ids.0 < 0 {
        format!(
            "<b>Это <a href='{0}'>{1}</a></b>\n🆔 <code>@_{2}</code>\n\n",
            user.link(),
            user.full_name(),
            user.ids
        )
    } else {
        format!(
            "<b>Это <a href='{0}'>{1}</a></b>\n🆔 <code>@{2}</code>\n\n",
            user.link(),
            user.full_name(),
            user.ids
        )
    };

    if added <= 0 {
        text.push_str("🗓 <b>Во вселенной Джузо</b> не существует");
    } else {
        let now = Utc::now();
        let time = TimeFormatted::from(added, now);

        let _ = write!(
            text,
            "🗓 <b>Во вселенной Джузо:</b> с {0} ({time})",
            time.datetime
                .format("%d.%m.%Y")
        );
    };

    if gender == 3 {
        text.push_str("\n👨 <b>Пол:</b> не указан")
    } else {
        let (g1, g2) = gender!(gender => [("👩", "женский"), ("👨", "мужской"), ("👨", "другой")]);

        let _ = write!(text, "\n{g1} <b>Пол:</b> {g2}");
    }

    bot.send(JuzoAnswer::message(&message).text(text))
        .await?;

    Ok(())
}

pub async fn first_appearance(
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
    if message
        .chat()
        .title()
        .is_some()
    {
        let true = module
            .check::<29>(ModuleAccess::M(&message))
            .await
        else {
            return Ok(());
        };
    }

    // Слышали про английский? -- Нет
    let added = UserAnketa::find_by_id(user.ids)
        .select_only()
        .column(anketa::Column::Added)
        .into_tuple::<i64>()
        .one(&db)
        .await
        .ok()
        .flatten()
        .unwrap_or_default();

    let text = if added <= 0 {
        format!(
            "🗓 Даты первого появления <a href='{0}'>{1}</a> во вселенной Джузо не существует",
            user.link(),
            user.full_name()
        )
    } else {
        let now = Utc::now();
        let time = TimeFormatted::from(added, now);

        format!(
            "🗓 Дата первого появления <a href='{0}'>{1}</a> во вселенной Джузо: {2} ({time})",
            user.link(),
            user.full_name(),
            time.datetime
                .format("%d.%m.%Y")
        )
    };

    bot.send(JuzoAnswer::message(&message).text(text))
        .await?;

    Ok(())
}

async fn edit_show_core(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
    show: bool,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    if message
        .chat()
        .title()
        .is_some()
    {
        let module = ModuleChecker::new(&bot, &db);
        let true = module
            .check::<28>(ModuleAccess::M(&message))
            .await
        else {
            return Ok(());
        };
    }

    let iam: UserModel = unsafe {
        message
            .from()
            .unwrap_unchecked()
    }
    .into();

    let model = anketa::ActiveModel {
        user_ids: Set(iam.ids),
        show: Set(show),
        is_user: Set(iam.is_user),
        ..Default::default()
    };

    let _ = UserAnketa::insert(model)
        .on_conflict(
            OnConflict::column(anketa::Column::UserIds)
                .update_column(anketa::Column::Show)
                .to_owned(),
        )
        .exec(&db)
        .await;

    if show {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Теперь <a href='{1}'>ваша</a> анкета видна",
            smail_tick(true),
            iam.link()
        )))
        .await?;
    } else {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Теперь <a href='{1}'>ваша</a> анкета скрыта",
            smail_cross(true),
            iam.link()
        )))
        .await?;
    }

    Ok(())
}

pub async fn edit_show_true(
    bot: Bot,
    message: Message,
    db: Extension<DbConn>,
    result: Extension<CommandResult>,
) -> HandlerResult<()> {
    edit_show_core(bot, message, db, result, true).await
}

pub async fn edit_show_false(
    bot: Bot,
    message: Message,
    db: Extension<DbConn>,
    result: Extension<CommandResult>,
) -> HandlerResult<()> {
    edit_show_core(bot, message, db, result, false).await
}
