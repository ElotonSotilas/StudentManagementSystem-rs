use actix_cors::Cors;
use actix_web::{App, HttpServer};
use backend::rest_api::*;
use rustls::server::ServerConfig;
use rustls::{Certificate, PrivateKey};
use std::fs::File;
use std::io::{BufRead, BufReader};

mod backend;

extern crate actix_web;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(
            vec![Certificate(
                BufReader::new(
                    File::open("/usr/local/etc/letsencrypt/live/be.duyenle.com/cert.pem").unwrap(),
                )
                .lines()
                .map(|l| l.unwrap())
                .collect::<String>()
                .into_bytes(),
            )],
            PrivateKey(
                BufReader::new(
                    File::open("/usr/local/etc/letsencrypt/live/be.duyenle.com/privkey.pem")
                        .unwrap(),
                )
                .lines()
                .map(|l| l.unwrap())
                .collect::<String>()
                .into_bytes(),
            ),
        )
        .unwrap();

    let http_server = HttpServer::new(|| {
        App::new()
            .wrap(Cors::permissive())
            .service(index)
            .service(get_users)
            .service(get_students)
            .service(get_teachers)
            .service(get_departments)
            .service(get_department)
            .service(new_department)
            .service(invite_to_department)
            .service(kick_from_department)
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
    .bind_rustls(("0.0.0.0", 8080), config)?;

    http_server.run().await
}
