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
            .service(get_course)
            .service(new_course)
            .service(update_course)
            .service(remove_course)
            .service(update_user)
            .service(delete_user)
            .service(update_self)
            .service(delete_self)
            .service(admin)
            .service(enroll)
            .service(unenroll)
            .service(login)
            .service(logout)
            .service(register)
            .service(register_admin)
    })
    .bind(("127.0.0.1", 8080))?;

    http_server.run().await
}
