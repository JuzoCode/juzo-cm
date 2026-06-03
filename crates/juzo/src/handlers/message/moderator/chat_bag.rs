
/// TODO
pub async fn show(juzo: &JuzoChatManager) {
    let sweets = 0;

    bot.send(
        JuzoAnswer::message(&message)
            .text(format!(
                "{0} В кубышке чата сейчас лежит {1}",
                smail_sweets(true),
            ))
    ).await?;
}