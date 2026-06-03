

pub async fn price_rub_sweets(
    juzo: &JuzoChatManager,
    Extension(result): Extension<CommandResult>,
) {
    let user_ind = UserIndex::new(&bot, &db);

    let first_line: Vec<&str> = result.first_line
        .split_whitespace()
        .collect();

    let amount: u32;

    let mut referrals_id: Option<UserIds> = None;
    let mut my_ids: Option<UserIds> = None;

    match first_line.len() {
        0 | 1 => {
            if !first_line.is_empty() {
                amount = parse_k_kk(first_line[0]).to_u32().unwrap_or_default();
            } else { amount = 0 }
        },
        2 => {
            amount = parse_k_kk(first_line[0]).to_u32().unwrap_or_default();

            let user_line = first_line[1]; 
            if user_line != "@" {
                if !juzo_user.valid_user_link(user_line) { return }
                referrals_id = match juzo_user.search_user(user_line).await {
                    Ok(user_id) => Some(user_id), Err(_) => return,
                };
            }
        },
        _ => return
    }

    let is_private = juzo.chat_type() == "private";
    let referrals = ReferralsSystem::new(amount, &juzo.bot);

    let mut prices: Vec<(u32, &str, u8)> = vec![];
    if is_private {
        prices.push((10, "  ", 0));
        prices.push((50, "  ", 0));

        if my_ids.is_none() { my_ids = juzo.user_id() }
    }

    prices.push((100, " ", 0));
    prices.push((200, " ", 0));

    prices.push((250, " ", 8));
    prices.push((500, " ", 20));
    prices.push((1_000, "", 20));

    let mut discounts: Vec<(&str, u8)> = vec![];
    discounts.push((" 10К", 25));
    discounts.push(("100К", 30));
    discounts.push(("500К", 35));

    let mut text = String::new();
    writeln!(
        text,
        "<tg-emoji emoji-id='5424799150912838494'>🍭</tg-emoji> <b>Покупка {0}{1}.",
        holiday_choice!(
            &["леденцов", "мандаринок", "тыковок"]
        ),
        if !is_private { " за рубли" } else { "" }
    ).unwrap();

    let is_not_limit_amount = amount > 0 && amount < 700_000;
    if is_not_limit_amount {
        let price_pluralized = pluralize!(
            sweets_price(amount), &["рубль", "рубля", "рублей"]
        );

        writeln!(
            text,
            "\n{0} {1}</b> = <code>{2}</code> {3}",
            smail_sweets(true),
            sweets_text(amount).full_text,
    
            price_pluralized.num,
            price_pluralized.pluralize,
        ).unwrap();
    } else { write!(text, "</b>").unwrap() }

    write!(text, "<blockquote expandable>").unwrap();
    let mut text = list_prices(
        text, my_ids, &referrals, prices
    ).await;
    writeln!(text, "</blockquote>").unwrap();
    let mut text = list_discounts(
        text, my_ids, &referrals, discounts
    ).await;

    let keyboard: InlineKeyboardMarkup;
    
    if is_private {
        writeln!(
            text,
            "\n💬 Чтобы купить {0} нужно ввести команду <code>купить {{число {1}}}</code>",
            holiday_choice!(
                &["леденцы", "мандаринки", "тыковки"]
            ),
            holiday_choice!(
                &["леденцов", "мандаринок", "тыковок"]
            ),
        ).unwrap();

        keyboard = InlineKeyboardMarkup::new(vec![
            vec![
                InlineKeyboardButton::callback(
                    "👛 100", "buy sweets 100"
                ),
                InlineKeyboardButton::callback(
                    "🎒 250", "buy sweets 250"
                ),
            ],
            vec![
                InlineKeyboardButton::callback(
                    "🛍 500", "buy sweets 500"
                ),
            ],
            vec![
                InlineKeyboardButton::callback(
                    "🚚 1 000", "buy sweets 1000"
                ),
                InlineKeyboardButton::callback(
                    "🚚 10 000", "buy sweets 10000"
                ),
            ],
            vec![
                InlineKeyboardButton::callback(
                    "🚢 100 000", "buy sweets 100000"
                ),
                InlineKeyboardButton::callback(
                    "🚢 500 000", "buy sweets 500000"
                ),
            ]
        ]);
    } else if is_not_limit_amount {
        let buy_url = referrals.referrals_buy(
            amount, referrals_id, false
        ).await;

        keyboard = InlineKeyboardMarkup::new(vec![
            vec![
                InlineKeyboardButton::url(
                    format!(
                        "{0} Купить {1}",
                        smail_sweets(false),
                        sweets_text(amount).full_text
                    ),
                    buy_url.parse().unwrap()
                ),
            ]
        ]);
    } else {
        let buy_url = referrals.referrals_add(my_ids, false).await;

        keyboard = InlineKeyboardMarkup::new(vec![
            vec![
                InlineKeyboardButton::url(
                    format!(
                        "{0} Купить {1}",
                        smail_sweets(false),
                        holiday_choice!(&["леденцы", "мандаринки", "тыковки"]),
                    ),
                    buy_url.parse().unwrap()
                ),
            ]
        ]);
    }
    
    let _ = juzo.answer(text)
        .reply_markup(keyboard)
        .await;
}
