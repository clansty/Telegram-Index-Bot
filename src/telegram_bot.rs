use teloxide::types::ParseMode;
use teloxide::{dispatching::UpdateHandler, prelude::*, utils::command::BotCommands};

use crate::elastic;

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "display this text.")]
    Start,
    #[command(description = "search for message.")]
    Search(String),
}

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(case![Command::Help].endpoint(help))
        .branch(case![Command::Start].endpoint(help))
        .branch(case![Command::Search(keyword)].endpoint(search));

    Update::filter_message()
        .branch(command_handler)
        .branch(dptree::endpoint(message_handler))
}

async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

async fn search(bot: Bot, keyword: String, msg: Message) -> HandlerResult {
    if keyword.is_empty() {
        bot.send_message(msg.chat.id, "Usage: /search <keyword>")
            .await?;
    } else {
        let res = elastic::search(msg.chat.id.0, &keyword).await;
        match res {
            Err(err) => {
                let _ = bot.send_message(msg.chat.id, format!("搜索失败: {}", err.to_string()));
            }
            Ok(res) => {
                if res.hits.hits.is_empty() {
                    let _ = bot.send_message(msg.chat.id, "搜索结果为空");
                    return Ok(());
                }
                let chat_id = msg.chat.id.to_string().replace("-100", "");
                let hits: Vec<String> = res
                    .hits
                    .hits
                    .iter()
                    .map(|hit| {
                        format!(
                            "<a href=\"https://t.me/c/{}/{}\">{}</a> {}:\n  {}",
                            chat_id,
                            hit.id,
                            hit.id,
                            hit.source.sender_name,
                            hit.highlight
                                .clone()
                                .map_or(hit.source.message.clone(), |h| h.message[0].clone())
                        )
                    })
                    .collect();
                let _ = bot
                    .send_message(msg.chat.id, hits.join("\n\n"))
                    .reply_to_message_id(msg.id.0)
                    .parse_mode(ParseMode::Html)
                    .await;
            }
        }
    }
    Ok(())
}

async fn message_handler(_bot: Bot, msg: Message) -> HandlerResult {
    log::debug!("{:#?}", msg);
    elastic::add_message(msg).await;
    Ok(())
}

pub async fn start() {
    let bot = Bot::from_env();

    Dispatcher::builder(bot, schema())
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
