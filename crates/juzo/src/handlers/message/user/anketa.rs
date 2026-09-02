use chrono::Utc;
use juzo_core::{
    application::{UserIndex, UserModel},
    common::emojis::smail_pensil,
    db::user::{anketa, prelude::UserAnketa},
    domain::TimeFormatted,
};
use sea_orm::{EntityTrait, QuerySelect, raw_sql};

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

    let true = module
        .check::<28>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    // SAFETY: TBA will never return None in message.from().
    let my_ids = unsafe {
        message
            .from()
            .unwrap_unchecked()
    }
    .id;

    let Ok(Some(anketa)) = UserAnketa::find()
        .from_raw_sql(raw_sql!(
            Postgres,
            r#"
            SELECT
                user_ids,
                CASE
                    WHEN user_ids = {my_ids} THEN true
                    ELSE show
                END AS show,
                is_user,
                gender,
                username,
                full_name,
                added
            FROM u
            WHERE user_ids = {user.ids};
            "#
        ))
        .one(&db)
        .await
    else {
        bot.send(
            JuzoAnswer::message(&message)
                .text(format!("{0} О данном пользователе ни слуху, ни духу.", smail_pensil(true))),
        )
        .await?;
        return Ok(());
    };

    if !anketa.show {
        bot.send(
            JuzoAnswer::message(&message)
                .text(format!("{0} Анкета же скрыта от незрелых глазок.", smail_pensil(true))),
        )
        .await?;
        return Ok(());
    }

    bot.send(JuzoAnswer::message(&message).text("Анкета есть"))
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

    let true = module
        .check::<29>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

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
