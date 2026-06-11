use juzo_core::{
    application::{FullName, UserIndex, UserModel},
    common::{
        emojis::{smail_pensil, smail_tick},
        tools::filter::filter_admins,
    },
    payloads::{
        base::PackedPayload,
        callback::{Callback, CallbackKind},
    },
};
use sea_orm::DbConn;
use telers::{
    methods::{GetChatAdministrators, PromoteChatMember},
    types::{ChatMember, InlineKeyboardButton, InlineKeyboardMarkup, ReplyParameters},
};

use super::super::*;

// ДОДЕЛАТЬ
pub async fn call(
    bot: Bot,
    message: Message,
    Extension(_db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    // SAFETY: TBA will never return None in message.from().
    let user = unsafe {
        message
            .from()
            .unwrap_unchecked()
    };

    let admins = bot
        .send(GetChatAdministrators::new(message.chat().id()))
        .await?;

    let humans = filter_admins(admins, user.id);

    if humans.is_empty() {
        bot.send(
            JuzoAnswer::message(&message)
                .text(format!("{0} Тут это бессмысленно.", smail_pensil(true))),
        )
        .await?;
        return Ok(());
    }

    let header = FullName::new(&user.first_name, user.last_name.as_deref())
        .with_name(|full_name| format!("🗣 {full_name} созывает тг-администраторов"));

    let mut last_msg: Option<Message> = None;
    let chunks: Vec<_> = humans
        .chunks(5)
        .collect();
    let total_chunks = chunks.len();

    let mut message_ids = Vec::with_capacity(total_chunks);

    for (i, chunk) in chunks.iter().enumerate() {
        let mut mentions = String::with_capacity(chunk.len() * 40);

        for admin in *chunk {
            let user_id = match admin {
                ChatMember::Administrator(m) => m.user.id,
                ChatMember::Creator(m) => m.user.id,
                _ => continue,
            };

            use core::fmt::Write;
            let _ = write!(mentions, "<a href='tg://user?id={user_id}'>\u{2069}</a>");
        }

        let mut req = JuzoAnswer::message(&message).text(format!("{header}{mentions}"));

        if let Some(prev) = &last_msg {
            req = req.reply_parameters(ReplyParameters::new(prev.message_id()));
        }

        if i + 1 == total_chunks {
            let callback = PackedPayload::new(0, Callback::new(CallbackKind::CallTgAdmin)).encode();

            req = req.reply_markup(InlineKeyboardMarkup::new([[InlineKeyboardButton::new(
                "➖ Удалить упоминание",
            )
            .callback_data(callback.as_str())]]));
        }

        let sent = bot.send(req).await?;
        message_ids.push(sent.message_id());
        last_msg = Some(sent);
    }

    Ok(())
}

// ДОДЕЛАТЬ
pub async fn add(
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
    let args = &text[result.args];

    let (_tag, user): (&str, UserModel) = match (args.is_empty(), message.reply_to_message()) {
        (_, Some(reply)) => {
            if args
                .chars()
                .nth(16)
                .is_some()
            {
                return Ok(());
            }

            // SAFETY: Branching does not allow processing the None.
            unsafe {
                (
                    args,
                    reply
                        .from()
                        .unwrap_unchecked()
                        .into(),
                )
            }
        }
        (false, _) => {
            let (tag, user_line) = args
                .rsplit_once(char::is_whitespace)
                .unwrap_or(("", args));
            if tag
                .chars()
                .nth(16)
                .is_some()
            {
                return Ok(());
            }

            let Ok(found_user) = user_ind
                .search_user(user_line)
                .await
            else {
                return Ok(());
            };

            (tag, found_user)
        }
        _ => return Ok(()),
    };

    // match juzo.add_tg_admin(user_id).await {
    //     Ok(_) => {
    //         let _ = juzo.bot.set_chat_administrator_custom_title(
    //             juzo.message.chat.id, user_id, custom_title,
    //         ).await?;
    //     },
    //     Err(_) => return
    // }

    bot.send(JuzoAnswer::message(&message).text(format!(
        "{0} {1} успешно назначен тг-администратором<a href='tg://user?id={1}'>\u{2069}</a>",
        smail_tick(true),
        user.ids
    )))
    .await?;

    Ok(())
}

pub async fn delete(
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
            } else {
                return Ok(());
            }
        },
    };

    bot.send(PromoteChatMember::new(message.chat().id(), user.ids).can_manage_chat(false))
        .await?;

    bot.send(JuzoAnswer::message(&message).text(format!(
        "{0} {1} исключён из тг-администраторов",
        smail_tick(true),
        user.ids
    )))
    .await?;

    Ok(())
}
