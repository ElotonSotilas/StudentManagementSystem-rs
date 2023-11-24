use actix_web::{get, HttpResponse, Responder};
use serde_json::json;

use crate::backend::table_models::User;

use super::{
    filter::{Filter, UsersFilter},
    server_connection_impl::*,
    table_models::Course,
};

#[get("/")]
pub async fn index() -> impl Responder {
    HttpResponse::Ok().json(json!({"success": true}))
}

#[get("/users")]
pub async fn users() -> impl Responder {
    let conn = ServerConnection::new();
    let users = conn.get_users();
    match users {
        Ok(u) => {
            let json = serde_json::to_string(&u);
            match json {
                Ok(j) => HttpResponse::Ok().body(j),
                Err(e) => HttpResponse::InternalServerError()
                    .json(json!({"error": e.to_string()})),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[get("/students")]
pub async fn students() -> impl Responder {
    let conn = ServerConnection::new();
    let students = conn.get_users_by_filters(vec![Filter::Users(UsersFilter::Role(
        "student".to_string(),
    ))]);
    match students {
        Ok(s) => {
            let json = serde_json::to_string(&s);
            match json {
                Ok(j) => HttpResponse::Ok().body(j),
                Err(e) => HttpResponse::InternalServerError()
                    .json(json!({"error": e.to_string()})),
            }
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(json!({"error": e.to_string()}))
        }
    }
}

#[get("/teachers")]
pub async fn teachers() -> impl Responder {
    let conn = ServerConnection::new();
    let teachers = conn.get_users_by_filters(vec![Filter::Users(UsersFilter::Role(
        "teacher".to_string(),
    ))]);
    match teachers {
        Ok(t) => {
            let json = serde_json::to_string(&t);
            match json {
                Ok(j) => HttpResponse::Ok().body(j),
                Err(e) => HttpResponse::InternalServerError()
                    .json(json!({"error": e.to_string()})),
            }
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(json!({"error": e.to_string()}))
        }
    }
}

#[get("/courses")]
pub async fn courses() -> impl Responder {
    let conn = ServerConnection::new();
    let courses = conn.search_courses("".to_string());
    match courses {
        Ok(c) => {
            let json = serde_json::to_string(&c);
            match json {
                Ok(j) => HttpResponse::Ok().body(j),
                Err(e) => HttpResponse::InternalServerError()
                    .json(json!({"error": e.to_string()})),
            }
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(json!({"error": e.to_string()}))
        }
    }
}

#[get("/admin")]
pub async fn admin(req: actix_web::HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();
    let request_headers = req.headers();

    let username = request_headers.get("username");
    let password = request_headers.get("password");

    if username.is_none() || password.is_none() {
        return HttpResponse::BadRequest().json(json!({"error": "Missing username or password"}));
    }

    let username = username.unwrap().to_str().unwrap();
    let password = password.unwrap().to_str().unwrap();

    let user = conn.search_users(format!("{}", username));

    match user {
        Ok(u) => {
            if u.len() == 0 {
                return HttpResponse::BadRequest().json(json!({"error": "User not found"}));
            } else {
                let user = u.get(0).unwrap();

                if user.role != "admin" {
                    return HttpResponse::BadRequest()
                        .json(json!({"error": "User is not an admin"}));
                }

                match conn.login(username.to_owned(), password.to_owned()) {
                    Ok(_) => {
                        let json = serde_json::to_string(&user);
                        match json {
                            Ok(j) => return HttpResponse::Ok().body(j),
                            Err(e) => {
                                return HttpResponse::InternalServerError()
                                    .json(json!({"error": e.to_string()}))
                            }
                        }
                    }
                    Err(e) => {
                        return HttpResponse::InternalServerError()
                            .json(json!({"error": e.to_string()}))
                    }
                }
            }
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({"error": e.to_string()}))
        }
    }
}

#[get("/enroll/{id}")]
pub async fn enroll(req: actix_web::HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();
    let request_headers = req.headers();

    let email = request_headers.get("email");
    let password = request_headers.get("password");
    let course_id = req.match_info().get("id");

    if email.is_none() || password.is_none() {
        return HttpResponse::BadRequest().json(json!({"error": "Missing email or password"}));
    }

    if course_id.is_none() {
        return HttpResponse::BadRequest().json(json!({"error": "Missing course id"}));
    }

    let username = email
        .unwrap()
        .to_str()
        .unwrap()
        .split("@")
        .take(1)
        .collect::<String>();
    let password = password.unwrap().to_str().unwrap().to_owned();
    let course_id = course_id.unwrap().to_owned();

    let user = conn.search_users(format!("{}", username));

    match user {
        Ok(u) => {
            if u.len() == 0 {
                return HttpResponse::BadRequest().json(json!({"error": "User not found"}));
            } else {
                let user = u.get(0).unwrap();

                match conn.login(username, password) {
                    Ok(_) => {
                        match conn.enroll_courses(
                            conn.search_courses(format!("{}", course_id))
                                .unwrap()
                                .iter()
                                .filter_map(|c| Some(c.0.clone()))
                                .collect(),
                        ) {
                            Ok(_) => {
                                let json = serde_json::to_string(&user);
                                match json {
                                    Ok(j) => return HttpResponse::Ok().body(j),
                                    Err(_) => {
                                        return HttpResponse::InternalServerError()
                                            .json(json!({"error": "Failed to serialize user"}))
                                    }
                                }
                            }

                            Err(e) => {
                                return HttpResponse::InternalServerError()
                                    .json(json!({"error": e.to_string()}))
                            }
                        }
                    }
                    Err(e) => {
                        return HttpResponse::InternalServerError()
                            .json(json!({"error": e.to_string()}))
                    }
                }
            }
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({"error": e.to_string()}))
        }
    }
}

#[get("/unenroll/{id}")]
pub async fn unenroll(req: actix_web::HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();
    let request_headers = req.headers();

    let email = request_headers.get("email");
    let password = request_headers.get("password");
    let course_id = req.match_info().get("id");

    if email.is_none() || password.is_none() {
        return HttpResponse::BadRequest().json(json!({"error": "Missing email or password"}));
    }

    if course_id.is_none() {
        return HttpResponse::BadRequest().json(json!({"error": "Missing course id"}));
    }

    let username = email
        .unwrap()
        .to_str()
        .unwrap()
        .split("@")
        .take(1)
        .collect::<String>();
    let password = password.unwrap().to_str().unwrap().to_owned();
    let course_id = course_id.unwrap().to_owned();

    let user = conn.search_users(format!("{}", username));

    match user {
        Ok(u) => {
            if u.len() == 0 {
                return HttpResponse::BadRequest().json(json!({"error": "User not found"}));
            } else {
                let user = u.get(0).unwrap();

                match conn.login(username, password) {
                    Ok(_) => {
                        let course_list = conn
                            .search_courses(format!("{}", course_id))
                            .unwrap()
                            .iter()
                            .filter_map(|c| Some(c.0.clone()))
                            .collect::<Vec<Course>>();

                        match conn.drop_courses(course_list) {
                            Ok(_) => {
                                let json = serde_json::to_string(&user);
                                match json {
                                    Ok(j) => return HttpResponse::Ok().body(j),
                                    Err(_) => {
                                        return HttpResponse::InternalServerError()
                                            .json(json!({"error": "Failed to serialize user"}))
                                    }
                                }
                            }

                            Err(e) => {
                                return HttpResponse::InternalServerError()
                                    .json(json!({"error": e.to_string()}))
                            }
                        }
                    }
                    Err(e) => {
                        return HttpResponse::InternalServerError()
                            .json(json!({"error": e.to_string()}))
                    }
                }
            }
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({"error": e.to_string()}))
        }
    }
}

#[get("/login")]
pub async fn login(req: actix_web::HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();
    let request_headers = req.headers();

    let email = request_headers.get("email");
    let password = request_headers.get("password");

    if email.is_none() || password.is_none() {
        return HttpResponse::BadRequest().json(json!({"error": "Missing username or password"}));
    }

    let email = email.unwrap().to_str().unwrap();
    let password = password.unwrap().to_str().unwrap();

    let user = conn.search_users(format!("{}", email));

    match user {
        Ok(u) => {
            if u.len() == 0 {
                return HttpResponse::BadRequest().json(json!({"error": "User not found"}));
            } else {
                let user = u.get(0).unwrap();

                match conn.login(email.to_owned(), password.to_owned()) {
                    Ok(_) => {
                        let json = serde_json::to_string(&user);
                        match json {
                            Ok(j) => return HttpResponse::Ok().body(j),
                            Err(e) => {
                                return HttpResponse::InternalServerError()
                                    .json(json!({"error": e.to_string()}))
                            }
                        }
                    }
                    Err(e) => {
                        return HttpResponse::InternalServerError()
                            .json(json!({"error": e.to_string()}))
                    }
                }
            }
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({"error": e.to_string()}))
        }
    }
}

