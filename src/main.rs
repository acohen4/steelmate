mod lib;

use actix_web::{web::Data, web::Path, App, HttpResponse, HttpServer, Responder, get, post};
use actix_http::Response;
use lib::controller::GameController;
use lib::errors::GameError;
use lib::game_repository::GameRepository;

// LEFT TO DO:
// - User Management
// - Integrate with DB
// - Infra: Docker, Hosting, SSL
// - UI
// - AI

// start web server
// checkout https://github.com/actix/actix-web ; https://actix.rs/

// UI:
// https://medium.com/better-programming/how-to-build-a-chess-board-with-javascript-480ab182739e
// https://medium.com/better-programming/create-the-match-match-memory-game-in-react-and-vue-js-1026f1df000e
// https://medium.com/better-programming/vue-js-basics-inputs-events-and-components-1a874528e66a


struct AppState {
    game_controller: GameController
}

#[post("/game")]
async fn start_game(data: Data<AppState>) -> impl Responder {
    match data.game_controller.start_game() {
        Ok(id) => HttpResponse::Ok().body(id.to_string()),
        Err(msg) =>  HttpResponse::InternalServerError().body(msg),
    }
}

#[get("/game/{id}")]
async fn get_game(Path(id): Path<u32>, data: Data<AppState>) -> impl Responder {
    println!("{}", String::from("Come on man!"));
    match data.game_controller.get_game(id) {
        Ok(board_json) => HttpResponse::Ok().content_type("application/json").body(board_json),
        Err(error) => process_game_error(error),
    }
}

#[post("/game/{id}/position/{pos}/move/{dest}")]
async fn post_game_move(Path((id, pos, dest)): Path<(u32, String, String)>,
                        data: Data<AppState>) -> impl Responder {
    match data.game_controller.play_move(id, pos, dest) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(error) => process_game_error(error),
    }
}

#[get("/game/{id}/position/{pos}/options")]
async fn get_piece_options(Path((id, pos)): Path<(u32, String)>,
                           data: Data<AppState>) -> impl Responder {

    match data.game_controller.get_piece_move_options(id, &pos) {
        Ok(moves) =>  HttpResponse::Ok().content_type("application/json").body(moves),
        Err(error) => process_game_error(error),
    }
}

#[get("/game/{id}/color/{c}/best_moves")]
async fn get_best_move() -> impl Responder {
    "Best move"
}

fn process_game_error(error: GameError) -> Response {
    match error {
        GameError::DoesNotExist => HttpResponse::NotFound().finish(),
        GameError::Internal(msg) => HttpResponse::InternalServerError().body(msg),
        GameError::NotAllowed => HttpResponse::BadRequest().finish(),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    // initialize game bank
    let app_state = Data::new(AppState {
        game_controller: GameController::new(GameRepository::new())
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(start_game)
            .service(get_game)
            .service(post_game_move)
            .service(get_piece_options)
            .service(get_best_move)
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

