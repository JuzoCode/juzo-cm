

/// TODO
pub async fn start_buying_sweets(
    juzo: &JuzoChatManager,
    Extension(result): Extension<CommandResult>,
) {
    let is_pay_bot = is_pay_bot(&juzo.bot).await; 

    if juzo.chat_type() != "private" && is_pay_bot {
        return price_rub_sweets(juzo, result).await
    }

    let first_line = &result.first_line;
    
    let amount = parse_k_kk(&first_line).to_u32().unwrap_or_default();
    
    let text = format!(
        "{0} <b>Выберите метод оплаты</b> для приобретения <b>{1}</b>",
        smail_sweets(true),
        holiday_choice!(
            &["леденцов", "мандаринок", "тыковок"]
        ),
    );

    if is_pay_bot {
        let mut my_ids: Option<UserIds> = None;
        let mut message_send = juzo.answer("");

        if juzo.chat_type() == "private" {
            if my_ids.is_none() { my_ids = juzo.user_id() }
        } else {
            let get_me = juzo.bot.get_me().await.unwrap();

            message_send = message_send.chat_id(juzo.message.from.as_ref().unwrap().id)
                .reply_to(MessageId(0));

            let _ = juzo.answer(
                format!(
                    "{text} в <a href='{0}'>ЛС Джузы</a>",
                    get_me.tme_url(),
                )
            ).await;
        }

        let referrals = ReferralsSystem::new(amount, &juzo.bot);

        let juzo_buy_url = referrals.referrals_buy(amount, my_ids, false).await;
        let iris_buy_url = referrals.referrals_buy(amount, my_ids, true).await;

        let keyboard = InlineKeyboardMarkup::new(vec![
            vec![
                InlineKeyboardButton::url(
                    format!(
                        "{0} {1}ми",
                        smail_sweets(false),
                        holiday_choice!(
                            &["Ириска", "Мандаринка", "Тыковка"]
                        )
                    ),
                    iris_buy_url.parse().unwrap()
                ),
                InlineKeyboardButton::url(
                    "💶 Рублями", juzo_buy_url.parse().unwrap()
                ),
            ]
        ]);

        let _ = message_send.text(text)
            .reply_markup(keyboard)
            .await;
        return
    }

    buying_sweets(juzo, result, "xtr").await
}

/// TODO
pub async fn buying_tg_stars(
    juzo: &JuzoChatManager,
    Extension(result): Extension<CommandResult>,
) {
    let first_line = &result.first_line;
    
    let amount = parse_k_kk(&first_line).to_u32().unwrap_or_default();
    let price = tg_stars_price(amount);

    let tg_stars_pluralize = pluralize!(
        amount, &["тг-звезду", "тг-звезды", "тг-звёзд"]
    );

    let limit_amount = price_limit("tg stars");
    if amount > limit_amount {
        let _ = juzo.answer(
            "📝 В миллион тг-звёзды ещё могут быть реальностью, но всё, что выше — это уже просто показуха."
        ).await;
        return
    }

    let mut text = String::new();
    writeln!(
        text,
        "{0} Вы собираетесь пополниться на {tg_stars_pluralize}.",
        smail_tg_stars(true),
    ).unwrap();
    write!(
        text,
        "{0} <b>Цена:</b> {1}",
        smail_sweets(true),
        sweets_text(price),
    ).unwrap();

    let _ = juzo.answer(text).await;
}

