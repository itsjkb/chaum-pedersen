use nanoid::nanoid;
use nmc_solution::ChaumPedersen;
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
        println!("RegisterRequest -> {:?}", request);

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
        println!("AuthenticationChallengeRequest -> {:?}", request);

        let request = request.into_inner();
        let username = request.user;

        println!(
            "Processing Auth::create_authentication_challenge() for {}",
            username
        );
        let user_info_hashmap = &mut self.user_info.lock().unwrap();

        if let Some(user_info) = user_info_hashmap.get_mut(&username) {
            user_info.r1 = BigUint::from_bytes_be(&request.r1);
            user_info.r2 = BigUint::from_bytes_be(&request.r2);

            let (_, _, _, q) = ChaumPedersen::get_constants();
            let c = ChaumPedersen::generate_random_below(&q);
            let auth_id = nanoid!();

            let auth_id_to_user = &mut self.auth_id_to_user.lock().unwrap();
            auth_id_to_user.insert(auth_id.clone(), username.clone());

            Ok(Response::new(AuthenticationChallengeResponse {
                auth_id,
                c: c.to_bytes_be(),
            }))
        } else {
            Err(Status::new(
                Code::NotFound,
                format!("User: {} not found!", username),
            ))
        }
    }

    async fn verify_authentication(
        &self,
        request: Request<AuthenticationAnswerRequest>,
    ) -> Result<Response<AuthenticationAnswerResponse>, Status> {
        println!("AuthenticationAnswerRequest -> {:?}", request);

        let request = request.into_inner();
        let auth_id = request.auth_id;

        println!(
            "Processing Auth::create_authentication_challenge() for {}",
            auth_id
        );

        let mut auth_id_to_user_hashmap = &mut self.auth_id_to_user.lock().unwrap();

        if let Some(username) = auth_id_to_user_hashmap.get(&auth_id) {
            let user_info_hashmap = &mut self.user_info.lock().unwrap();
            let user_info = user_info_hashmap
                .get_mut(username)
                .expect("AuthId not found on hashmap");

            let s = BigUint::from_bytes_be(&request.s);
            user_info.solution = s;

            let (alpha, beta, p, q) = ChaumPedersen::get_constants();
            let cp = ChaumPedersen { alpha, beta, p, q };

            let verification = cp.verify(
                &user_info.r1,
                &user_info.r2,
                &user_info.y1,
                &user_info.y2,
                &user_info.challenge,
                &user_info.solution,
            );

            if verification {
                let session_id = nanoid!();

                println!("✅ Correct Challenge Solution for username: {:?}", username);

                Ok(Response::new(AuthenticationAnswerResponse { session_id }))
            } else {
                println!("❌ Wrong Challenge Solution for username: {:?}", username);

                Err(Status::new(
                    Code::PermissionDenied,
                    format!("AuthId: {} bad solution to the challenge", auth_id),
                ))
            }
        } else {
            Err(Status::new(
                Code::NotFound,
                format!("AuthId: {} not found in database", auth_id),
            ))
        }
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
