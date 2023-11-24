use anyhow::{Ok, Result};
use rusqlite::Connection;

pub struct DatabaseConnection {
    pub connection: Connection,
}

impl DatabaseConnection {
    pub fn new() -> Result<Self> {
        let connection = Connection::open("system.db")?;

        Ok(Self { connection })
    }

    pub fn create_tables(&mut self) -> Result<&mut Self> {
        let table_count = self.connection.query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='tableName';",
            (),
            |row| row.get::<usize, usize>(0),
        )?;

        if table_count == 0 {
            self.connection.execute_batch(
                r#"
            BEGIN;
            CREATE TABLE "USERS" (
                id INTEGER PRIMARY KEY NOT NULL,
                username TEXT NOT NULL,
                password TEXT NOT NULL,
                email TEXT NOT NULL,
                phone TEXT NOT NULL,
                created_at DATE NOT NULL DEFAULT DATE('now'),
                updated_at DATE NOT NULL DEFAULT DATE('now'),
                verified BOOLEAN NOT NULL,
                suspended BOOLEAN NOT NULL,
                forcenewpw BOOLEAN NOT NULL,
                role TEXT NOT NULL
            );
            
            CREATE TRIGGER update_time_USERS
            AFTER UPDATE ON USERS
            FOR EACH ROW
            BEGIN
               UPDATE USERS SET updated_at = DATE('now') WHERE id = OLD.id;
            END;
            
            CREATE TABLE "STUDENT_ACCOUNT" (
                id INTEGER PRIMARY KEY NOT NULL,
                student_id INTEGER NOT NULL,
                advisor_id INTEGER NOT NULL,
                discipline TEXT NOT NULL,
                enrollment TEXT NOT NULL,
                cgpa REAL NOT NULL,
                can_grad BOOLEAN NOT NULL CHECK (cgpa > 2.0 AND cum_credit >= 120),
                cur_credit INTEGER NOT NULL,
                cum_credit INTEGER NOT NULL,
                FOREIGN KEY (student_id) REFERENCES USERS(id),
                FOREIGN KEY (advisor_id) REFERENCES USERS(id)
            );
            
            CREATE TABLE "TEACHER_ACCOUNT" (
                id INTEGER PRIMARY KEY NOT NULL,
                teacher_id INTEGER NOT NULL,
                dept_id INTEGER NOT NULL,
                dept TEXT NOT NULL,
                FOREIGN KEY (teacher_id) REFERENCES USERS(id),
                FOREIGN KEY (dept_id) REFERENCES DEPARTMENTS(id)
            );
            
            CREATE TABLE "COURSES" (
                id INTEGER PRIMARY KEY NOT NULL,
                teacher_id INTEGER NOT NULL,
                course TEXT NOT NULL,
                cr_cost INTEGER NOT NULL,
                created_at DATE NOT NULL DEFAULT DATE('now'),
                updated_at DATE NOT NULL DEFAULT DATE('now'),
                FOREIGN KEY (teacher_id) REFERENCES USERS(id)
            );
            
            CREATE TRIGGER update_time_COURSES
            AFTER UPDATE ON COURSES
            FOR EACH ROW
            BEGIN
               UPDATE COURSES SET updated_at = DATE('now') WHERE id = OLD.id;
            END;
            
            CREATE TABLE "STUDENT_COURSES" (
                student_id INTEGER NOT NULL,
                course_id INTEGER NOT NULL,
                grade REAL NOT NULL,
                semester TEXT NOT NULL,
                FOREIGN KEY (student_id) REFERENCES USERS(id),
                FOREIGN KEY (course_id) REFERENCES COURSES(id)
            );
            
            CREATE TABLE DEPARTMENTS (
                id INTEGER PRIMARY KEY NOT NULL,
                dept_head INTEGER NOT NULL,
                name TEXT NOT NULL,
                FOREIGN KEY (dept_head) REFERENCES TEACHER_ACCOUNT(id)
            );
            COMMIT;
            "#,
            )?;
        }

        Ok(self)
    }
}
