use crate::types::search_result::SearchResult;
use elasticsearch::{
    http::transport::Transport, indices::*, Elasticsearch, Error, IndexParts, SearchParts,
};
use once_cell::sync::Lazy;
use serde_json::json;

static CLIENT: Lazy<Elasticsearch> = Lazy::new(|| {
    Elasticsearch::new(Transport::single_node(&std::env::var("ELASTIC_ENDPOINT").unwrap()).unwrap())
});

pub async fn add_message(msg: teloxide::prelude::Message) {
    let response = CLIENT
        .indices()
        .create(IndicesCreateParts::Index(&format!(
            "telegram_index_{}",
            msg.chat.id
        )))
        .body(json!({
            "mappings": {
                "properties": {
                    "message": {
                        "type": "text",
                        "analyzer": "ik_max_word",
                        "search_analyzer": "ik_smart"
                    },
                    "senderName": {
                        "type": "text",
                        "analyzer": "ik_max_word",
                        "search_analyzer": "ik_smart"
                    },
                    "senderUsername": {
                        "type": "keyword",
                    },
                    "date": {
                        "type": "date",
                    },
                    "chatId": {
                        "type": "long"
                    },
                    "id": {
                        "type": "integer"
                    },
                    "senderId": {
                        "type": "long"
                    }
                }
            }
        }))
        .send()
        .await;
    let response_body = response.expect("REASON").json::<serde_json::Value>().await;
    log::debug!("{:#?}", response_body);
    let msg_text = msg.text().unwrap_or(msg.caption().unwrap_or_default());
    if msg_text.is_empty() {
        return;
    }
    let response  = CLIENT
        .index(IndexParts::IndexId(
            &format!("telegram_index_{}", msg.chat.id),
            &msg.id.to_string(),
        ))
        .body(json!({
            "id": msg.id.0,
            "chatId": msg.chat.id,
            "senderId": msg.sender_chat().map(|u|u.id.0).or(msg.from().map(|u|u.id.0 as i64)).unwrap_or_default(),
            "senderName": msg.sender_chat()
                .map(|u| u.title().or(u.first_name()))
                .unwrap_or(msg.from().map(|u| u.first_name.as_str())),
            "senderUsername": msg.sender_chat()
                .map(|u| u.username().map(|it|it.to_owned()))
                .or(msg.from().map(|u| u.username.clone()))
                .unwrap_or_default(),
            "date": msg.date.to_rfc3339(),
            "message": msg_text
        }))
        .send()
        .await;
    let response_body = response.expect("REASON").json::<serde_json::Value>().await;
    log::debug!("{:#?}", response_body);
}

pub async fn search(chat_id: i64, keyword: &str) -> Result<SearchResult, Error> {
    let response = CLIENT
        .search(SearchParts::Index(&[&format!(
            "telegram_index_{}",
            chat_id
        )]))
        .body(json!({
            "query": {
                "simple_query_string" : {
                    "query": keyword,
                    "fields": ["message", "senderName", "senderUsername"],
                    "default_operator": "and",
                    "analyzer": "ik_smart"
                }
            },
            "highlight" : {
                "pre_tags" : ["<b>"],
                "post_tags" : ["</b>"],
                "fields" : {
                    "message" : {}
                }
            }
        }))
        .send()
        .await;
    match response {
        Ok(res) => res.json::<SearchResult>().await,
        Err(err) => Err(err),
    }
}