#[get("/logout")]
pub async fn logout(req: actix_web::HttpRequest) -> impl Responder {
    let request_headers = req.headers();

    let username = request_headers.get("username");
    let password = request_headers.get("password");

    if username.is_none() || password.is_none() {
        HttpResponse::Ok().json(json!({"message": "Successfully logged out."}))
    } else {
        HttpResponse::InternalServerError().json(json!({"error": "Failed to logout."}))
    }
}

#[get("/register")]
pub async fn register(req: actix_web::HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();
    let request_headers = req.headers();

    let username = request_headers.get("username");
    let password = request_headers.get("password");
    let email = request_headers.get("email");
    let phone = request_headers.get("phone");

    if username.is_none() || password.is_none() || email.is_none() {
        return HttpResponse::BadRequest()
            .json(json!({"error": "Missing username, password, email, or role"}));
    }

    let username = username.unwrap().to_str().unwrap().to_owned();
    let password = password.unwrap().to_str().unwrap().to_owned();
    let email = email.unwrap().to_str().unwrap().to_owned();
    let phone = phone
        .is_some_and(|_| true)
        .then(|| phone.unwrap().to_str().unwrap().to_owned())
        .or_else(|| Some(String::from("")))
        .unwrap_or_default();

    let u = User {
        id: 0,
        username,
        password,
        email,
        phone,
        verified: false,
        suspended: false,
        forcenewpw: false,
        role: String::from("student"),
    };

    match conn.register_user(u) {
        Ok(_) => HttpResponse::Ok().json(json!({"message": "Successfully registered."})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}
