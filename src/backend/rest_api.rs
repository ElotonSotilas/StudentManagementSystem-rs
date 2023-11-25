use actix_web::{delete, get, patch, post, HttpRequest, HttpResponse, Responder};
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
                Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
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
                Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
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
                Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
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
                Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[get("/courses/{id}")]
pub async fn get_course(req: HttpRequest) -> impl Responder {
    let conn = ServerConnection::new();
    let id = req.match_info().get("id").unwrap_or_else(|| "0");

    if id == "0" {
        return HttpResponse::BadRequest().json(json!({"error": "Missing course id."}));
    }

    let course_list = conn.search_courses(id.to_string());

    match course_list {
        Ok(c) => {
            let json = serde_json::to_string(&c);
            match json {
                Ok(j) => HttpResponse::Ok().body(j),
                Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[post("/courses")]
pub async fn new_course(req: HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();
    let request_headers = req.headers();

    let name = request_headers.get("name");
    let description = request_headers.get("description");
    let course_nr = request_headers.get("course_nr");
    let teacher_id = request_headers.get("id");
    let cr_cost = request_headers.get("cr_cost");
    let timeslots = request_headers.get("timeslots");

    if name.is_none()
        || teacher_id.is_none()
        || course_nr.is_none()
        || cr_cost.is_none()
        || timeslots.is_none()
    {
        return HttpResponse::BadRequest().json(json!({"error": "Missing required data."}));
    }

    let name = name.unwrap().to_str().unwrap();
    let description = description
        .unwrap()
        .to_str()
        .unwrap_or("No description.")
        .to_string();
    let course_nr = course_nr.unwrap().to_str().unwrap().to_string();
    let teacher_id = teacher_id
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<i32>()
        .unwrap_or(0);
    let cr_cost = cr_cost
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<i32>()
        .unwrap_or(0);
    let timeslots = timeslots.unwrap().to_str().unwrap().to_string();

    if teacher_id == 0 {
        return HttpResponse::BadRequest().json(json!({"error": "Invalid teacher id."}));
    }

    if cr_cost == 0 {
        return HttpResponse::BadRequest().json(json!({"error": "Invalid course cost."}));
    }

    let course = Course {
        id: 0, // This will be set by the database.
        description,
        teacher_id,
        course: name.to_string(),
        course_nr,
        cr_cost,
        timeslots,
    };

    match conn.register_courses(vec![course]) {
        Ok(_) => HttpResponse::Ok().json(json!({"message": "Successfully registered course."})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[delete("/courses/{id}")]
pub async fn remove_course(req: HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();
    let request_headers = req.headers();

    let id = req.match_info().get("id").unwrap();

    let requestee = request_headers.get("id");

    if requestee.is_none() {
        return HttpResponse::BadRequest().json(json!({"error": "Not logged in."}));
    }

    let requestee = requestee.unwrap().to_str().unwrap();
    let find_course = conn.search_courses(id.to_string());

    match find_course {
        Ok(c) => {
            if c.len() == 0 {
                return HttpResponse::BadRequest().json(json!({"error": "Course not found."}));
            }

            if c.len() > 1 {
                return HttpResponse::InternalServerError()
                    .json(json!({"error": "Multiple courses found."}));
            }

            if c.get(0).unwrap().0.teacher_id != requestee.parse::<i32>().unwrap() {
                return HttpResponse::BadRequest().json(json!({"error": "Unauthorized."}));
            }

            let course = c.get(0).unwrap().0.clone();

            match conn.remove_courses(vec![course]) {
                Ok(_) => {
                    HttpResponse::Ok().json(json!({"message": "Successfully removed course."}))
                }
                Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
            }
        }

        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({"error": e.to_string()}));
        }
    }
}

#[patch("/courses/{id}")]
pub async fn update_course(req: HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();
    let request_headers = req.headers();

    let name = request_headers.get("name");
    let description = request_headers.get("description");
    let course_nr = request_headers.get("course_nr");
    let teacher_id = request_headers.get("id");
    let cr_cost = request_headers.get("cr_cost");
    let timeslots = request_headers.get("timeslots");

    let id = req.match_info().get("id").unwrap();

    if teacher_id.is_none()
        || cr_cost.is_none()
        || timeslots.is_none()
        || name.is_none()
        || course_nr.is_none()
    {
        return HttpResponse::BadRequest().json(json!({"error": "Missing data."}));
    }

    let teacher_id = teacher_id.unwrap().to_str().unwrap();
    let cr_cost = cr_cost.unwrap().to_str().unwrap();
    let timeslots = timeslots.unwrap().to_str().unwrap();
    let name = name.unwrap().to_str().unwrap();
    let course_nr = course_nr.unwrap().to_str().unwrap().to_string();
    let description = description
        .unwrap()
        .to_str()
        .unwrap_or("No description.")
        .to_string();

    let find_course = conn.search_courses(id.to_string());

    match find_course {
        Ok(c) => {
            if c.len() == 0 {
                return HttpResponse::BadRequest().json(json!({"error": "Course not found."}));
            }

            if c.len() > 1 {
                return HttpResponse::InternalServerError()
                    .json(json!({"error": "Multiple courses found."}));
            }

            if c.get(0).unwrap().0.teacher_id != teacher_id.parse::<i32>().unwrap() {
                return HttpResponse::BadRequest().json(json!({"error": "Unauthorized."}));
            }

            let mut course = c.get(0).unwrap().0.clone();

            course.course = name.to_string();
            course.description = description;
            course.course_nr = course_nr;
            course.cr_cost = cr_cost.parse::<i32>().unwrap();
            course.timeslots = timeslots.to_string();

            match conn.remove_courses(vec![course]) {
                Ok(_) => {
                    HttpResponse::Ok().json(json!({"message": "Successfully removed course."}))
                }
                Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
            }
        }

        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({"error": e.to_string()}));
        }
    }
}

#[get("/admin")]
pub async fn admin(req: HttpRequest) -> impl Responder {
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
        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[patch("/admin/users/{id}")]
pub async fn update_user(req: HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();
    let request_headers = req.headers();

    let id = match req.match_info().get("id").unwrap().parse::<i32>() {
        Ok(i) => i,
        Err(_) => return HttpResponse::BadRequest().json(json!({"error": "Invalid id."})),
    };
    let mut username = match request_headers.get("username") {
        Some(u) => u.to_str().unwrap().to_string(),
        None => String::new(),
    };
    let mut password = match request_headers.get("password") {
        Some(p) => p.to_str().unwrap().to_string(),
        None => String::new(),
    };
    let mut email = match request_headers.get("email") {
        Some(e) => e.to_str().unwrap().to_string(),
        None => String::new(),
    };
    let mut phone = match request_headers.get("phone") {
        Some(p) => p.to_str().unwrap().to_string(),
        None => String::new(),
    };
    let mut verified = match request_headers.get("verified") {
        Some(v) => match v.to_str().unwrap().parse::<bool>() {
            Ok(b) => b,
            Err(_) => {
                return HttpResponse::BadRequest().json(json!({"error": "Invalid verified."}))
            }
        },
        None => false,
    };
    let mut suspended = match request_headers.get("suspended") {
        Some(s) => match s.to_str().unwrap().parse::<bool>() {
            Ok(b) => b,
            Err(_) => {
                return HttpResponse::BadRequest().json(json!({"error": "Invalid suspended."}))
            }
        },
        None => false,
    };
    let mut forcenewpw = match request_headers.get("forcenewpw") {
        Some(f) => match f.to_str().unwrap().parse::<bool>() {
            Ok(b) => b,
            Err(_) => {
                return HttpResponse::BadRequest().json(json!({"error": "Invalid forcenewpw."}))
            }
        },
        None => false,
    };
    let mut role = match request_headers.get("role") {
        Some(r) => r.to_str().unwrap().to_string(),
        None => String::new(),
    };

    let mut lookup_user = match conn.search_users(format!("{}", id)) {
        Ok(u) => match u.get(0) {
            Some(u) => u.to_owned(),
            None => return HttpResponse::BadRequest().json(json!({"error": "User not found."})),
        },
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({"error": e.to_string()}));
        }
    }
    .to_owned();

    if username == "" {
        username = lookup_user.username;
    }
    if password == "" {
        password = lookup_user.password;
    }
    if email == "" {
        email = lookup_user.email;
    }
    if phone == "" {
        phone = lookup_user.phone;
    }
    if role == "" {
        role = lookup_user.role;
    }
    if verified == false {
        verified = lookup_user.verified; // it might do another reassignment of the same value, but this is done because I unwrapped the option
    }
    if suspended == false {
        suspended = lookup_user.suspended;
    }
    if forcenewpw == false {
        forcenewpw = lookup_user.forcenewpw;
    }

    lookup_user.username = username;
    lookup_user.password = password;
    lookup_user.email = email;
    lookup_user.phone = phone;
    lookup_user.verified = verified;
    lookup_user.suspended = suspended;
    lookup_user.forcenewpw = forcenewpw;
    lookup_user.role = role;

    match conn.update_user(lookup_user.clone()) {
        Ok(_) => {
            let json = serde_json::to_string(&lookup_user);
            match json {
                Ok(j) => return HttpResponse::Ok().body(j),
                Err(e) => {
                    return HttpResponse::InternalServerError()
                        .json(json!({"error": e.to_string()}))
                }
            }
        }
        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[delete("/admin/users/{id}")]
pub async fn delete_user(req: HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();

    let id = match req.match_info().get("id").unwrap().parse::<i32>() {
        Ok(i) => i,
        Err(_) => return HttpResponse::BadRequest().json(json!({"error": "Invalid id."})),
    };

    let fetch_user = match conn.search_users(format!("{}", id)) {
        Ok(u) => match u.get(0) {
            Some(u) => u.to_owned(),
            None => return HttpResponse::BadRequest().json(json!({"error": "User not found."})),
        },
        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    };

    match conn.delete_user(fetch_user) {
        Ok(_) => HttpResponse::Ok().json(json!({"message": "Successfully deleted user."})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[delete("/account")]
pub async fn delete_self(req: HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();
    let request_headers = req.headers();

    let id = request_headers.get("id");
    let email = request_headers.get("email");
    let password = request_headers.get("password");

    if id.is_none() || email.is_none() || password.is_none() {
        return HttpResponse::BadRequest().json(json!({"error": "Missing id, email or password"}));
    }

    let id = match id.unwrap().to_str().unwrap().parse::<i32>() {
        Ok(i) => i,
        Err(_) => return HttpResponse::BadRequest().json(json!({"error": "Invalid id."})),
    };

    let email = email.unwrap().to_str().unwrap().to_string();
    let password = password.unwrap().to_str().unwrap().to_string();

    let user = conn.search_users(format!("{}", email));

    match user {
        Ok(u) => {
            if u.len() == 0 {
                return HttpResponse::BadRequest().json(json!({"error": "User not found"}));
            }

            if u.len() > 1 {
                return HttpResponse::InternalServerError()
                    .json(json!({"error": "Multiple users found."}));
            }

            let user = u.get(0).unwrap();

            if user.id != id {
                return HttpResponse::BadRequest().json(json!({"error": "Unauthorized."}));
            }

            if user.password != password {
                return HttpResponse::BadRequest().json(json!({"error": "Unauthorized."}));
            }

            match conn.delete_user(user.to_owned()) {
                Ok(_) => {
                    HttpResponse::Ok()
                        .json(json!({"message": "Successfully deleted user."}))
                }
                Err(e) => {
                    HttpResponse::InternalServerError()
                        .json(json!({"error": e.to_string()}))
                }
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[patch("/account")]
pub async fn update_self(req: HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();
    let request_headers = req.headers();

    let id = request_headers.get("id");

    let username = request_headers.get("username");
    let email = request_headers.get("email");
    let password = request_headers.get("password");
    let phone = request_headers.get("phone");
    let role = request_headers.get("role");

    if id.is_none() {
        return HttpResponse::BadRequest().json(json!({"error": "Missing id"}));
    }

    let id = match id.unwrap().to_str().unwrap().parse::<i32>() {
        Ok(i) => i,
        Err(_) => return HttpResponse::BadRequest().json(json!({"error": "Invalid id."})),
    };

    let mut username = match username {
        Some(u) => u.to_str().unwrap().to_string(),
        None => String::new(),
    };
    let mut email = match email {
        Some(e) => e.to_str().unwrap().to_string(),
        None => String::new(),
    };
    let mut password = match password {
        Some(p) => p.to_str().unwrap().to_string(),
        None => String::new(),
    };
    let mut phone = match phone {
        Some(p) => p.to_str().unwrap().to_string(),
        None => String::new(),
    };
    let mut role = match role {
        Some(r) => r.to_str().unwrap().to_string(),
        None => String::new(),
    };

    let user = conn.search_users(format!("{}", id));

    match user {
        Ok(u) => {
            if u.len() == 0 {
                return HttpResponse::BadRequest().json(json!({"error": "User not found"}));
            }

            if u.len() > 1 {
                return HttpResponse::InternalServerError()
                    .json(json!({"error": "Multiple users found."}));
            }

            let mut user = u.get(0).unwrap().to_owned();

            if username == "" {
                username = user.username;
            }
            if email == "" {
                email = user.email;
            }
            if password == "" {
                password = user.password;
            }
            if phone == "" {
                phone = user.phone;
            }
            if role == "" {
                role = user.role;
            }

            user.username = username;
            user.email = email;
            user.password = password;
            user.phone = phone;
            user.role = role;

            match conn.update_user(user.clone()) {
                Ok(_) => {}
                Err(e) => {
                    return HttpResponse::InternalServerError()
                        .json(json!({"error": e.to_string()}))
                }
            }
            let json = serde_json::to_string(&user);

            match json {
                Ok(j) => return HttpResponse::Ok().body(j),
                Err(e) => {
                    return HttpResponse::InternalServerError().json(json!({"error": e.to_string()}))
                }
            }
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({"error": e.to_string()}))
        }
    }
}

#[get("/enroll/{id}")]
pub async fn enroll(req: HttpRequest) -> impl Responder {
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

    let email = email.unwrap().to_str().unwrap().to_string();
    let password = password.unwrap().to_str().unwrap().to_owned();
    let course_id = course_id.unwrap().to_owned();

    let user = conn.search_users(format!("{}", email));

    match user {
        Ok(u) => {
            if u.len() == 0 {
                return HttpResponse::BadRequest().json(json!({"error": "User not found"}));
            } else {
                let user = u.get(0).unwrap();

                match conn.login(email, password) {
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
        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
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

    let email = email.unwrap().to_str().unwrap().to_string();
    let password = password.unwrap().to_str().unwrap().to_owned();
    let course_id = course_id.unwrap().to_owned();

    let user = conn.search_users(format!("{}", email));

    match user {
        Ok(u) => {
            if u.len() == 0 {
                return HttpResponse::BadRequest().json(json!({"error": "User not found"}));
            } else {
                let user = u.get(0).unwrap();

                match conn.login(email, password) {
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
        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
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
        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
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

#[get("/admin/register")]
pub async fn register_admin(req: actix_web::HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();
    let request_headers = req.headers();

    let username = request_headers.get("username");
    let password = request_headers.get("password");
    let email = request_headers.get("email");
    let phone = request_headers.get("phone");

    match request_headers.get("access_code") {
        Some(c) => if c.to_str().unwrap() != "I_BECOME_THY_ADMIN_AND_I_FUCK_YOUR_MOTHER32131!@#@!#@!" {
            return HttpResponse::BadRequest()
                .json(json!({"error": "Invalid access code."}))
        },
        None => {
            return HttpResponse::BadRequest()
                .json(json!({"error": "Missing access code."}))
        }
    };

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
        role: String::from("admin"),
    };

    match conn.register_user(u) {
        Ok(_) => HttpResponse::Ok().json(json!({"message": "Successfully registered."})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}