#[macro_use]
extern crate lazy_static;

pub mod errors;
mod eth_api;
mod helpers;
pub mod validators;

use actix_example_service::{
    sea_orm::{Database, DatabaseConnection},
    Mutation, Query,
};
use actix_files::Files as Fs;
use actix_web::{
    error, get, middleware, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Result,
};

use crate::errors::ServerError;
use eth_api::*;
use helpers::*;
use listenfd::ListenFd;
use migration::{Migrator, MigratorTrait};
use serde::{Deserialize, Serialize};
use std::env;
use tera::Tera;
use validators::*;

const DEFAULT_TRANSACTIONS_PER_PAGE: u64 = 10;

#[derive(Debug, Clone)]
struct AppState {
    templates: tera::Tera,
    conn: DatabaseConnection,
}

#[derive(Debug, Deserialize)]
pub struct Params {
    page: Option<u64>,
    transactions_per_page: Option<u64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TransactionFormInput {
    pub address: String,
    pub starting_block_number: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct FlashData {
    kind: String,
    message: String,
}

#[get("/")]
async fn load_transactions_data(data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let template = &data.templates;
    let ctx = tera::Context::new();
    let body = template
        .render("new.html.tera", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

#[get("/list/{address}/{starting_block_number}")]
async fn list(
    req: HttpRequest,
    data: web::Data<AppState>,
    address_and_starting_block: web::Path<(String, u64)>,
) -> Result<HttpResponse, Error> {
    let template = &data.templates;
    let conn = &data.conn;

    // get params
    let params = web::Query::<Params>::from_query(req.query_string()).unwrap();

    let (address, starting_block_number) = address_and_starting_block.into_inner();

    let page = params.page.unwrap_or(1);
    let transactions_per_page = params
        .transactions_per_page
        .unwrap_or(DEFAULT_TRANSACTIONS_PER_PAGE);

    let (transactions, num_pages) = Query::find_transactions_in_page(
        conn,
        address.clone(),
        starting_block_number,
        page,
        transactions_per_page,
    )
    .await
    .map_err(ServerError::from)?;

    let total_transactions_count = Query::get_transactions_count_since_block_for_selected_address(
        conn,
        starting_block_number,
        address.clone(),
    )
    .await
    .map_err(ServerError::from)?;

    let mut ctx = tera::Context::new();

    ctx.insert("address", &address);
    ctx.insert("starting_block_number", &starting_block_number);
    ctx.insert("transactions", &transactions);
    ctx.insert("total_transactions_count", &total_transactions_count);
    ctx.insert("page", &page);
    ctx.insert("transactions_per_page", &transactions_per_page);
    ctx.insert("num_pages", &num_pages);

    let body = template
        .render("index.html.tera", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

#[post("/")]
async fn create(
    data: web::Data<AppState>,
    transaction_form: web::Form<TransactionFormInput>,
) -> Result<HttpResponse, Error> {
    let conn = &data.conn;

    let form = transaction_form.into_inner();

    ensure_valid_eth_address(&form.address)?;

    let current_block_number = get_current_block_number().await?;

    ensure_valid_starting_block_number(form.starting_block_number, current_block_number)?;

    let fetched_block_numbers_since_block =
        Query::get_block_numbers_since_block_for_selected_address(
            conn,
            &form.address,
            form.starting_block_number,
        )
        .await
        .map_err(ServerError::from)?;

    let block_ranges_for_unfetched_transactions = get_block_ranges_for_unfetched_transactions(
        fetched_block_numbers_since_block,
        form.starting_block_number,
        current_block_number,
    );

    let mut unfetched_transactions = vec![];

    for block_range_for_unfetched_transactions in block_ranges_for_unfetched_transactions {
        let transactions =
            fetch_transactions(block_range_for_unfetched_transactions, &form).await?;
        unfetched_transactions.extend(transactions);
    }

    if !unfetched_transactions.is_empty() {
        Mutation::save_transactions(conn, unfetched_transactions)
            .await
            .map_err(ServerError::from)?;
    }

    Ok(HttpResponse::Found()
        .append_header((
            "location",
            "/list/".to_string() + &form.address + "/" + &form.starting_block_number.to_string(),
        ))
        .finish())
}

async fn not_found(data: web::Data<AppState>, request: HttpRequest) -> Result<HttpResponse, Error> {
    let mut ctx = tera::Context::new();
    ctx.insert("uri", request.uri().path());

    let template = &data.templates;
    let body = template
        .render("error/404.html.tera", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

#[actix_web::main]
async fn start() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();

    // get env vars
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let server_url = format!("{host}:{port}");

    // establish connection to database and apply migrations
    // -> create post table if not exists
    let conn = Database::connect(&db_url).await.unwrap();
    Migrator::up(&conn, None).await.unwrap();

    // load tera templates and build app state
    let templates = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
    let state = AppState { templates, conn };

    // create server and try to serve over socket if possible
    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(move || {
        App::new()
            .service(Fs::new("/static", "./api/static"))
            .app_data(web::Data::new(state.clone()))
            .wrap(middleware::Logger::default()) // enable logger
            .default_service(web::route().to(not_found))
            .configure(init)
    });

    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => server.bind(&server_url)?,
    };

    println!("Starting server at {server_url}");
    server.run().await?;

    Ok(())
}

fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(load_transactions_data);
    cfg.service(list);
    cfg.service(create);
}

pub fn main() {
    let result = start();

    if let Some(err) = result.err() {
        println!("Error: {err}");
    }
}
