/// TODO
pub async fn sweets(
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
    let args = result.args::<2>(text);

    // match args
    
    // UserBalance

    if sweets == 0 {
        let _ = juzo.answer(
            format!(
                "{SMAIL_PENSIL} Эй! Ты планировал передать {0}.",
                sweets_text(0).full_text
            )
        ).await;
        return
    // } else if comment.chars().count() > 125 {
    //     let _ = juzo.answer(
    //         format!("{SMAIL_PENSIL} Длина текста превышает 125 символов.")
    //     ).await;
    //     return
    } else if sweets > bag_sweets {
        let _ = juzo.answer(
            format!(
                "{SMAIL_PENSIL} Нет столько {0} в мешке для передачи.",
                holiday_choice!(
                    &["леденцов", "мандаринок", "тыковок"]
                )
            )
        ).await;
        return
    }

    let convey = ConveySweets::new_sweets(sweets, bag_score, donate_score);

    let score_pluralized = donate_score_text(convey.fee_score);
    let sweets_pluralized = pluralize_sweets(
        sweets, &["леденец", "мандаринку", "тыковку"]
    ).full_text;

    let mut quote_details = String::new();
    let mut quote_details_send = String::new();

    match convey.require_fee(bag_sweets, peer_id) {
        Ok(result) => {
            if !convey.fee_score.is_zero() {
                    writeln!(
                        quote_details,
                        "{0} -{score_pluralized}",
                        smail_score(true),
                    ).unwrap();
                if convey.score > 0 && result.fee.is_zero() {
                    writeln!(
                        quote_details_send,
                        "{0} +{1}",
                        smail_score(true),
                        donate_score_text(convey.score),
                    ).unwrap();
                }
            }

            if !result.fee.is_zero() {
                writeln!(
                    quote_details,
                    "<b>{SMAIL_COFFE} Съедено:</b> {0} ({1}×{2})",
                    sweets_text(result.fee).full_text,
                    convey.to_percentage(),
                    result.amount,
                ).unwrap()
            }
        }
        Err(require) => {
            let mut message = String::new();

            write!(
                message,
                "{SMAIL_PENSIL} Нет столько {0} в мешке для передачи.<blockquote>",
                sweets_text(0).full_text
            ).unwrap();

            if !convey.fee_score.is_zero() {
                writeln!(
                    message,
                    "<b>{0} Имеется:</b> {score_pluralized}",
                    smail_score(true),
                ).unwrap();
            };

            write!(
                message,
                "<b>{SMAIL_COFFE} Нужно:</b> {0}</blockquote>",
                pluralize_sweets(require, &["леденец", "мандаринку", "тыковку"]).full_text,
            ).unwrap();

            let _ = juzo.answer(message).await;
            return;
        }
    }

    let quote_start = "<blockquote expandable>";
    let quote_end = "</blockquote>";

    writeln!(quote_details_send, "<b>👤 Отправитель:</b> {{my_full_name}}").unwrap();
    if !comment.is_empty() {
        let signature = format!("<b>💬 Подпись к переводу:</b> {comment}");
        write!(quote_details, "{signature}").unwrap();
        write!(quote_details_send, "{signature}").unwrap();
    }

    let mut text = String::new();
    writeln!(
        text,
        "{0} {peer_id} получил {sweets_pluralized}",
        smail_sweets(true)
    ).unwrap();
    text.push_str(quote_start);
    text.push_str(&quote_details);
    text.push_str(quote_end);

    let mut text_send = String::new();
    writeln!(
        text_send,
        "{0} Вам перевели {sweets_pluralized}",
        smail_sweets(true),
    ).unwrap();
    text_send.push_str(quote_start);
    text_send.push_str(&quote_details_send);
    text_send.push_str(quote_end);

    let _ = text_send;
    let _ = juzo.answer(text).await;
}

/// TODO
pub async fn donate_point(
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
    let args = result.args::<2>(text);

    // match args
    
    // UserBalance

    if donate_score == 0 {
        let _ = juzo.answer(
            format!(
                "{SMAIL_PENSIL} Эй! Ты планировал передать {0}.",
                donate_score_text(0).full_text
            )
        ).await;
        return
    // } else if comment.chars().count() > 125 {
    //     let _ = juzo.answer(
    //         format!("{SMAIL_PENSIL} Длина текста превышает 125 символов.")
    //     ).await;
    //     return
    } else if donate_score > bag_score {
        let _ = juzo.answer(
            format!(
                "{SMAIL_PENSIL} Нет столько {0} в мешке для передачи.",
                donate_score_text(0).pluralize
            )
        ).await;
        return
    }

    let mut text = String::new();
    let mut text_send = String::new();

    let score_pluralize = donate_score_text(donate_score);

    writeln!(
        text,
        "{0} {peer_id} получил {score_pluralize}",
        smail_score(true),
    ).unwrap();

    writeln!(
        text_send,
        "{0} Вам перевели {score_pluralize}а\n\
        <blockquote expandable>\
        <b>👤 Отправитель:</b> {1}",
        smail_score(true),
        "user_full_name",
    ).unwrap();

    if !comment.is_empty() {
        let signature = format!("<b>💬 Подпись к переводу:</b> {comment}");
        write!(text, "<blockquote expandable>{signature}</blockquote>").unwrap();
        write!(text_send, "{signature}</blockquote>").unwrap();
    } else { text_send.push_str("</blockquote>") }

    let _ = juzo.answer(text).await;
}