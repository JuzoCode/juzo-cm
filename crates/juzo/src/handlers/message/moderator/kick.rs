use juzo_core::{
    application::{UserIndex, UserModel},
    common::emojis::smail_cross,
    domain::UserModelExt,
    gender,
};
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

    let true = module
        .check::<46>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    let chat_ids = message.chat().id();

    // сделать скл чекер на ранг

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
