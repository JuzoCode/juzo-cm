use core::intrinsics::unreachable;

use juzo_core::{
    application::JuzoBonus,
    common::{
        emojis::{smail_pensil, smail_sweets, smail_tick},
        inflection::plur_sweets,
    },
    db::user::prelude::UserBalance,
    domain::{AttachResult, enums::BonusLog},
};
use sea_orm::{ConnectionTrait, EntityTrait, QuerySelect, raw_sql, sea_query::Expr};
use telers::{
    methods::GetChat,
    types::{Chat, ChatFullInfo, ChatFullInfoSupergroup},
};

use super::super::*;

pub async fn bag(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(arch): Extension<AttachResult>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    let chat_ids = arch.chat_ids.0;

    let module = ModuleChecker::new(&bot, &db);
    let true = module
        .check::<32>(ModuleAccess::CustomM(&message, chat_ids))
        .await
    else {
        return Ok(());
    };

    // ради кубышки делать отдельную таблицу в БД? Мне что, делать нечего?
    let sweets = UserBalance::find_by_id(chat_ids)
        .select_only()
        .column_as(Expr::cust("TRUNC(sweets)::int"), "sweets")
        .into_tuple::<u32>()
        .one(&db)
        .await
        .ok()
        .flatten()
        .unwrap_or_default();

    bot.send(JuzoAnswer::message(&message).text(format!(
        "{0} В кубышке чата сейчас лежит {1}",
        smail_sweets(true),
        plur_sweets(sweets)
    )))
    .await?;

    Ok(())
}

pub async fn show_ids(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    let module = ModuleChecker::new(&bot, &db);
    let true = module
        .check::<37>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    bot.send(
        JuzoAnswer::message(&message)
            .text(format!("🗓 ID чата: <code>{0}</code>", message.chat().id())),
    )
    .await?;

    Ok(())
}

pub async fn update(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    let Chat::Supergroup(chat) = message.chat() else {
        // SAFETY: The ChatType filter allows processing only of "Supergroup".
        unsafe { unreachable() }
    };

    let module = ModuleChecker::new(&bot, &db);
    let bonus = JuzoBonus::new(&db).chat_ids(chat.id);

    let true = module
        .check::<22>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    bonus.delete(BonusLog::Premium);

    // SAFETY: The "None" value only occurs for users.
    let full_name = unsafe {
        chat.title
            .as_deref()
            .unwrap_unchecked()
    };

    let community_ids = match bot
        .send(GetChat::new(chat.id))
        .await?
    {
        ChatFullInfo::Supergroup(ChatFullInfoSupergroup {
            community: Some(c),
            ..
        }) => c.id,
        ChatFullInfo::Supergroup(_) => 0,
        _ => unsafe { unreachable() },
    };

    let Ok(_) = db
        .execute_raw(raw_sql!(
            Postgres,
            r#"
            INSERT INTO c (
                chat_ids,
                full_name,
                community
            )
            VALUES ({chat.id}, {full_name}, {community_ids})
            ON CONFLICT (chat_ids) DO UPDATE SET
                full_name = EXCLUDED.full_name
            WHERE
                c.full_name IS DISTINCT FROM EXCLUDED.full_name
            "#
        ))
        .await
    else {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Чат не был обновлён, но некоторые данные будут перекэшированы.",
            smail_pensil(true)
        )))
        .await?;
        return Ok(());
    };

    bot.send(JuzoAnswer::message(&message).text(format!("{0} Чат обновлён", smail_tick(true))))
        .await?;

    Ok(())
}
