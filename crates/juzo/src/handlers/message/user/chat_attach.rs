// use juzo_core::{
//     common::emojis::smail_tick,
// };

use super::super::*;

pub async fn yes(
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

    Ok(())
}

// pub async fn no(
//     bot: Bot,
//     message: Message,
//     Extension(db): Extension<DbConn>,
//     Extension(result): Extension<CommandResult>,
// ) -> HandlerResult<()> {
//     if !result.args.is_empty() {
//         return Ok(());
//     }

//     Ok(())
// }
