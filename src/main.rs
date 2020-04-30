mod error;
mod schema;
use schema::{Context, Query, Schema};

#[macro_use]
extern crate juniper;
use juniper::EmptyMutation;

#[macro_use]
extern crate dgraph;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate log;

use std::io;
use std::sync::Arc;

use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;

const PROTOCOL: &str = "http";
const HOSTNAME: &str = "127.0.0.1";
const PORT: u32 = 4000;

const DGRAPH_HOSTNAME: &str = "127.0.0.1";
const DGRAPH_PORT: u32 = 9080;

async fn graphiql() -> HttpResponse {
    let html = graphiql_source(&format!("{}://{}:{}/graphql", PROTOCOL, HOSTNAME, PORT));
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

async fn graphql(
    st: web::Data<Arc<Schema>>,
    context: web::Data<Arc<Context>>,
    data: web::Json<GraphQLRequest>,
) -> Result<HttpResponse, Error> {
    let user = web::block(move || {
        let res = data.execute(&st, &context);
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

    //Create Dgraph client
    let context = std::sync::Arc::new(Context {
        dgraph_client: make_dgraph!(dgraph::new_dgraph_client(&format!(
            "{}:{}",
            DGRAPH_HOSTNAME, DGRAPH_PORT
        ))),
    });

    // Start http server
    HttpServer::new(move || {
        App::new()
            .data(schema.clone())
            .data(context.clone())
            .wrap(middleware::Logger::default())
            .service(web::resource("/graphql").route(web::post().to(graphql)))
            .service(web::resource("/graphiql").route(web::get().to(graphiql)))
    })
    .bind(format!("{}:{}", HOSTNAME, PORT))?
    .run()
    .await
}
