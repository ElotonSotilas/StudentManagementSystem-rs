use super::db_driver::*;
use super::filter::*;
use super::password;
use super::table_models::*;

use anyhow::anyhow;
use anyhow::Ok;
use anyhow::Result;
use chrono::Datelike;
use regex::Regex;

pub struct Statistics {
    pub registered_users: i32,
    pub suspended_users: i32,
    pub faculty_members: i32,
    pub active_students: i32,
    pub graduated_students: i32,
    pub courses: i32,
    pub departments: i32,
}

pub struct ServerConnection {
    db: DbDriver,
    session: Option<User>,
}

// Public methods
impl ServerConnection {
    pub fn new() -> Self {
        Self {
            db: DbDriver::init(),
            session: None,
        }
    }

    // fetch all users from the database
    pub fn get_users(&self) -> Result<Vec<User>> {
        let users = self.db.find(Table::Users, vec![], None)?;
        let u = users
            .into_iter()
            .map(|x| {
                if let ReceiverType::User(user) = x {
                    user
                } else {
                    unreachable!()
                }
            })
            .collect();

        Ok(u)
    }

    pub fn get_user(&self, id: i32) -> Result<User> {
        let finding = self
            .db
            .find(Table::Users, vec![Filter::Users(UsersFilter::Id(id))], None)?;

        // this cannot have more than 1 entry, that's pretty much a bug if it happens, blame the database
        assert!(finding.len() <= 1);

        if finding.is_empty() {
            Err(anyhow!("No user found"))
        } else if let ReceiverType::User(user) = &finding[0] {
            Ok(user.to_owned())
        } else {
            Err(anyhow!("Internal server error"))
        }
    }

    pub fn get_users_by_filters(&self, filters: Vec<Filter>) -> Result<Vec<User>> {
        let finding = self.db.find(Table::Users, filters, None)?;

        let users = finding
            .into_iter()
            .filter_map(|x| {
                if let ReceiverType::User(user) = x {
                    Some(user)
                } else {
                    None
                }
            })
            .collect();

        Ok(users)
    }