/// TODO
pub async fn buying_sweets(
    juzo: &JuzoChatManager,
    Extension(result): Extension<CommandResult>,
    price_type: &str,
) {
    let is_pay_bot = is_pay_bot(&juzo.bot).await; 
    let is_private = juzo.chat_type() != "private";
    
    if !vec!["xtr", "juzo", "iris"].contains(&price_type) {
        return
    } else if (!is_pay_bot && price_type != "xtr") || (is_pay_bot && price_type == "xtr") {
        return
    }

    if price_type == "iris" {
        let _ = juzo.answer(
            format!(
                "{SMAIL_PENSIL} На данный момент этот метод оплаты недоступен, выберите другой"
            )
        ).await;
        return
    }

    let user_ind = UserIndex::new(&bot, &db);

    let first_line_str = result.first_line
        .replacen("_currub=", " ", 1);

    let first_line: Vec<&str> = first_line_str
        .split_whitespace()
        .collect();

    let amount: u32;

    let mut referrals_id: Option<UserIds> = None;

    match first_line.len() {
        1 => amount = parse_k_kk(first_line[0]).to_u32().unwrap_or_default(),
        2 => {
            if !is_pay_bot { return }
            
            let user_line = first_line[1]; 
            referrals_id = match juzo_user.search_user(user_line).await {
                Ok(user_id) => Some(user_id), Err(_) => return,
            };
            
            amount = parse_k_kk(first_line[0]).to_u32().unwrap_or_default();
        },
        _ => return
    }

    let referrals = ReferralsSystem::new(amount, &juzo.bot);

    // let meow = (amount, referrals_id, price_type);

    let limit_amount = price_limit(price_type);
    if amount > limit_amount {
        let _ = juzo.answer(
            format!(
                "{SMAIL_PENSIL} Мы переживаем за ваше здоровье и что у \
                вас что-нибудь слипнется. Ограничьте свой запрос хотя бы на {0}.",
                sweets_text(limit_amount).full_text,
            )
        ).await;
        return
    } else if amount < 10 {
        let mut buy_url: Option<String> = None;

        if price_type == "iris" {
            buy_url = Some(referrals.referrals_buy(
                10, referrals_id, true
            ).await);
        } else if price_type == "juzo" {
            buy_url = Some(referrals.referrals_buy(
                10, referrals_id, false
            ).await);
        }

        let mut keyboard = InlineKeyboardMarkup::new(vec![vec![]]);
        if let Some(url) = buy_url {
            keyboard = InlineKeyboardMarkup::new(vec![
                vec![
                    InlineKeyboardButton::url(
                        format!(
                            "{0} {1}",
                            smail_sweets(false),
                            sweets_text(10).full_text
                        ),
                        url.parse().unwrap()
                    )
                ]
            ]);
        }
        
        let _ = juzo.answer(
            format!(
                "{SMAIL_PENSIL} Минимальное количество для приобретения: {0}.",
                sweets_text(10).full_text,
            )
        )
        .reply_markup(keyboard)
        .await;
        return
    }
    
    let mut text = String::new();
    writeln!(
        text,
        "🗓 <b>Пополнение запасов {0}</b>",
        holiday_choice!(
            &["леденцов", "мандаринок", "тыковок"]
        ),
    ).unwrap();
    write!(
        text,
        "{0} <b>{1}</b> = \"руб. -> Pluralize.full_text is None\"",
        smail_sweets(true),
        sweets_text(amount)
    ).unwrap();

    let mut message_send = juzo.answer("");
    if is_private {
        let get_me = juzo.bot.get_me().await.unwrap();

        message_send = message_send.chat_id(juzo.message.from.as_ref().unwrap().id)
            .reply_to(MessageId(0));

        let _ = juzo.answer(
            format!(
                "{0} Информация о пополнении {1} отправлена в <a href='{2}'>ЛС Джузы</a>",
                smail_sweets(true),
                holiday_choice!(
                    &["леденцов", "мандаринок", "тыковок"]
                ),
                get_me.tme_url(),
            )
        ).await;
    }

    if price_type == "iris" {
        return
    } else if price_type == "juzo" {
        
    } else  {
        let _ = juzo.bot.send_invoice(
            juzo.user_id().unwrap(),
            format!(
                "Покупка {0} за ТГ-звёзды.",
                holiday_choice!(
                    &["леденцов", "мандаринок", "тыковок"]
                )
            ),
            format!(
                "{0} Пополнение мешка на {1}",
                smail_sweets(false),
                sweets_text(amount).full_text,
            ),
            "buy XTR-sweets",
            "XTR",
            vec![LabeledPrice { label: String::from("XTR"), amount: amount }],
        ).await;
        return
    }

    let _ = message_send.text(text).await;
}
