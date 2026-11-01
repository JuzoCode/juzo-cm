use juzo_core::{
    common::emojis::smail_tick,
    db::user::{prelude::UserSetting, setting},
    domain::UserModel,
};
use sea_orm::{EntityTrait, Set, sea_query::OnConflict};
use telers::{methods::SendMessage, types::ReplyParameters};

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

}