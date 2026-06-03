

/// TODO
pub async fn delete_message_purge(
    juzo: &JuzoChatManager,
    Extension(result): Extension<CommandResult>,
    quietly: bool,
) {
    if juzo.chat_type() == "private" { return } 

    let first_line: Vec<&str> = result.first_line.split_whitespace().collect();
    let mut limit_id: MessageId = MessageId(1000);
    let mut is_limit: bool = false;

    match (first_line.len(), juzo.message.reply_to_message()) {
        (args, Some(_)) if args >= 1 => {
            limit_id = match first_line[0].parse::<i32>() {
                Ok(num) => MessageId(num), Err(_) => return,
            };
            is_limit = true
        },
        (args, Some(_)) if args <= 1 => (),
        _ => {
            let _ = juzo.answer(format!(
                "{SMAIL_PENSIL} Для работы команды ответьте на любое сообщение, чтобы запустить удаление сообщений вниз."
            )).await;
            return
        }
    };

    let (limit_ids, _) = (limit_id.0, is_limit);
    if limit_ids <= 0 { return }

    let is_premium = is_premium_bot(&juzo.bot).await;

    // let end_msg_id = juzo.message.id;
    // let mut delete_ids = vec![end_msg_id];

    match is_premium {
        true => if limit_id.0 > 1000 { limit_id = MessageId(1000); },
        false => if limit_id.0 > 100 { limit_id = MessageId(100) },
    }

    let _ = limit_id;

    // let start_msg_id = juzo.message.reply_to_message().unwrap().id;
    // let count_to_add = (end_msg_id.0.saturating_sub(start_msg_id.0)..end_msg_id.0).count() - 1;
    // if !is_limit { limit_id = MessageId(count_to_add as i32) }

    // delete_ids.reserve(count_to_add);
    // for i in (end_msg_id.0.saturating_sub(start_msg_id.0)..end_msg_id.0).rev() {
    //     if i <= limit_id.0 {
    //         delete_ids.push(MessageId(i));
    //         if !is_premium { sleep(Duration::from_millis(2)).await; }
    //     }
    // }

    // let _ = juzo.purge(delete_ids, is_premium).await;

    if !quietly {
        let _ = juzo.answer(
            format!(
                "{smail_tick} Удаление {}, выполнено",
                pluralize!(
                    0, &[
                        "сообщения", "сообщения", "сообщений"
                    ]
                ),
            )
        ).await;
    } else { return }
}

pub async fn delete_message(
    juzo: &JuzoChatManager,
    Extension(result): Extension<CommandResult>,
    quietly: bool,
) {
    let first_line: Vec<&str> = result.first_line.split_whitespace().collect();

    let mut delete_messages: bool = false;
    let mut message_id: MessageId;

    match (first_line.len(), juzo.message.reply_to_message()) {
        (args, _) if args >= 1 => {
            delete_messages = true;
            message_id = match first_line[0].parse::<i32>() {
                Ok(num) => MessageId(num), Err(_) => return,
            }
        },
        (args, Some(reply)) if args <= 1 => { message_id = reply.id },
        _ => return
    };

    if message_id.0 <= 0 { return }

    let is_premium = is_premium_bot(&juzo.bot).await;
    
    let start_msg_id = juzo.message.id;
    let mut delete_ids = vec![start_msg_id];
    
    if delete_messages {
        match is_premium {
            true => if message_id.0 > 1000 { message_id = MessageId(1000); },
            false => if message_id.0 > 100 { message_id = MessageId(100) },
        }

        let end_msg_id = start_msg_id.0.saturating_sub(message_id.0);
        let count_to_add = (end_msg_id..start_msg_id.0).count() - 1;

        delete_ids.reserve(count_to_add);
        for i in (end_msg_id..start_msg_id.0).rev() {
            delete_ids.push(MessageId(i));
            if !is_premium { sleep(Duration::from_millis(2)).await }
        }
    } else {
        delete_ids.push(message_id)
    }

    let _ = juzo.purge(delete_ids, is_premium).await;

    if !quietly && delete_messages {
        let _ = juzo.answer(
            format!(
                "{smail_tick} Удаление {}, выполнено",
                pluralize!(
                    message_id, &[
                        "сообщения", "сообщения", "сообщений"
                    ]
                ),
            )
        ).await;
    } else { return }
}