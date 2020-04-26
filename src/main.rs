mod error;
mod schema;
use schema::{Query, Schema};

#[macro_use]
extern crate juniper;
use juniper::EmptyMutation;

#[macro_use]
extern crate dgraph;
extern crate serde_json;
#[macro_use]
extern crate log;

use std::io;
use std::sync::Arc;

use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;

async fn graphiql() -> HttpResponse {
    let html = graphiql_source("http://127.0.0.1:4000/graphql");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

async fn graphql(
    st: web::Data<Arc<Schema>>,
    data: web::Json<GraphQLRequest>,
) -> Result<HttpResponse, Error> {
    let user = web::block(move || {
        let res = data.execute(&st, &());
        Ok::<_, serde_json::error::Error>(serde_json::to_string(&res)?)
    })
    .await?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(user))
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info,haikubot_rs_api");
    env_logger::init();

    // Create Juniper schema
    let schema = std::sync::Arc::new(Schema::new(Query, EmptyMutation::new()));

    // Start http server
    HttpServer::new(move || {
        App::new()
            .data(schema.clone())
            .wrap(middleware::Logger::default())
            .service(web::resource("/graphql").route(web::post().to(graphql)))
            .service(web::resource("/graphiql").route(web::get().to(graphiql)))
    })
    .bind("127.0.0.1:4000")?
    .run()
    .await
}
