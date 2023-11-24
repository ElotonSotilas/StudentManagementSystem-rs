use actix_web::{App, HttpServer};
use backend::rest_api::*;

mod backend;

extern crate actix_web;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let http_server = HttpServer::new(|| {
        App::new()
            .service(index)
            .service(users)
            .service(students)
            .service(teachers)
            .service(courses)
            .service(admin)
            .service(enroll)
            .service(unenroll)
            .service(login)
            .service(logout)
            .service(register)
    })
    .bind(("127.0.0.1", 8080))?;

    http_server.run().await
}