    pub fn register_user(&mut self, user: User) -> Result<()> {
        if !self.session.is_none() {
            return Err(anyhow!("Must be signed out."));
        }

        let email_regex = Regex::new(r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@aubg\.edu$")?;
        let phone_regex = Regex::new(r#"^\+?[0-9]{2}[-. ]?[0-9]{4}[-. ]?[0-9]{4}$"#)?;
        let password_rules = user.password.len() >= 8
            && user.password.chars().any(|c| c.is_ascii_lowercase())
            && user.password.chars().any(|c| c.is_ascii_uppercase())
            && user.password.chars().any(|c| c.is_ascii_digit())
            && user.password.chars().any(|c| "@$!%*?&".contains(c));

        if self
            .get_users_by_filters(vec![Filter::Users(UsersFilter::Email(
                user.email.to_lowercase().clone(),
            ))])?
            .len()
            > 0
        {
            return Err(anyhow!("A user with this email already exists."));
        }

        if user.username.is_empty() {
            return Err(anyhow!("Account name cannot be empty."));
        }

        if !email_regex.is_match(&user.email) {
            return Err(anyhow!("Must be a valid AUBG email."));
        }

        if !phone_regex.is_match(&user.phone) && !user.phone.is_empty() {
            return Err(anyhow!("Invalid phone number."));
        }

        if !password_rules {
            return Err(anyhow!(
                "The password does not meet the following criteria:\n
            - Must be at least 8 characters long\n
            - Must contain at least 1 uppercase letter\n
            - Must contain at least 1 lowercase letter\n
            - Must contain at least 1 number\n
            - Must contain at least 1 special character (@, $, !, %, *, ?, &)\n"
            ));
        }

        let mut user = user.to_owned();

        let salt = password::generate_salt();
        user.password = password::hash(&user.password, salt);

        self.db.insert(vec![ReceiverType::User(user)])?;

        Ok(())
    }

    pub fn login(&mut self, email: String, password: String) -> Result<()> {
        let binding = self.get_users_by_filters(vec![Filter::Users(UsersFilter::Email(email))])?;
        let user = binding.get(0).ok_or_else(|| anyhow!("User not found."))?; // if none, user not found

        // If the user is suspended, they cannot login
        if user.suspended {
            return Err(anyhow!("User is suspended."));
        }

        // check hash for validity and then compare both server and client password hashes
        if password::verify(&user.password, &password) {
            self.session = Some(user.to_owned());
            Ok(())
        } else {
            Err(anyhow!("Invalid username or password."))
        }
    }

    pub fn update_user(&mut self, user: User) -> Result<()> {
        if let Some(s) = &self.session {
            match s.role.to_lowercase().as_str() {
                "admin" => self.update_user_as_admin(user)?,
                _ => self.update_user_as_student(user)?,
            }
        } else {
            return Err(anyhow!("Must be signed in."));
        }

        Ok(())
    }

    pub fn delete_user(&mut self, user: User) -> Result<()> {
        if let Some(s) = &self.session {
            match s.role.to_lowercase().as_str() {
                "admin" => {
                    if user.id != s.id {
                        self.db.delete(vec![ReceiverType::User(user.clone())])?;

                        Ok(())
                    } else {
                        Err(anyhow!(
                            "You cannot delete your own account as an administrator."
                        ))
                    }
                }
                _ => {
                    if user.id == s.id {
                        self.db.delete(vec![ReceiverType::User(user.clone())])?;

                        Ok(())
                    } else {
                        Err(anyhow!("You do not have permission to delete this user."))
                    }
                }
            }
        } else {
            Err(anyhow!("Must be signed in."))
        }
    }

    pub fn register_courses(&mut self, courses: Vec<Course>) -> Result<()> {
        if let Some(s) = &self.session {
            match s.role.to_lowercase().as_str() {
                "admin" => {
                    let upcast = courses
                        .iter()
                        .map(|x| ReceiverType::Course(x.to_owned()))
                        .collect();

                    self.db.insert(upcast)?;

                    Ok(())
                }
                "teacher" => {
                    let errors = courses
                        .iter()
                        .filter_map(|x| {
                            if x.teacher_id != s.id {
                                Some(
                                    anyhow!(
                                        "You do not have permission to register courses on someone else's behalf."
                                    )
                                )
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();

                    if errors.len() > 0 {
                        return Err(
                            anyhow!(
                                "You do not have permission to register courses on someone else's behalf. No action was taken."
                            )
                        );
                    }

                    let upcast = courses
                        .iter()
                        .map(|x| ReceiverType::Course(x.to_owned()))
                        .collect();

                    self.db.insert(upcast)?;

                    Ok(())
                }
                _ => Err(anyhow!("You do not have permission to register courses.")),
            }
        } else {
            Err(anyhow!("Must be signed in."))
        }
    }

    pub fn remove_courses(&mut self, courses: Vec<Course>) -> Result<()> {
        if let Some(session) = &self.session {
            match session.role.to_lowercase().as_str() {
                "admin" => {
                    let upcast = courses
                        .iter()
                        .map(|x| ReceiverType::Course(x.to_owned()))
                        .collect();

                    self.db.delete(upcast)?;

                    Ok(())
                }
                "teacher" => {
                    let errors = courses
                        .iter()
                        .filter_map(|x| {
                            if x.teacher_id != session.id {
                                Some(anyhow!("You do not have permission to remove this course."))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();

                    if errors.len() > 0 {
                        return Err(anyhow!(
                            "Some courses to not belong to you. No action was taken."
                        ));
                    }

                    let upcast = courses
                        .iter()
                        .map(|x| ReceiverType::Course(x.to_owned()))
                        .collect();

                    self.db.delete(upcast)?;

                    Ok(())
                }
                _ => Err(anyhow!("You do not have permission to remove courses.")),
            }
        } else {
            Err(anyhow!("Must be signed in."))
        }
    }

    pub fn update_courses(&mut self, courses: Vec<Course>) -> Result<()> {
        if let Some(session) = &self.session {
            match session.role.to_lowercase().as_str() {
                "admin" => {
                    let upcast = courses
                        .iter()
                        .map(|x| ReceiverType::Course(x.to_owned()))
                        .collect();

                    self.db.update(upcast)?;

                    Ok(())
                }
                "teacher" => {
                    let errors = courses
                        .iter()
                        .filter_map(|x| {
                            if x.teacher_id != session.id {
                                Some(anyhow!("You do not have permission to update this course."))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();

                    if errors.len() > 0 {
                        return Err(anyhow!(
                            "Some courses to not belong to you. No action was taken."
                        ));
                    }

                    let upcast = courses
                        .iter()
                        .map(|x| ReceiverType::Course(x.to_owned()))
                        .collect();

                    self.db.update(upcast)?;

                    Ok(())
                }
                _ => Err(anyhow!("You do not have permission to update courses.")),
            }
        } else {
            Err(anyhow!("Must be signed in."))
        }
    }

    pub fn search_users(&self, query: String) -> Result<Vec<User>> {
        let findings = self.db.find(
            Table::Users,
            vec![Filter::Users(UsersFilter::All)],
            Some(Associativity::Or),
        )?;

        let users = findings
            .into_iter()
            .filter_map(|x| {
                if let ReceiverType::User(user) = x {
                    Some(user)
                } else {
                    None
                }
            })
            .filter(|x| {
                x.username.contains(&query)
                    || x.email.contains(&query)
                    || x.phone.contains(&query)
                    || x.id.to_string().contains(&query)
            })
            .collect();

        Ok(users)
    }

    pub fn search_courses(
        &self,
        query: String,
    ) -> Result<Vec<(Course, TeacherAccount, Department, User)>> {
        let course_query = self.db.find(
            Table::Courses,
            vec![Filter::Courses(CoursesFilter::All)],
            Some(Associativity::Or),
        )?;
        let teachers_query = self.db.find(
            Table::TeacherAccount,
            vec![Filter::TeacherAccount(TeacherAccountFilter::All)],
            Some(Associativity::Or),
        )?;
        let department_query = self.db.find(
            Table::Departments,
            vec![Filter::Departments(DepartmentsFilter::All)],
            Some(Associativity::Or),
        )?;
        let users_query = self.db.find(
            Table::Users,
            vec![Filter::Users(UsersFilter::All)],
            Some(Associativity::Or),
        )?;

        let query = query.trim().to_lowercase(); // trim and convert to lowercase
        let mut joined_data = Vec::new();

        self.join_courses_with_data(
            course_query,
            teachers_query,
            department_query,
            users_query,
            query,
            &mut joined_data,
        );

        Ok(joined_data)
    }

    pub fn enroll_courses(&mut self, courses: Vec<Course>) -> Result<()> {
        if let Some(session) = &self.session {
            match session.role.to_lowercase().as_str() {
                "student" => {
                    let errors = courses
                        .iter()
                        .filter_map(|x| {
                            let c = self.transmute_course_to_student_course(x.to_owned());
                            if c.student_id != session.id {
                                Some(
                                    anyhow!(
                                        "You do not have permission to register courses on someone else's behalf."
                                    )
                                )
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();

                    if errors.len() > 0 {
                        return Err(
                            anyhow!(
                                "You do not have permission to register courses on someone else's behalf. No action was taken."
                            )
                        );
                    }

                    let upcast = courses
                        .iter()
                        .map(|x| {
                            ReceiverType::StudentCourse(
                                self.transmute_course_to_student_course(x.to_owned()),
                            )
                        })
                        .collect();

                    self.db.insert(upcast)?;

                    Ok(())
                }
                _ => Err(anyhow!("You do not have permission to enroll courses.")),
            }
        } else {
            Err(anyhow!("Must be signed in."))
        }
    }

    pub fn drop_courses(&mut self, courses: Vec<Course>) -> Result<()> {
        if let Some(session) = &self.session {
            match session.role.to_lowercase().as_str() {
                "student" => {
                    let errors = courses
                        .iter()
                        .filter_map(|x| {
                            let c = self.transmute_course_to_student_course(x.to_owned());
                            if c.student_id != session.id {
                                Some(
                                    anyhow!(
                                        "You do not have permission to register courses on someone else's behalf."
                                    )
                                )
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();

                    if errors.len() > 0 {
                        return Err(
                            anyhow!(
                                "You do not have permission to register courses on someone else's behalf. No action was taken."
                            )
                        );
                    }

                    let upcast = courses
                        .iter()
                        .map(|x| {
                            ReceiverType::StudentCourse(
                                self.transmute_course_to_student_course(x.to_owned()),
                            )
                        })
                        .collect();

                    self.db.delete(upcast)?;

                    Ok(())
                }
                _ => Err(anyhow!("You do not have permission to drop courses.")),
            }
        } else {
            Err(anyhow!("Must be signed in."))
        }
    }

    pub fn set_user_role(&mut self, user: User, role: &str) -> Result<()> {
        if let Some(session) = &self.session {
            match session.role.to_lowercase().as_str() {
                "admin" => {
                    let mut updated = user.to_owned();
                    updated.role = role.to_string();
                    let upcast = ReceiverType::User(updated);

                    self.db.update(vec![upcast])?;

                    Ok(())
                }

                _ => {
                    return Err(anyhow!(
                        "You do not have permission to promote users to teachers."
                    ))
                }
            }
        } else {
            Err(anyhow!("Must be signed in."))
        }
    }

    pub fn suspend_user(&mut self, user: User) -> Result<()> {
        if let Some(session) = &self.session {
            match session.role.to_lowercase().as_str() {
                "admin" => {
                    let mut updated = user.to_owned();
                    updated.suspended = true;
                    let upcast = ReceiverType::User(updated);

                    self.db.update(vec![upcast])?;

                    Ok(())
                }

                _ => Err(anyhow!("You do not have permission to suspend users.")),
            }
        } else {
            Err(anyhow!("Must be signed in."))
        }
    }

    pub fn unsuspend_user(&mut self, user: User) -> Result<()> {
        if let Some(session) = &self.session {
            match session.role.to_lowercase().as_str() {
                "admin" => {
                    let mut updated = user.to_owned();
                    updated.suspended = false;
                    let upcast = ReceiverType::User(updated);

                    self.db.update(vec![upcast])?;

                    Ok(())
                }

                _ => Err(anyhow!("You do not have permission to unsuspend users.")),
            }
        } else {
            Err(anyhow!("Must be signed in."))
        }
    }

    pub fn generate_statistics(&self) -> Result<Statistics> {
        let registered_users = self.get_users()?.len() as i32;
        let suspended_users = self
            .get_users_by_filters(vec![Filter::Users(UsersFilter::Suspended(true))])?
            .len() as i32;
        let faculty_members = self
            .get_users_by_filters(vec![Filter::Users(UsersFilter::Role(
                "teacher".to_string(),
            ))])?
            .len() as i32;
        let active_students = self
            .get_users_by_filters(vec![Filter::Users(UsersFilter::Role(
                "student".to_string(),
            ))])?
            .len() as i32
            - suspended_users;
        let graduated_students = self
            .get_users_by_filters(vec![Filter::StudentAccount(StudentAccountFilter::CanGrad(
                true,
            ))])?
            .len() as i32;
        let courses = self.db.find(Table::Courses, vec![], None)?.len() as i32;
        let departments = self.db.find(Table::Departments, vec![], None)?.len() as i32;

        Ok(Statistics {
            registered_users,
            suspended_users,
            faculty_members,
            active_students,
            graduated_students,
            courses,
            departments,
        })
    }

    pub fn logout(&mut self) {
        self.session = None;
    }
}

// Private methods
impl ServerConnection {
    fn transmute_course_to_student_course(&self, course: Course) -> StudentCourse {
        StudentCourse {
            student_id: self.session.as_ref().unwrap().id,
            course_id: course.id,
            grade: -1.0,
            semester: match chrono::Local::now().month() {
                6..=12 => "Fall".to_string(),
                _ => "Spring".to_string(),
            },
        }
    }

    fn join_courses_with_data(
        &self,
        course_query: Vec<ReceiverType>,
        teachers_query: Vec<ReceiverType>,
        department_query: Vec<ReceiverType>,
        users_query: Vec<ReceiverType>,
        query: String,
        joined_data: &mut Vec<(Course, TeacherAccount, Department, User)>,
    ) {
        course_query.into_iter().for_each(|course_item| {
            if let ReceiverType::Course(course) = course_item {
                let teacher_id = course.teacher_id;

                self.join_with_teacher(
                    &teachers_query,
                    teacher_id,
                    &department_query,
                    &users_query,
                    &query,
                    course,
                    joined_data,
                );
            }
        });
    }

    fn join_with_teacher(
        &self,
        teachers_query: &Vec<ReceiverType>,
        teacher_id: i32,
        department_query: &Vec<ReceiverType>,
        users_query: &Vec<ReceiverType>,
        query: &String,
        course: Course,
        joined_data: &mut Vec<(Course, TeacherAccount, Department, User)>,
    ) {
        if let Some(ReceiverType::TeacherAccount(teacher)) = teachers_query.iter().find(|&t| {
            if let ReceiverType::TeacherAccount(teacher) = t {
                teacher.id == teacher_id
            } else {
                false
            }
        }) {
            let dept_id = teacher.dept_id;

            self.join_with_department(
                department_query,
                dept_id,
                teacher,
                users_query,
                query,
                course,
                joined_data,
            );
        }
    }

    fn join_with_department(
        &self,
        department_query: &Vec<ReceiverType>,
        dept_id: i32,
        teacher: &TeacherAccount,
        users_query: &Vec<ReceiverType>,
        query: &String,
        course: Course,
        joined_data: &mut Vec<(Course, TeacherAccount, Department, User)>,
    ) {
        if let Some(ReceiverType::Department(department)) = department_query.iter().find(|&d| {
            if let ReceiverType::Department(department) = d {
                department.id == dept_id
            } else {
                false
            }
        }) {
            let teacher_id = teacher.teacher_id;

            self.join_with_user(
                users_query,
                teacher_id,
                query,
                course,
                teacher,
                department,
                joined_data,
            );
        }
    }

    fn join_with_user(
        &self,
        users_query: &Vec<ReceiverType>,
        teacher_id: i32,
        query: &String,
        course: Course,
        teacher: &TeacherAccount,
        department: &Department,
        joined_data: &mut Vec<(Course, TeacherAccount, Department, User)>,
    ) {
        if let Some(ReceiverType::User(user)) = users_query.iter().find(|&u| {
            if let ReceiverType::User(user) = u {
                user.id == teacher_id
            } else {
                false
            }
        }) {
            // Check if the query string is empty or matches any of the fields
            let query_matched = self.did_query_match(query, &course, teacher, department, user);

            if query_matched {
                joined_data.push((
                    course.clone(),
                    teacher.clone(),
                    department.clone(),
                    user.clone(),
                ));
            }
        }
    }

    fn did_query_match(
        &self,
        query: &String,
        course: &Course,
        teacher: &TeacherAccount,
        department: &Department,
        user: &User,
    ) -> bool {
        let query_matched = query.is_empty()
            || course.course.contains(query)
            || course.id.to_string().contains(query)
            || teacher.id.to_string().contains(query)
            || teacher.dept.contains(query)
            || teacher.teacher_id.to_string().contains(query)
            || department.id.to_string().contains(query)
            || user.id.to_string().contains(query)
            || user.email.contains(query)
            || user.username.contains(query)
            || user.phone.contains(query);
        query_matched
    }

    fn update_user_as_student(&mut self, user: User) -> Result<()> {
        let binding =
            self.get_users_by_filters(vec![Filter::Users(UsersFilter::Id(user.id.clone()))])?;
        let u = binding.get(0).ok_or_else(|| anyhow!("User not found."))?;

        // Check permissions

        if user.username != u.username {
            return Err(anyhow!("Username cannot be changed."));
        }

        if user.suspended != u.suspended {
            return Err(anyhow!("Suspended cannot be changed."));
        }

        if user.verified != u.verified {
            return Err(anyhow!("Verified cannot be changed."));
        }

        if user.role != u.role {
            return Err(anyhow!("Role cannot be changed."));
        }

        self.db.update(vec![ReceiverType::User(user)])?;

        Ok(())
    }

    fn update_user_as_admin(&mut self, user: User) -> Result<()> {
        let binding =
            self.get_users_by_filters(vec![Filter::Users(UsersFilter::Id(user.id.clone()))])?;
        let u = binding.get(0).ok_or_else(|| anyhow!("User not found."))?;

        self.db.update(vec![ReceiverType::User(user)])?;

        Ok(())
    }
}
