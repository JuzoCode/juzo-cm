use juzo_core::{
    db::bot::{managed, prelude::ManagedBot},
    // config::ALLOWEDS_UPDATES,
    payloads::base::encode_token,
};
use sea_orm::{DbConn, EntityTrait, Set, sea_query::OnConflict};
use telers::{
    Extension,
    // enums::UpdateType,
    methods::{
        // SetWebhook,
        GetManagedBotToken,
    },
};

use super::*;

pub async fn set(
    bot: Bot,
    managed: ManagedBotUpdated,
    Extension(db): Extension<DbConn>,
) -> HandlerResult<()> {
    let bot_ids = managed.bot.id;
    let token = bot
        .send(GetManagedBotToken::new(bot_ids))
        .await?;

    // I'm telling you right away: the encoder is only one-way.
    // That is, it cannot be decoded, but it is also particularly important.
    // No one will get it in any form.
    let secret_token = unsafe { encode_token(token.as_bytes()) };

    let model = managed::ActiveModel {
        bot_ids: Set(bot_ids.into()),
        creator_ids: Set(managed.user.id.into()),
        managed_bot: Set(bot.id.into()),
        token: Set(token.into()),
        secret_token: unsafe { Set(str::from_utf8_unchecked(&secret_token).into()) },
        ..Default::default()
    };

    let _ = ManagedBot::insert(model)
        .on_conflict(
            OnConflict::column(managed::Column::BotIds)
                .update_columns([managed::Column::Token, managed::Column::SecretToken])
                .to_owned(),
        )
        .exec(&db)
        .await;

    // bot.send(
    //     SetWebhook::new("JUZO MEOW")
    //         .drop_pending_updates(true)
    //         .allowed_updates(
    //             ALLOWEDS_UPDATES
    //                 .iter()
    //                 .filter(|&&u| u != UpdateType::ManagedBot)
    //                 .copied()
    //         )
    //         .secret_token(secret_token)
    // ).await?;

    Ok(())
}
