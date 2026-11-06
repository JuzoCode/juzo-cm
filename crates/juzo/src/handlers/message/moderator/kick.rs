use juzo_core::{
    application::{UserIndex, UserModel},
    common::emojis::{smail_cross, smail_pensil},
    domain::UserModelExt,
    gender,
};
use sea_orm::{ConnectionTrait, raw_sql};
use telers::methods::{BanChatMember, BanChatSenderChat, SendMessage};

use super::super::*;

pub async fn yes(
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
                UserModel::new(
                    &db,
                    r.from()
                        .unwrap_unchecked(),
                )
                .await
            } else {
                return Ok(());
            }
        },
        ArgsResult::Unk => return Ok(()),
    };

    // SAFETY: TBA will never return None in message.from().
    let my_ids = unsafe {
        message
            .from()
            .unwrap_unchecked()
    }
    .id;

    if user.ids.0 == my_ids {
        return Ok(());
    }

    let true = module
        .check::<46>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    let chat_ids = message.chat().id();

    let Ok(Some(_)) = db
        .query_one_raw(raw_sql!(
            Postgres,
            r#"
            SELECT 1
            FROM c4
            WHERE chat_ids = {chat_ids}
                AND user_ids = {my_ids}
                AND COALESCE(rank, 0) > COALESCE((
                    SELECT rank
                    FROM c4
                    WHERE chat_ids = {chat_ids}
                        AND user_ids = {user.ids}
                ), 0)
            "#
        ))
        .await
    else {
        bot.send(JuzoAnswer::<SendMessage>::to(&message).text(format!(
            "{0} Ваш ранг либо недостаточен, либо его вовсе не хватает.",
            smail_pensil(true)
        )))
        .await?;
        return Ok(());
    };

    if user.ids.0 < 0 {
        bot.send(BanChatSenderChat::new(chat_ids, user.ids))
            .await?;
    } else {
        bot.send(BanChatMember::new(chat_ids, user.ids))
            .await?;
    };

    let g1 = gender!(user.gender => ["ена", "ён"]);

    bot.send(JuzoAnswer::<SendMessage>::to(&message).text(format!(
        "{0} <a href='{1}'>{2}</a> исключ{g1} из чата",
        smail_cross(true),
        user.link(),
        user.full_name(),
    )))
    .await?;

    Ok(())
}
