

/// TODO
async fn scam(
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

    let user: UserModel = match (args, message.reply_to_message()) {
        (Some(spans), _) => {
            let Ok(found_user) = user_ind
                .search_user(&text[spans[0]])
                .await
            else {
                return Ok(());
            };
            found_user
        }
        // SAFETY: Branching does not allow processing the None.
        (None, Some(reply)) => unsafe {
            reply
                .from()
                .unwrap_unchecked()
                .into()
        }
        _ => return Ok(()),
    };

    Ok(())
}
