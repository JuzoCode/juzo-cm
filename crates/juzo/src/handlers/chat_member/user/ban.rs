use core::fmt::Write;

use juzo_core::{common::emojis::smail_pensil, domain::UserModel};
use sea_orm::{ConnectionTrait, raw_sql};
use telers::{
    enums::ParseMode,
    event::EventReturn,
    methods::{BanChatMember, SendMessage},
    types::LinkPreviewOptions,
};

use super::super::*;

pub async fn yes(
    bot: Bot,
    member: ChatMemberUpdated,
    Extension(db): Extension<DbConn>,
) -> HandlerResult {
    let chat_ids = member.chat.id();
    let user: UserModel = member
        .new_chat_member
        .user()
        .into();
    let iam: UserModel = member
        .from
        .as_ref()
        .into();

    let Ok(Some(row)) = db
        .query_one_raw(raw_sql!(
            Postgres,
            r#"
            SELECT
                COALESCE(c4.rank, 0)::smallint AS my_rank,
                c8.rank,
                c8.reason
            FROM c8
            LEFT JOIN c4
                ON c4.user_ids = {iam.ids}
                AND c4.chat_ids = {chat_ids}
            WHERE c8.user_ids = {user.ids}
              AND c8.chat_ids = {chat_ids}
              AND c8.is_ban
              AND (
                  c8.removed = 0
                  OR c8.removed > EXTRACT(EPOCH FROM NOW())
              )
            "#
        ))
        .await
    else {
        return Ok(EventReturn::Skip);
    };

    let (my_rank, rank, reason) = unsafe {
        (
            row.try_get::<i16>("", "my_rank")
                .unwrap_unchecked(),
            row.try_get::<i16>("", "rank")
                .unwrap_unchecked(),
            row.try_get::<String>("", "reason")
                .unwrap_unchecked(),
        )
    };

    let can_unban = my_rank >= rank;

    if iam.ids == user.ids && can_unban {
        let _ = db
            .execute_raw(raw_sql!(
                Postgres,
                r#"
                DELETE FROM c8
                WHERE user_ids = {user.ids}
                    AND chat_ids = {chat_ids}
                    AND is_ban
                "#
            ))
            .await;
        return Ok(EventReturn::Finish);
    }

    let mut text: String;
    let status = if my_rank > 0 {
        format!(
            "Модератор <a href='{0}'>{1}</a> ({my_rank})",
            user.link(),
            iam.full_name()
        )
    } else {
        format!("<a href='{0}'>{1}</a>", user.link(), iam.full_name())
    };

    if iam.ids == user.ids {
        text = format!("🗓 <b>{status} был исключён</b>: имеется действующий <b>бан",);
    } else if !can_unban {
        text = format!(
            "{0} <b>{status} добавил <a href='{1}'>{2}</a></b>, у которого имеется <b>бан",
            smail_pensil(true),
            user.link(),
            user.full_name()
        );
    } else {
        text = format!(
            "🗓 <b>{status} добавил <a href='{0}'>{1}</a></b>, у которого был <b>бан",
            user.link(),
            user.full_name()
        );
    }

    if !reason.is_empty() {
        let _ = write!(text, " по причине:</b> <i>{0}</i>", reason);
    } else {
        text.push_str("</b>");
    }

    if iam.ids != user.ids {
        if can_unban {
            let _ = write!(
                text,
                " ({rank}), и <b><a href='{0}'>он</a> был исключён из бан-листа</b>",
                user.link()
            );
        } else if my_rank > 0 {
            let _ = write!(
                text,
                " ({rank}), но из-за его должности не смог выдать разбан, и <a href='{0}'>{1}</a> \
                 был исключён.",
                user.link(),
                user.full_name()
            );
        } else {
            let _ = write!(
                text,
                " ({rank}). Поэтому <a href='{0}'>{1}</a> был исключён.",
                user.link(),
                user.full_name()
            );
        }
    }

    bot.send(
        SendMessage::new(chat_ids, text)
            .parse_mode(ParseMode::HTML)
            .link_preview_options(LinkPreviewOptions {
                is_disabled: Some(true),
                ..Default::default()
            }),
    )
    .await?;

    if can_unban {
        let _ = db
            .execute_raw(raw_sql!(
                Postgres,
                r#"
                DELETE FROM c8
                WHERE user_ids = {user.ids}
                    AND chat_ids = {chat_ids}
                    AND is_ban
                "#
            ))
            .await;

        Ok(EventReturn::Finish)
    } else {
        bot.send(BanChatMember::new(chat_ids, user.ids))
            .await?;

        Ok(EventReturn::Cancel)
    }
}
