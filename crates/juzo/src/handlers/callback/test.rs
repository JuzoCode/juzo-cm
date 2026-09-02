use telers::methods::AnswerCallbackQuery;

use super::*;

pub async fn ping(
    bot: Bot,
    callback: CallbackQuery,
    Extension(db): Extension<DbConn>,
) -> HandlerResult<()> {
    let module = ModuleChecker::new(&bot, &db);
    let true = module
        .check::<35>(ModuleAccess::C(&callback))
        .await
    else {
        return Ok(());
    };
    let from = callback.from;
    let username = from
        .username
        .unwrap_or_else(|| {
            let s = from.id.to_string();
            Box::from(s.as_str())
        });

    bot.send(
        AnswerCallbackQuery::new(callback.id)
            .text(format!("{username} котик"))
            .cache_time(1200),
    )
    .await?;

    Ok(())
}
