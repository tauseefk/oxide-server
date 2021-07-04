use std::sync::Arc;

use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status, Code};

use oxide::chat_service_server::{ChatService, ChatServiceServer};
use oxide::{
    Chat, CreateNewChatRequest, FetchChatsForUserRequest, FetchTextsForChatRequest,
    FetchUsersRequest, LoginInfo, SendTextToUserRequest, SignupOrLoginUserRequest, Text, TextSent,
    User,
};

mod model;

pub mod oxide {
    tonic::include_proto!("oxide");
}

mod data;

#[derive(Debug, Default)]
pub struct OxideService {
    texts: Arc<Vec<Text>>,
    chats: Arc<Vec<Chat>>,
}

#[tonic::async_trait]
impl ChatService for OxideService {
    type FetchTextsForChatStream = ReceiverStream<Result<Text, Status>>;
    type FetchChatsForUserStream = ReceiverStream<Result<Chat, Status>>;
    type FetchUsersStream = ReceiverStream<Result<User, Status>>;

    async fn fetch_texts_for_chat(
        &self,
        request: Request<FetchTextsForChatRequest>,
    ) -> Result<Response<Self::FetchTextsForChatStream>, Status> {
        let chat_id = request.get_ref().chat_id.clone();
        let (tx, rx) = mpsc::channel(4);

        let texts = model::oxide_db::get_texts_for_chat(&chat_id)
            .await
            .unwrap();

        tokio::spawn(async move {
            for text in &texts[..] {
                println!("{:?}", text.id);
                tx.send(Ok(Text {
                    id: text.id.clone(),
                    content: text.content.clone(),
                    from: text.from.clone(),
                    chat_id: text.chat_id.clone(),
                }))
                .await
                .unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn create_new_chat(
        &self,
        request: Request<CreateNewChatRequest>,
    ) -> Result<Response<Chat>, Status> {
        let req = request.get_ref();
        println!("Create new chat between: {:?} {:?}", req.from, req.to);

        match model::oxide_db::create_empty_chat(&req.from, &req.to).await {
            Ok(chat_id) => Ok(Response::new(Chat {
                id: chat_id,
                participant_ids: vec![req.from.clone(), req.to.clone()],
            })),
            Err(_) => {
                Err(Status::new(Code::InvalidArgument, "could not create chat"))
            }
        }
    }

    async fn fetch_chats_for_user(
        &self,
        request: Request<FetchChatsForUserRequest>,
    ) -> Result<Response<Self::FetchChatsForUserStream>, Status> {
        let user_id = request.get_ref().user_id.clone();
        println!("Fetch Chats for User with id: {:?}", user_id);

        let (tx, rx) = mpsc::channel(4);
        let chats = model::oxide_db::get_chats_for_user(&user_id)
            .await
            .unwrap();

        tokio::spawn(async move {
            for chat in &chats[..] {
                tx.send(Ok(Chat {
                    id: chat.id.clone(),
                    participant_ids: chat.participant_ids.clone(),
                }))
                .await
                .unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn fetch_users(
        &self,
        _: Request<FetchUsersRequest>,
    ) -> Result<Response<Self::FetchUsersStream>, Status> {
        println!("Fetch all users");

        let (tx, rx) = mpsc::channel(4);
        let users = model::oxide_db::get_all_users().await.unwrap();

        tokio::spawn(async move {
            for user in &users[..] {
                tx.send(Ok(User {
                    id: user.id.clone(),
                    name: user.id.clone(),
                }))
                .await
                .unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn login_user(
        &self,
        request: Request<SignupOrLoginUserRequest>,
    ) -> Result<Response<LoginInfo>, Status> {
        let request_obj = request.get_ref();
        let mut login_info = self::LoginInfo { logged_in: false };

        match model::oxide_db::authenticate_user(&request_obj.username, &request_obj.password)
            .await
        {
            Ok(is_logged_in) => {
                login_info.logged_in = is_logged_in;
                Ok(Response::new(login_info))
            }
            Err(_) => {
                println!("Error fetching document");
                Ok(Response::new(login_info))
            }
        }
    }

    async fn signup_user(
        &self,
        request: Request<SignupOrLoginUserRequest>,
    ) -> Result<Response<LoginInfo>, Status> {
        let request_obj = request.get_ref();
        let mut login_info = self::LoginInfo { logged_in: false };
        let mut user_already_exists = false;

        match model::oxide_db::get_user_for_id(&request_obj.username).await {
            Ok(has_user) => {
                user_already_exists = has_user;
            }
            Err(_) => {
                println!("Error fetching document");
            }
        }

        if user_already_exists == true {
            Ok(Response::new(login_info))
        } else {
            match model::oxide_db::create_user(&request_obj.username, &request_obj.password)
                .await
            {
                Ok(user_created) => {
                    login_info.logged_in = user_created;
                }
                Err(_) => {
                    println!("Error creating user");
                }
            }
            Ok(Response::new(login_info))
        }
    }

    async fn send_text_to_user(
        &self,
        request: Request<SendTextToUserRequest>,
    ) -> Result<Response<TextSent>, Status> {
        let request_obj = request.get_ref();

        if let Err(_err) = model::oxide_db::send_text_to_user(
            &request_obj.chat_id,
            &request_obj.content,
            &request_obj.from,
        )
        .await
        {
            println!("User not found");
        } else {
            println!("Attempting sending text for {:?}", request_obj.content);
        }
        let text_sent = self::TextSent {
            text: String::from("sent"),
        };

        Ok(Response::new(text_sent))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:3001".parse()?;

    let texts = Arc::new(data::load_texts());
    let chats = Arc::new(data::load_chats());

    let fetch_texts_for_chat = OxideService { texts, chats };

    let service = ChatServiceServer::new(fetch_texts_for_chat);

    Server::builder().add_service(service).serve(addr).await?;

    Ok(())
}
