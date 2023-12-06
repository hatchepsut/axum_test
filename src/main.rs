use http::StatusCode;

use axum::{
    routing::{get, post},
    Json, Router, BoxError,
    error_handling::HandleErrorLayer,
    response::IntoResponse,
};

//redis
use fred::prelude::*;
use tower::ServiceBuilder;

use tower_sessions::{Session, RedisStore, SessionManagerLayer};

use tower_http::{
    services::ServeDir,
    trace::TraceLayer,
};

use serde::{Deserialize, Serialize};

const COUNTER_KEY: &str = "counter";

#[derive(Default, Deserialize, Serialize)]
struct Counter(usize);

#[tokio::main]
async fn main() {
    // initialize tracing
    //tracing_subscriber::fmt::init();

    let client = RedisClient::default();
    println!("Connecting to redis. {:?}", client);
    let _ = client.connect();
    let res = client.wait_for_connect().await;
    if let Err(e) = res {
        println!("Error connecting: {:?}", e);
        return();
    };
    println!("Connected to redis.");

    let session_store = RedisStore::new(client);

    let session_service = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|_: BoxError| async {
            StatusCode::BAD_REQUEST
        }))
        .layer(
            SessionManagerLayer::new(session_store)
                .with_secure(false)
                //.with_expiry(Expiry::OnInactivity(Duration::seconds(10))),
        );


    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/user", post(create_user))
        .nest_service("/assets", ServeDir::new("assets"))
        .layer(session_service)
        .layer(TraceLayer::new_for_http());

    // run our app with hyper, listening globally on port 3000
    // let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
        
}

// basic handler that responds with a static string
async fn root(session: Session) -> impl IntoResponse {
    let counter: Counter = session.get(COUNTER_KEY).unwrap().unwrap_or_default();
    session.insert(COUNTER_KEY, counter.0 + 1).unwrap();
    let str = format!("Current count: {}", counter.0);
    str
}

async fn create_user(Json(payload): Json<CreateUser>) -> (StatusCode, Json<User>) {

    //insert your application logic here
    let user = User {
        id: 1337,
        username: payload.username,
        utype: payload.utype,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(user))
}

// the input to our `create_user` handler
#[derive(Deserialize, Debug)]
struct CreateUser {
    username: String,
    utype: String,
}

// the output to our `create_user` handler
#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: u64,
    username: String,
    utype: String,
}
