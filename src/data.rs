  
use serde::Deserialize;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Text {
    id: String,
    content: String,
    from: String,
    chat_id: String
}

#[derive(Debug, Deserialize)]
struct Chat {
    id: String,
    participant_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct User {
    id: String,
    name: String
}

#[allow(dead_code)]
pub fn load_texts() -> Vec<crate::oxide::Text> {
    let file = File::open("./src/texts.json").expect("failed to open data file");

    let decoded: Vec<Text> =
        serde_json::from_reader(&file).expect("failed to deserialize texts");

    decoded
        .into_iter()
        .map(|text| crate::oxide::Text {
            id: text.id,
            content: text.content,
            from: text.from,
            chat_id: text.chat_id
        })
        .collect()
}

#[allow(dead_code)]
pub fn load_chats() -> Vec<crate::oxide::Chat> {
    let file = File::open("./src/chats.json").expect("failed to open data file");

    let decoded: Vec<Chat> =
        serde_json::from_reader(&file).expect("failed to deserialize chats");

    decoded
        .into_iter()
        .map(|chat| crate::oxide::Chat {
            id: chat.id,
            participant_ids: chat.participant_ids,
        })
        .collect()
}

#[allow(dead_code)]
pub fn load_users() -> Vec<crate::oxide::User> {
    let file = File::open("./src/users.json").expect("failed to open data file");

    let decoded: Vec<User> =
        serde_json::from_reader(&file).expect("failed to deserialize users");

    decoded
        .into_iter()
        .map(|user| crate::oxide::User {
            id: user.id,
            name: user.name,
        })
        .collect()
}
