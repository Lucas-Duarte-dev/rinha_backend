use std::{collections::HashMap, sync::Arc, env, net::SocketAddr};

use axum::{
    routing::{get, post},
    Router,
    response::IntoResponse,
    http::StatusCode,
    extract::{State, Path},
    Json,
};
use serde::{Serialize, Deserialize};
use time::{Date, macros::date};
use tokio::sync::RwLock;
use uuid::Uuid;


time::serde::format_description!(date_format, Date, "[year]-[month]-[day]"); 

#[derive(Clone, Serialize)]
pub struct Person {
    pub id: Uuid,
    #[serde(rename = "nome")]
    pub name: String,
    #[serde(rename = "apelido")]
    pub nick: String,
    #[serde(rename = "nascimento", with = "date_format")]
    pub birth_date: Date,
    pub stack: Option<Vec<String>>,
}

#[derive(Clone, Deserialize)]
pub struct NewPerson {
    #[serde(rename = "nome")]
    pub name: String,
    #[serde(rename = "apelido")]
    pub nick: String,
    #[serde(rename = "nascimento", with = "date_format")]
    pub birth_date: Date,
    pub stack: Option<Vec<String>>, 
}

type AppState = Arc<RwLock<HashMap<Uuid, Person>>>;

#[tokio::main]
async fn main() {
    let mut person_map: HashMap<Uuid, Person > = HashMap::new();

    let person = Person {
        id: Uuid::now_v7(),
        name: String::from("Lucas Duarte"),
        nick: String::from("Lucas_Duarte_dev"),
        birth_date: date!(2000 - 08 - 31),
        stack: Some(vec![String::from("NodeJs")])
    };
    println!("{}", person.id);

    person_map.insert(person.id, person);

    let app_state: AppState = Arc::new(RwLock::new(person_map));
    let port = env::var("PORT")
        .ok()
        .and_then(|port| port.parse::<u16>().ok())
        .unwrap_or(9999);

    let app = Router::new()
        .route("/pessoas", get(search_person))
        .route("/pessoas/:id", get(find_person))
        .route("/pessoas", post(create_person))
        .route("/contagem-pessoas", get(count_person ))
        .with_state(app_state) ;
 
    axum::Server::bind(&SocketAddr::from(([0, 0, 0, 0], port)))
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn search_person() -> impl IntoResponse {
    (StatusCode::OK, "Busca Pessoa por ID")
}

async fn find_person(
    State(_person): State<AppState>,
    Path(person_id): Path<Uuid>
) -> impl IntoResponse {
    let thread_person = _person.read().await;

    match thread_person.get(&person_id) {
        Some(person) => Ok(Json(person.clone())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn create_person(
    State(_person): State<AppState>,
    Json(_new_person): Json<NewPerson>
) -> impl IntoResponse {
    if _new_person.name.len() > 100 || _new_person.nick.len() > 32 {
        return Err(StatusCode::UNPROCESSABLE_ENTITY)
    }

    if let Some(ref steck) = _new_person.steck {
        if stack.iter().any(|tech| tech.len() > 32) {
            return Err(StatusCode::UNPROCESSABLE_ENTITY);
        }
    }

    let id: Uuid = Uuid::now_v7();
    let person = Person {
        id,
        name: _new_person.name,
        nick: _new_person.nick,
        birth_date: _new_person.birth_date,
        stack: _new_person.stack
    };

    _person.write().await.insert(id, person.clone());
    return Ok((StatusCode::OK, Json(person)));
}

async fn count_person(State(_person): State<AppState>) -> impl IntoResponse {
    let count = _person.read ().await.len();
    (StatusCode::OK, Json(count))
}