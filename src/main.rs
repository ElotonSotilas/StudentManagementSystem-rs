use actix_web::{App, HttpServer};
use actix_cors::Cors;
use backend::rest_api::*;

mod backend;

extern crate actix_web;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let http_server = HttpServer::new(|| {
        App::new().wrap(Cors::permissive())
            .service(index)
            .service(get_users)
            .service(get_students)
            .service(get_teachers)
            .service(get_courses)
            .service(get_course)
            .service(new_course)
            .service(update_course)
            .service(remove_course)
            .service(update_user)
            .service(delete_user)
            .service(get_self)
            .service(update_self)
            .service(admin)
            .service(enroll)
            .service(unenroll)
            .service(login)
            .service(logout)
            .service(register)
            .service(register_admin)
    })
    .bind(("0.0.0.0", 8080))?;

    http_server.run().await
}
