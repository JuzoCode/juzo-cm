use tokio::time::{sleep, Duration};
use std::fmt::Write;

use teloxide::sugar::request::RequestReplyExt;

use crate::{
    juzo_cm::{
        bot::regexp::RegexpResult,
        user::{
            game::JuzoGame,
            users::{UserIndex, UserIds},
        },
        juzo::JuzoChatManager,
    },
    common::emojis::SMAIL_PENSIL,
};

/// TODO
pub async fn cube(
    juzo: &JuzoChatManager,
    Extension(result): Extension<CommandResult>,
) {
    if juzo.chat_type() == "private" { return } 

    let user_ind = UserIndex::new(&bot, &db);
    
    let first_line: Vec<&str> = result.first_line.split_whitespace().collect();
    let is_link = juzo_user.valid_user_link(first_line.last().unwrap_or(&""));

    let message_from = juzo.message.from.as_ref().unwrap();
    let user_id: UserIds; 
    let mut gold_count: u32 = 0;

    match (first_line.len(), juzo.message.reply_to_message(), is_link) {
        (args, _, true) if args <= 2 => {
            if args != 1 {
                gold_count = match first_line[0].parse::<u32>() {
                    Ok(num) => num, Err(_) => return,
                };
            }
            let user_line = first_line.last().unwrap();
            user_id = match juzo_user.search_user(user_line).await {
                Ok(ok) => ok, Err(_) => return
            };
        },
        (args, Some(reply), false) if args <= 1 => {
            match &reply.from {
                Some(reply_from) => {
                    if reply_from.is_bot {
                        let _ = juzo.answer(
                            format!(
                                "{SMAIL_PENSIL} Нет смысла предлагать бросить кубики боту. Он не ответит..."
                            )
                        ).await;
                        return 
                    };
                },
                None => return
            }

            user_id = juzo.reply_user_id().unwrap_or_default();
            if args == 1 {
                gold_count = match first_line[0].parse::<u32>() {
                    Ok(num) => num, Err(_) => return,
                };
            }
        },
        _ => return
    }

    // let bag_gold_player = dec!(0);
    // let bag_gold_opponent = dec!(0);

    if user_id == message_from.id {
        let _ = juzo.answer(
            format!(
                "{SMAIL_PENSIL} Если вы хотите испытать судьбу, воспользуйтесь командой <code>!русская рулетка</code>. Вперёд!"
            )
        ).await;
        return
    }

    let player_full_name = message_from.id;
    let opponent_full_name = user_id;

    let msg_player = juzo.answer(
        format!("🎲 Первый бросок кубика совершает <b>{player_full_name}</b>")
    ).await.unwrap();

    let message_id = msg_player.id;

    sleep(Duration::from_secs_f32(0.5)).await;

    let dice_response_player = juzo.dise_reply().reply_to(message_id).await.unwrap();
    let dise_player = dice_response_player.dice().unwrap();
    
    sleep(Duration::from_secs_f32(3.5)).await;

    let msg_opponent = juzo.reply(
        format!("🎲 Второй бросок кубика совершает <b>{opponent_full_name}</b>")
    ).reply_to(message_id).await.unwrap();
 
    sleep(Duration::from_secs_f32(0.5)).await;

    let dice_response_opponent = juzo.dise_reply().reply_to(msg_opponent.id).await.unwrap();
    let dise_opponent = dice_response_opponent.dice().unwrap();

    let juzo_game = JuzoGame::new(
        gold_count,
        UserIds::from(message_from.id),
        user_id,
        dise_player.value,
        dise_opponent.value,
    );

    let (win_value, win_user_id) = juzo_game.win_user();
    let (defeat_value, defeat_user_id) = juzo_game.defeat_user();

    let mut text = "🎲 Игроки бросили кубики на доску. ".to_string();

    if juzo_game.equal_number {
        write!(
            text, 
            "И у нас ничья!\n\
            <blockquote expandable>\
            <tg-emoji emoji-id='5452069934089641166'>{win_value}⃣</tg-emoji> <b>Ничья: {win_user_id}, {defeat_user_id}</b>\
            </blockquote>"
        ).unwrap()
    } else {
        write!( 
            text,
            "И вот у нас есть победитель!\n\
            <blockquote expandable>\
            <tg-emoji emoji-id='5440539497383087970'>{win_value}⃣</tg-emoji> <b>Победа: {win_user_id}.</b>\n\
            <tg-emoji emoji-id='5447203607294265305'>{defeat_value}⃣</tg-emoji> Поражение: {defeat_user_id}.\
            </blockquote>"
        ).unwrap()
    };

    text.push_str(&juzo_game.gold_text());

    sleep(Duration::from_secs_f32(3.5)).await;

    let _ = juzo.reply(text).reply_to(message_id).await;
}