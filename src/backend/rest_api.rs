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
                Err(_) => HttpResponse::InternalServerError()
                    .json(json!({"error": "Failed to serialize users"})),
            }
        }
        Err(_) => HttpResponse::InternalServerError().json(json!({"error": "Failed to get users"})),
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
                Err(_) => HttpResponse::InternalServerError()
                    .json(json!({"error": "Failed to serialize students"})),
            }
        }
        Err(_) => {
            HttpResponse::InternalServerError().json(json!({"error": "Failed to get students"}))
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
                Err(_) => HttpResponse::InternalServerError()
                    .json(json!({"error": "Failed to serialize teachers"})),
            }
        }
        Err(_) => {
            HttpResponse::InternalServerError().json(json!({"error": "Failed to get teachers"}))
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
                Err(_) => HttpResponse::InternalServerError()
                    .json(json!({"error": "Failed to serialize courses"})),
            }
        }
        Err(_) => {
            HttpResponse::InternalServerError().json(json!({"error": "Failed to get courses"}))
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
                            Err(_) => {
                                return HttpResponse::InternalServerError()
                                    .json(json!({"error": "Failed to serialize user"}))
                            }
                        }
                    }
                    Err(_) => {
                        return HttpResponse::InternalServerError()
                            .json(json!({"error": "Failed to login"}))
                    }
                }
            }
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({"error": "Failed to get user"}))
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

                            Err(_) => {
                                return HttpResponse::InternalServerError()
                                    .json(json!({"error": "Failed to enroll"}))
                            }
                        }
                    }
                    Err(_) => {
                        return HttpResponse::InternalServerError()
                            .json(json!({"error": "Failed to login"}))
                    }
                }
            }
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({"error": "Failed to get user"}))
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

                            Err(_) => {
                                return HttpResponse::InternalServerError()
                                    .json(json!({"error": "Failed to unenroll"}))
                            }
                        }
                    }
                    Err(_) => {
                        return HttpResponse::InternalServerError()
                            .json(json!({"error": "Failed to login"}))
                    }
                }
            }
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({"error": "Failed to get user"}))
        }
    }
}

#[get("/login")]
pub async fn login(req: actix_web::HttpRequest) -> impl Responder {
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

                match conn.login(username.to_owned(), password.to_owned()) {
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
                    Err(_) => {
                        return HttpResponse::InternalServerError()
                            .json(json!({"error": "Failed to login"}))
                    }
                }
            }
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({"error": "Failed to get user"}))
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

// MUST REFACTOR USER STRUCT AND ENDPOINT: USERNAME IS NOW THE ACTUAL NAME OF THE USER,
// WHILE USERNAME PASSED HERE IS JUST PART OF THE EMAIL BEFORE @aubg.edu
#[get("/register")]
pub async fn register(req: actix_web::HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();
    let request_headers = req.headers();

    let username = request_headers.get("username");
    let password = request_headers.get("password");
    let email = request_headers.get("email");
    let phone = request_headers.get("phone");

    if username.is_none() || password.is_none() || email.is_none() || phone.is_none() {
        return HttpResponse::BadRequest()
            .json(json!({"error": "Missing username, password, email, phone, or role"}));
    }

    let username = username.unwrap().to_str().unwrap().to_owned();
    let password = password.unwrap().to_str().unwrap().to_owned();
    let email = email.unwrap().to_str().unwrap().to_owned();
    let phone = phone.unwrap().to_str().unwrap().to_owned();

    let u = User {
        id: 0,
        username,
        password,
        email,
        phone,
        created_at: String::new(),
        updated_at: String::new(),
        verified: false,
        suspended: false,
        forcenewpw: false,
        role: String::from("student"),
    };

    match conn.register_user(u) {
        Ok(_) => HttpResponse::Ok().json(json!({"message": "Successfully registered."})),
        Err(_) => HttpResponse::InternalServerError().json(json!({"error": "Failed to register"})),
    }
}
