use num_bigint::BigUint;
use std::{collections::HashMap, sync::Mutex};
use tonic::{transport::Server, Code, Request, Response, Status};

pub mod nillion {
    include!("./nillion.rs");
}

use nillion::{
    auth_server::{Auth, AuthServer},
    AuthenticationAnswerRequest, AuthenticationAnswerResponse, AuthenticationChallengeRequest,
    AuthenticationChallengeResponse, RegisterRequest, RegisterResponse,
};

#[derive(Debug, Default)]
struct AuthImpl {
    pub user_info: Mutex<HashMap<String, UserInfo>>,
    pub auth_id_to_user: Mutex<HashMap<String, String>>,
}

#[derive(Debug, Default)]
pub struct UserInfo {
    pub username: String, // Registered Username String
    pub y1: BigUint,      // Registered secret y1
    pub y2: BigUint,      // Registered secret y2
    pub r1: BigUint,
    pub r2: BigUint,
    pub challenge: BigUint,
    pub solution: BigUint,
    pub session_id: String,
}

#[tonic::async_trait]
impl Auth for AuthImpl {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let request = request.into_inner();
        let username = request.user;

        println!("Processing Auth::register() for {}", username);

        let user_info = UserInfo {
            username: username.clone(),
            y1: BigUint::from_bytes_be(&request.y1),
            y2: BigUint::from_bytes_be(&request.y2),
            ..Default::default()
        };

        let user_info_hashmap = &mut self.user_info.lock().unwrap();
        user_info_hashmap.insert(username.clone(), user_info);

        println!("✅ Successful Registration username: {:?}", username);

        Ok(Response::new(RegisterResponse {}))
    }

    async fn create_authentication_challenge(
        &self,
        request: Request<AuthenticationChallengeRequest>,
    ) -> Result<Response<AuthenticationChallengeResponse>, Status> {
        todo!();
    }

    async fn verify_authentication(
        &self,
        request: Request<AuthenticationAnswerRequest>,
    ) -> Result<Response<AuthenticationAnswerResponse>, Status> {
        todo!();
    }
}

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:50051".to_string();

    println!("Building the server at {}", addr);

    let auth_impl = AuthImpl::default();

    Server::builder()
        .add_service(AuthServer::new(auth_impl))
        .serve(addr.parse().expect("could not parse address"))
        .await
        .unwrap();
}
