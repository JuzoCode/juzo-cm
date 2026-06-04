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
    bot: Bot,
    message: Message,
    Extension(_db): Extension<DbConn>,
    Extension(_result): Extension<CommandResult>,
) {
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

    // let bag_gold_player = dec!(0);
    // let bag_gold_opponent = dec!(0);

    if user_id == message_from.id {
        let _ = (
            format!(
                "{0} Если вы хотите испытать судьбу, воспользуйтесь командой <code>!русская рулетка</code>."
            )
        );
        return
    }

    let player_full_name = message_from.id;
    let opponent_full_name = user_id;

    let msg_player = (
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

    (text).reply_to(message_id);
}