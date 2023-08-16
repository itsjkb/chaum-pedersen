pub mod nillion {
    include!("./nillion.rs");
}

use nmc_solution::ChaumPedersen;
use num_bigint::BigUint;
use std::io::stdin;
use std::{collections::HashMap, sync::Mutex};
use tonic::{transport::Server, Code, Request, Response, Status};

use nillion::{
    auth_client::AuthClient, AuthenticationAnswerRequest, AuthenticationAnswerResponse,
    AuthenticationChallengeRequest, AuthenticationChallengeResponse, RegisterRequest,
    RegisterResponse,
};

#[tokio::main]
async fn main() {
    let mut buf = String::new();

    let mut client = AuthClient::connect("http://127.0.0.1:50051")
        .await
        .expect("Unable to connect to server");

    println!("✅ Connected to server.");

    println!("Please provide username:");
    stdin()
        .read_line(&mut buf)
        .expect("Unable to read username from user input");
    let username = buf.trim().to_string();
    buf.clear();

    println!("Please provide password:");
    stdin()
        .read_line(&mut buf)
        .expect("Unable to read username from user input");
    let password = BigUint::from_bytes_be(buf.trim().as_bytes());
    buf.clear();

    let (alpha, beta, p, q) = ChaumPedersen::get_constants();
    let cp = ChaumPedersen {
        alpha: alpha.clone(),
        beta: beta.clone(),
        p: p.clone(),
        q: q.clone(),
    };

    let (y1, y2) = cp.compute_pair(&password);

    let request = RegisterRequest {
        user: username.clone(),
        y1: y1.to_bytes_be(),
        y2: y2.to_bytes_be(),
    };

    let _response = client
        .register(request)
        .await
        .expect("Unable to register user.");

    println!("✅ Registration was successful.");
    println!("Server Response for Register -> {:?}", _response);

    println!("Please provide the password (to login):");
    stdin()
        .read_line(&mut buf)
        .expect("Could not get the username from stdin");
    let password = BigUint::from_bytes_be(buf.trim().as_bytes());
    buf.clear();

    let k = ChaumPedersen::generate_random_below(&q);
    let (r1, r2) = cp.compute_pair(&password);

    let request = AuthenticationChallengeRequest {
        user: username.clone(),
        r1: r1.to_bytes_be(),
        r2: r2.to_bytes_be(),
    };

    let response = client
        .create_authentication_challenge(request)
        .await
        .expect("Unable to get authentication challenge.")
        .into_inner();
    println!(
        "Server Response for Authentication Challenge -> {:?}",
        _response
    );

    let auth_id = response.auth_id;
    let challenge = BigUint::from_bytes_be(&response.c);
    let s = cp.solve(&k, &challenge, &password);

    let request = AuthenticationAnswerRequest {
        auth_id: auth_id,
        s: s.to_bytes_be(),
    };

    let response = client
        .verify_authentication(request)
        .await
        .expect("Unable to verify authentication.")
        .into_inner();

    println!("✅Logging successful! session_id: {}", response.session_id);
    println!(
        "Server Response for Verify Authentication -> {:?}",
        response
    );
}
