use log::{LevelFilter, info};
use rtfw_http::{file_server::FileServer, http::HttpMethod, router::Router, web_server::WebServer};

use config::Config;

use crate::database::initialize_database;

mod config;
mod controllers;
mod database;
mod http_helpers;
mod models;
mod routes;
mod security;
mod utils;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::new()
        .filter_module("advent_of_time", LevelFilter::Trace)
        .filter_module("rtfw_http", LevelFilter::Warn)
        .init();

    let config = Config::load_from_file()?;
    initialize_database()?;

    let file_server = FileServer::new()
        .map_file("/favicon.ico", "src/assets/favicon.ico")?
        .map_file("/main.css", "src/styles/main.css")?
        .map_file("/leaderboard.css", "src/styles/leaderboard.css")?
        .map_dir("/static", "src/assets/")?
        .map_dir("/scripts", "src/scripts/")?;

    let router = Router::new()
        // index
        .get("/", routes::get_index)?
        .get("/home", routes::get_index)?
        .get("/index", routes::get_index)?
        // about
        .get("/about", routes::get_about)?
        // auth
        .get("/auth/login", controllers::auth::get_login)?
        .get("/auth/logout", controllers::auth::get_logout)?
        .get("/auth/oauth2", controllers::auth::get_oauth2_login)?
        .get(
            "/auth/oauth2-redirect",
            controllers::auth::get_oauth2_redirect,
        )?
        .get("/auth/me", controllers::auth::get_me)?
        .get("/leaderboard", controllers::leaderboard::get_leaderboard)?
        // day
        .get("/day/:id", controllers::day::get_single_day)?
        .get("/day-pic/:id", controllers::day::get_day_picture)?
        // guess
        .post("/guess/:id", controllers::guess::post_guess)?
        // others
        .catch_all(HttpMethod::GET, routes::catcher_get_404)?
        .set_file_server(file_server);

    info!("ROUTER: {:#?}", router);
    let server = WebServer::new(&config.hostname, router)?;
    server.run()
}
