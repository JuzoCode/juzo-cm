use core::fmt::Write;

use juzo_core::domain::UserModel;
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
                a3.reason
            FROM a3
            INNER JOIN c
                ON c.chat_ids = {chat_ids}
                AND NOT c.spam
            LEFT JOIN c4
                ON c4.user_ids = {iam.ids}
                AND c4.chat_ids = {chat_ids}
            WHERE a3.user_ids = {user.ids}
                AND a3.function = 2
            "#
        ))
        .await
    else {
        return Ok(EventReturn::Finish);
    };

    let (is_moder, reason) = unsafe {
        (
            row.try_get::<i16>("", "my_rank")
                .unwrap_unchecked()
                > 0,
            row.try_get::<String>("", "reason")
                .unwrap_unchecked(),
        )
    };

    let mut text: String;

    let status = if is_moder { "Модератор " } else { "" };

    if iam.ids != user.ids {
        text = format!(
            "🗓 <b>{status}<a href='{0}'>{1}</a> добавил <a href='{2}'>{3}</a></b>. Пользователь \
             находится в базе «Джузо-антиспам»",
            iam.link(),
            iam.full_name(),
            user.link(),
            user.full_name()
        );

        if !reason.is_empty() {
            let _ = write!(text, " <b>по причине: <i>{reason}</i></b>");
        }

        if !is_moder {
            text.push_str(", поэтому был исключён");
        }
    } else {
        text = format!(
            "📛 В чат зашёл <a href='{0}'>спамер</a>!<a href='tg://user?id={1}'>\u{2069}</a> \
             Исключаю",
            iam.link(),
            user.ids
        );

        if !reason.is_empty() {
            let _ = write!(text, "\n<blockquote expandable>{reason}</blockquote>");
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

    if is_moder {
        return Ok(EventReturn::Finish);
    } else {
        bot.send(BanChatMember::new(chat_ids, user.ids))
            .await?;

        Ok(EventReturn::Cancel)
    }
}
