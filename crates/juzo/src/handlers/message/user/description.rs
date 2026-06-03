use juzo_core::{
    application::{UserIndex, UserModel},
    common::emojis::smail_tick,
};
use sea_orm::DbConn;

use super::super::*;

/// TODO
pub async fn show(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) {
    let user_ind = UserIndex::new(&bot, &db);

    // SAFETY: The Command filter will not allow processing of a "None" value.
    let text = unsafe {
        message
            .text()
            .or_else(|| message.caption())
            .unwrap_unchecked()
    };
    let args = result.args::<1>(text);

    (format!("{0} {{}} пока не заполнил описание о себе.", smail_tick(true)));
}

/// TODO
pub async fn edit(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) {
    let user_ind = UserIndex::new(&bot, &db);

    // SAFETY: The Command filter will not allow processing of a "None" value.
    let text = unsafe {
        message
            .text()
            .or_else(|| message.caption())
            .unwrap_unchecked()
    };
    let args = result.args::<1>(text);


    (format!("{0} Описание пользователя обновлено", smail_tick(true)));
}
