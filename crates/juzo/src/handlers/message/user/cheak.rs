

/// TODO
pub async fn active(
    juzo: &JuzoChatManager,
    Extension(result): Extension<CommandResult>,
) {
    let user_ind = UserIndex::new(&bot, &db);

    match first_line.len() {
        2 => {
            let user_line = first_line[0]; 
            user_id = match juzo_user.search_user(user_line).await {
                Ok(user_id) => user_id, Err(_) => return,
            };
            active_token = first_line[1].to_string();
        },
        _ => return
    }

    let _ = juzo.answer(format!("{user_id}-{active_token}")).await;
}

/// TODO
pub async fn add(
    juzo: &JuzoChatManager,
    Extension(result): Extension<CommandResult>,
) {

    let sweets = match first_line.parse::<Decimal>() {
        Ok(val) => val, Err(_) => return,
    };

    let bag_score = dec!(0);
    let bag_sweets = dec!(0);
    if sweets.is_zero() {
        let _ = juzo.answer(
            format!(
                "{SMAIL_PENSIL} Невозможно сделать чек с абсолютным значением нуля"
            )
        ).await;
        return
    } else if comment.chars().count() > 125 {
        let _ = juzo.answer(
            format!("{SMAIL_PENSIL} Длина текста превышает 125 символов")
        ).await;
        return
    // } else if is_entity_limit {
    //     let _ = juzo.answer(text_entity_limit).await;
    //     return
    } else if sweets > bag_sweets {
        let _ = juzo.answer(
            format!(
                "{SMAIL_PENSIL} Нет столько {0} в мешке для создания чека",
                holiday_choice!(
                    &["леденцов", "мандаринок", "тыковок"]
                )
            )
        ).await;
        return
    }

    let convey = ConveySweets::new_sweets(sweets, bag_score, donate_score);

    let score_pluralized = donate_score_text(convey.fee_score);

    let get_me = juzo.bot.get_me().await.unwrap();
    let my_ids = juzo.message.from.as_ref().unwrap().id;
    
    let mut signature: String = String::new();
    if !comment.is_empty() {
        signature = format!("<b>💬 Подпись к чеку:</b> {comment}\n");
    }

    if juzo.chat_type() != "private" {
        let mut text = String::new();
        writeln!(
            text,
            "💳 Чек <code>{{num}}</code> на {0} создан.",
            pluralize_sweets(sweets, &["леденец", "мандаринку", "тыковку"]).pluralize,
        ).unwrap();
        write!(
            text,
            "<blockquote>💬 Подробности отправлены в <a href='{0}'>ЛС Джузы</a></blockquote>",
            get_me.tme_url(),
        ).unwrap();

        let _ = juzo.answer(text).await;
    };

    let token_cheak = format!("{my_ids}-test_activate");
    let activate_link = format!(
        "{0}?start=activate_check_{token_cheak}",
        get_me.tme_url()
    );

    let mut quote_details = String::new();
    let mut quote_details_send = String::new();

    writeln!(
        quote_details,
        "{0} {1}",
        smail_sweets(true),
        sweets_text(sweets).pluralize
    ).unwrap();

    match convey.require_fee(bag_sweets, UserIds(0)) {
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

    let _ = quote_details_send;

    write!(quote_details, "{signature}").unwrap();

    let quote_end = "</blockquote>";
    let quote_start = "<blockquote expandable>";

    let mut text = "💳 <b>Чек <code>{num}</code> создан.</b>".to_string();
    text.push_str(quote_start);
    text.push_str(&quote_details);
    text.push_str(quote_end);

    writeln!(text, "\n🔗 <a href='{activate_link}'>Ссылка для активации чека.</a>").unwrap();
    writeln!(text,
        "<tg-emoji emoji-id='5330115548900501467'>🔑</tg-emoji> <code>активировать чек {token_cheak}</code>"
    ).unwrap();

    text.push_str(
        "\n<tg-emoji emoji-id='5420323339723881652'>❗️</tg-emoji> <b>Не показывайте это сообщение и код никому кроме получателя.</b>"
    );

    let _ = juzo.answer(text)
        .chat_id(my_ids)
        .protect_content(true)
        .reply_to(MessageId(0))
        .await;
}

