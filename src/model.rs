pub mod oxide_db {
  use futures::stream::TryStreamExt;
  use mongodb::bson::doc;
  use mongodb::{self, Client, Collection};
  use rand::Rng;
  use serde::{Deserialize, Serialize};

  #[derive(Debug, Serialize, Deserialize)]
  pub struct User {
    pub id: String,
  }

  #[derive(Debug, Serialize, Deserialize)]
  pub struct Chat {
    pub id: String,
    pub participant_ids: Vec<String>,
    pub text_ids: Vec<String>,
  }

  #[derive(Debug, Serialize, Deserialize)]
  pub struct Text {
    pub id: String,
    pub content: String,
    pub from: String,
    pub chat_id: String,
  }

  static CLIENT_URI: &str = "mongodb://localhost:27017";
  const PASS: &str = "password";

  pub async fn get_db_client() -> Result<mongodb::Client, Box<dyn std::error::Error>> {
    let options = mongodb::options::ClientOptions::parse_with_resolver_config(
      &CLIENT_URI,
      mongodb::options::ResolverConfig::cloudflare(),
    )
    .await?;
    Ok(Client::with_options(options)?)
  }

  pub async fn get_user_for_id(id: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let db_client = get_db_client().await?;

    let users: Collection = db_client.database("oxide").collection("user");

    match users
      .find_one(
        doc! {
              "id": id,
        },
        None,
      )
      .await
    {
      Ok(doc) => match doc {
        Some(_) => Ok(true),
        None => {
          println!("Missing Document!");
          Ok(false)
        }
      },
      Err(e) => Err(Box::new(e)),
    }
  }

  pub async fn authenticate_user(
    id: &str,
    password: &str,
  ) -> Result<bool, Box<dyn std::error::Error>> {
    let db_client = get_db_client().await?;

    let users: Collection = db_client.database("oxide").collection("user");

    match users
      .find_one(
        doc! {
              "id": id,
        },
        None,
      )
      .await
    {
      Ok(doc) => match doc {
        Some(d) => match d.get_str(PASS) {
          Ok(p) => Ok(p.eq(password)),
          Err(_) => Ok(false),
        },
        None => {
          println!("Missing Document!");
          Ok(false)
        }
      },
      Err(e) => Err(Box::new(e)),
    }
  }

  pub async fn get_all_users() -> Result<Vec<User>, Box<dyn std::error::Error>> {
    let db_client = get_db_client().await?;

    let users: Collection<User> = db_client.database("oxide").collection("user");

    let mut cursor = users.find(None, None).await?;

    let mut result: Vec<User> = Vec::new();
    while let Some(user) = cursor.try_next().await? {
      result.push(User { id: user.id });
    }

    println!("{:?}", result);
    Ok(result)
  }

  pub async fn create_user(
    username: &str,
    password: &str,
  ) -> Result<bool, Box<dyn std::error::Error>> {
    let db_client = get_db_client().await?;
    let users: Collection = db_client.database("oxide").collection("user");

    let new_user = doc! {
      "id": username,
      "password": password
    };

    let insert_result = users.insert_one(new_user.clone(), None).await?;

    println!("New document created: {}", insert_result.inserted_id);

    Ok(true)
  }

  pub async fn get_chats_for_user(username: &str) -> Result<Vec<Chat>, Box<dyn std::error::Error>> {
    let db_client = get_db_client().await?;
    let chats: Collection<Chat> = db_client.database("oxide").collection("chats");

    let filter = doc! { "participant_ids": username };

    let mut cursor = chats.find(filter, None).await?;

    let mut result: Vec<Chat> = Vec::new();
    println!("fetching chats for: {:?}", username);
    while let Some(chat) = cursor.try_next().await? {
      println!("{:?}", chat.id);
      result.push(Chat {
        id: chat.id,
        participant_ids: chat.participant_ids,
        text_ids: chat.text_ids,
      });
    }

    Ok(result)
  }

  pub async fn get_texts_for_chat(id: &str) -> Result<Vec<Text>, Box<dyn std::error::Error>> {
    let db_client = get_db_client().await?;

    let texts: Collection<Text> = db_client.database("oxide").collection("texts");
    // Remove for debug
    let filter = doc! { "chat_id": &id };

    let mut cursor = texts.find(filter, None).await?;

    let mut result: Vec<Text> = Vec::new();
    while let Some(text) = cursor.try_next().await? {
      result.push(Text {
        id: text.id,
        content: text.content,
        from: text.from,
        chat_id: text.chat_id,
      });
    }

    Ok(result)
  }

  pub async fn send_text_to_user(
    chat_id: &str,
    content: &str,
    from: &str,
  ) -> Result<(), Box<dyn std::error::Error>> {
    let text_id: u32 = rand::thread_rng().gen();

    let db_client = get_db_client().await?;

    let texts: Collection = db_client.database("oxide").collection("texts");
    let new_text =
      doc! { "content": content, "from": from, "id": text_id.to_string(), "chat_id": chat_id };

    let insert_result = texts.insert_one(new_text.clone(), None).await?;

    println!("New document created: {}", insert_result.inserted_id);

    Ok(())
  }

  pub async fn create_empty_chat(from: &str, to: &str) -> Result<String, Box<dyn std::error::Error>> {
    let chat_id: u32 = rand::thread_rng().gen();

    let db_client = get_db_client().await?;

    let chats: Collection = db_client.database("oxide").collection("chats");
    let new_chat =
      doc! { "participant_ids": vec![from.to_string(), to.to_string()], "text_ids": [], "id": chat_id.to_string() };

    let insert_result = chats.insert_one(new_chat.clone(), None).await?;

    println!("New document created: {:?}", insert_result);

    Ok(chat_id.to_string())
  }
}
