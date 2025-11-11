use log::{LevelFilter, info};
use rtfw_http::{file_server::FileServer, http::HttpMethod, router::Router, web_server::WebServer};

use crate::database::{
    picture_meta_repository::PictureMetaRepository, user_repository::UserRepository,
};
use config::Config;

mod config;
mod controllers;
mod database;
mod http_helpers;
mod models;
mod oauth2;
mod routes;
mod security;
mod utils;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::new()
        .filter_module("advent_of_time", LevelFilter::Trace)
        .filter_module("rtfw_http", LevelFilter::Warn)
        .init();

    let config = Config::get()?;

    UserRepository::initialize_database()?;
    PictureMetaRepository::initialize_database()?;

    let file_server = FileServer::new()
        .map_file("/favicon.ico", "src/assets/favicon.ico")?
        .map_file("/main.css", "src/styles/main.css")?
        .map_file("/about.css", "src/styles/about.css")?
        .map_file("/day.css", "src/styles/day.css")?
        .map_file("/leaderboard.css", "src/styles/leaderboard.css")?
        .map_file("/profile.css", "src/styles/profile.css")?
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
            "/auth/oauth2-redirect/discord",
            controllers::auth::get_discord_oauth2_redirect,
        )?
        .get(
            "/auth/oauth2-redirect/microsoft",
            controllers::auth::get_microsoft_oauth2_redirect,
        )?
        .get(
            "/auth/oauth2-redirect/github",
            controllers::auth::get_github_oauth2_redirect,
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
    info!("server listening on: {}", config.hostname);

    let server = WebServer::new(&config.hostname, router)?;
    server.run()
}
