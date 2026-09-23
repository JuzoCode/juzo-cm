use core::{fmt::Write, intrinsics::unreachable};

use chrono::{Timelike, Utc};
use juzo_core::{
    application::{BonusResult, JuzoBonus, UserIndex, UserModel},
    common::{
        emojis::{
            smail_asterisks, smail_bag, smail_cross, smail_gold, smail_pensil, smail_score,
            smail_sweets, smail_tick, smail_vip,
        },
        inflection::{plur_asterisks, plur_gold, plur_score, plur_sweets},
    },
    db::user::{balance, prelude::UserBalance},
    domain::{TimeFormatted, UserModelExt, enums::BonusLog},
    gender,
};
use sea_orm::{
    EntityTrait, Set, raw_sql,
    sea_query::{OnConflict, prelude::rust_decimal::prelude::ToPrimitive},
};

use super::super::*;

/// добавить в бизнес мод
async fn show_core(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
    my: bool,
    ls: bool,
) -> HandlerResult<()> {
    let user_ind = UserIndex::new(&bot, &db);
    let module = ModuleChecker::new(&bot, &db);
    let bonus = JuzoBonus::new(&db);

    let user: UserModel;
    let is_chat = message
        .chat()
        .title()
        .is_some();

    if my {
        user = UserModel::new(
            &db,
            // SAFETY: TBA will never return None in message.from().
            unsafe {
                message
                    .from()
                    .unwrap_unchecked()
            },
        )
        .await
    } else {
        // SAFETY: The Command filter will not allow processing of a "None" value.
        let text = unsafe {
            message
                .text()
                .or_else(|| message.caption())
                .unwrap_unchecked()
        };
        let args = result.args::<1>(text);

        user = match args {
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
                    UserModel::new(
                        &db,
                        r.from()
                            .unwrap_unchecked(),
                    )
                    .await
                } else {
                    UserModel::new(
                        &db,
                        message
                            .from()
                            .unwrap_unchecked(),
                    )
                    .await
                }
            },
            ArgsResult::Unk => return Ok(()),
        };
    }

    if is_chat {
        let true = module
            .check::<31>(ModuleAccess::M(&message))
            .await
        else {
            return Ok(());
        };
    }

    if !user.is_user {
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

    // SAFETY: TBA will never return None in message.from().
    let my_ids = unsafe {
        message
            .from()
            .unwrap_unchecked()
    }
    .id;

    let chat_ids = if ls && is_chat {
        let _ = bot
            .send(JuzoAnswer::message(&message).text(format!(
                "{0} Мешок отправлен в <a href='https://t.me/juzo_cm_bot'>личные сообщения</a> \
                 Джузо",
                smail_bag(true)
            )))
            .await;
        my_ids
    } else {
        message.chat().id()
    };

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
        let (g1, g2) = gender!(user.gender => [("a", "ё"), ("", "го")]);

        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} <b><a href='{1}'>{2}</a></b> скрыл{g1} мешок.<blockquote>Заслужите е{g2} \
             милость, и тогда <b>просите открыть его!</b> =)</blockquote>",
            smail_pensil(true),
            user.link(),
            user.full_name(),
        )))
        .await?;
        return Ok(());
    }

    let mut text: String;

    if balance.is_empty() {
        text = format!(
            "{0} <b>В мешке <a href='{1}'>{2}</a></b> пустеет так, что не осталось даже пыли\n",
            smail_bag(true),
            user.link(),
            user.full_name(),
        );
    } else {
        text = format!(
            "{0} <b>В мешке <a href='{1}'>{2}</a></b>.<blockquote expandable>{3} {4} {5} {6}\n{7} \
             {8}",
            smail_bag(true),
            user.link(),
            user.full_name(),
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
            smail_asterisks(true),
            plur_asterisks(balance.asterisks),
        );

        if balance.score > 0 {
            let _ = write!(
                text,
                "\n{0} {1}",
                smail_score(true),
                plur_score(balance.score)
            );
        }

        text.push_str("</blockquote>");
    }

    if is_chat {
        // SAFETY: ¯\_(ツ)_/¯
        let now = unsafe {
            Utc::now()
                .with_nanosecond(0)
                .unwrap_unchecked()
        };

        let bonuses = bonus
            .chat_ids(
                message
                    .chat()
                    .id()
                    .into(),
            )
            .user_ids(user.ids)
            .all([BonusLog::Vip, BonusLog::Plus, BonusLog::Minus])
            .await;

        for (index, bonus) in bonuses
            .into_iter()
            .enumerate()
        {
            let BonusResult::Active(removed) = bonus else {
                continue;
            };

            let (smail, name) = match index {
                0 => (smail_vip(true), "VIP-статус"),
                1 => ("✨", "Плюсопад"),
                2 => ("➖", "Минусит"),
                _ => unsafe { unreachable() },
            };

            let _ = write!(
                text,
                "\n{smail} <b>{name}</b> на {0}",
                TimeFormatted::until(removed, now)
            );
        }
    }

    bot.send(
        JuzoAnswer::message(&message)
            .text(text)
            .chat_id(chat_ids),
    )
    .await?;

    Ok(())
}

pub async fn show(
    bot: Bot,
    message: Message,
    db: Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    show_core(bot, message, db, Extension(result), false, false).await
}

pub async fn my_show(
    bot: Bot,
    message: Message,
    db: Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    show_core(bot, message, db, Extension(result), true, false).await
}

pub async fn my_ls_show(
    bot: Bot,
    message: Message,
    db: Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    show_core(bot, message, db, Extension(result), true, true).await
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
            .check::<31>(ModuleAccess::M(&message))
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

    if !iam.is_user {
        return Ok(());
    }

    let model = balance::ActiveModel {
        user_ids: Set(iam.ids),
        show: Set(show),
        ..Default::default()
    };

    let _ = UserBalance::insert(model)
        .on_conflict(
            OnConflict::column(balance::Column::UserIds)
                .update_column(balance::Column::Show)
                .to_owned(),
        )
        .exec(&db)
        .await;

    if show {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Теперь <a href='{1}'>ваш</a> мешок открыт",
            smail_tick(true),
            iam.link()
        )))
        .await?;
    } else {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Теперь <a href='{1}'>ваш</a> мешок закрыт",
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
