use std::fmt::{Display, Formatter};
use serde_derive::{Deserialize, Serialize};
use super::db_driver::Join;

pub enum Action {
    Insert,
    Update,
    Delete
}

pub enum Table {
    Users,
    StudentAccount,
    TeacherAccount,
    Courses,
    StudentCourses,
    Departments
}

impl Display for Table {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Table::Users => write!(f, "USERS"),
            Table::StudentAccount => write!(f, "STUDENT_ACCOUTNS"),
            Table::TeacherAccount => write!(f, "TEACHER_ACCOUNTS"),
            Table::Courses => write!(f, "COURSES"),
            Table::StudentCourses => write!(f, "STUDENT_COURSES"),
            Table::Departments => write!(f, "DEPARTMENTS")
        }
    }
}

impl Table {
    pub fn join(&self, other: &Table, join_as: Join) -> String {
        format!("{} {} {}", self, join_as, other)
    }
}

pub trait ToSQL {
    fn to_sql(&self, a: Action) -> String;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub email: String,
    pub phone: String,
    pub created_at: String,
    pub updated_at: String,
    pub verified: bool,
    pub suspended: bool,
    pub forcenewpw: bool,
    pub role: String,
}

impl ToSQL for User {
    fn to_sql(&self, a: Action) -> String {
        match a {
            Action::Insert => format!(
                "INSERT INTO USERS (username, password, email, phone, created_at, updated_at, 
                    verified, suspended, forcenewpw, role) VALUES ('{}', '{}', '{}', '{}', '{}', '{}', {}, {}, {}, '{}')",
                self.username, self.password, self.email, self.phone, self.created_at, self.updated_at, 
                    self.verified, self.suspended, self.forcenewpw, self.role
            ),

            Action::Update => format!(
                "UPDATE USERS SET username = '{}', password = '{}', email = '{}', phone = '{}', 
                    created_at = '{}', updated_at = '{}', verified = {}, suspended = {}, forcenewpw = {}, role = '{}' 
                    WHERE id = {}",
                self.username, self.password, self.email, self.phone, self.created_at, self.updated_at, 
                    self.verified, self.suspended, self.forcenewpw, self.role, self.id
            ),

            Action::Delete => format!(
                "DELETE FROM USERS WHERE id = {}", self.id
            )
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentAccount {
    pub id: i32,
    pub student_id: i32,
    pub advisor_id: i32,
    pub discipline: String,
    pub enrollment: String,
    pub cgpa: f32,
    pub can_grad: bool,
    pub cur_credit: i32,
    pub cum_credit: i32,
}

impl ToSQL for StudentAccount {
    fn to_sql(&self, a: Action) -> String {
        match a {
            Action::Insert => format!(
                "INSERT INTO STUDENT_ACCOUNTS (student_id, advisor_id, discipline, enrollment, cgpa, can_grad, cur_credit, cum_credit) 
                VALUES ({}, {}, '{}', '{}', {}, {}, {}, {})",
                self.student_id, self.advisor_id, self.discipline, self.enrollment, self.cgpa, self.can_grad, self.cur_credit, self.cum_credit
            ),

            Action::Update => format!(
                "UPDATE STUDENT_ACCOUNTS SET student_id = {}, advisor_id = {}, discipline = '{}', 
                enrollment = '{}', cgpa = {}, can_grad = {}, cur_credit = {}, cum_credit = {} 
                WHERE id = {}",
                self.student_id, self.advisor_id, self.discipline, self.enrollment, self.cgpa, 
                self.can_grad, self.cur_credit, self.cum_credit, self.id
            ),

            Action::Delete => format!(
                "DELETE FROM STUDENT_ACCOUNTS WHERE id = {}", self.id
            )
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeacherAccount {
    pub id: i32,
    pub teacher_id: i32,
    pub dept_id: i32,
    pub dept: String,
}

impl ToSQL for TeacherAccount {
    fn to_sql(&self, a: Action) -> String {
        match a {
            Action::Insert => format!(
                "INSERT INTO TEACHER_ACCOUNTS (teacher_id, dept_id, dept) VALUES ({}, {}, '{}')",
                self.teacher_id, self.dept_id, self.dept
            ),

            Action::Update => format!(
                "UPDATE TEACHER_ACCOUNTS SET teacher_id = {}, dept_id = {}, dept = '{}' WHERE id = {}",
                self.teacher_id, self.dept_id, self.dept, self.id
            ),

            Action::Delete => format!(
                "DELETE FROM TEACHER_ACCOUNTS WHERE id = {}", self.id
            )
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Course {
    pub id: i32,
    pub teacher_id: i32,
    pub course: String,
    pub cr_cost: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl ToSQL for Course {
    fn to_sql(&self, a: Action) -> String {
        match a {
            Action::Insert => format!(
                "INSERT INTO COURSES (teacher_id, course, cr_cost, created_at, updated_at) 
                VALUES ({}, '{}', {}, '{}', '{}')",
                self.teacher_id, self.course, self.cr_cost, self.created_at, self.updated_at
            ),

            Action::Update => format!(
                "UPDATE COURSES SET teacher_id = {}, course = '{}', cr_cost = {}, created_at = '{}', 
                updated_at = '{}' WHERE id = {}",
                self.teacher_id, self.course, self.cr_cost, self.created_at, self.updated_at, self.id
            ),
            
            Action::Delete => format!(
                "DELETE FROM COURSES WHERE id = {}", self.id
            )
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentCourse {
    pub student_id: i32,
    pub course_id: i32,
    pub grade: f32,
    pub semester: String,
}

impl ToSQL for StudentCourse {
    fn to_sql(&self, a: Action) -> String {
        match a {
            Action::Insert => format!(
                "INSERT INTO student_courses (student_id, course_id, grade, semester) 
                VALUES ({}, {}, {}, '{}')",
                self.student_id, self.course_id, self.grade, self.semester
            ),

            Action::Update => format!(
                "UPDATE student_courses SET student_id = {}, course_id = {}, grade = {}, semester = '{}' 
                WHERE student_id = {} AND course_id = {}",
                self.student_id, self.course_id, self.grade, self.semester, self.student_id, self.course_id
            ),

            Action::Delete => format!(
                "DELETE FROM student_courses WHERE student_id = {} AND course_id = {}", self.student_id, self.course_id
            )
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Department {
    pub id: i32,
    pub dept_head: i32,
    pub name: String,
}

impl ToSQL for Department {
    fn to_sql(&self, a: Action) -> String {
        match a {
            Action::Insert => format!(
                "INSERT INTO departments (dept_head, name) VALUES ({}, '{}')",
                self.dept_head, self.name
            ),

            Action::Update => format!(
                "UPDATE departments SET dept_head = {}, name = '{}' WHERE id = {}",
                self.dept_head, self.name, self.id
            ),

            Action::Delete => format!(
                "DELETE FROM departments WHERE id = {}", self.id
            )
        }
    }
}
