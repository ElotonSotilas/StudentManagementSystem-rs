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
                "id" INTEGER NOT NULL UNIQUE,
                "username" TEXT NOT NULL,
                "password" TEXT NOT NULL,
                "email" TEXT NOT NULL UNIQUE,
                "phone" TEXT NOT NULL,
                "verified" BOOLEAN NOT NULL,
                "suspended" BOOLEAN NOT NULL,
                "forcenewpw" BOOLEAN NOT NULL,
                "role" TEXT NOT NULL,
                PRIMARY KEY("id" AUTOINCREMENT)
            );

            CREATE TABLE "STUDENT_ACCOUNT" (
                "id" INTEGER NOT NULL UNIQUE,
                "student_id" INTEGER NOT NULL UNIQUE,
                "advisor_id" INTEGER NOT NULL,
                "discipline" TEXT NOT NULL,
                "enrollment" TEXT NOT NULL,
                "cgpa" REAL NOT NULL,
                "can_grad" BOOLEAN NOT NULL CHECK ("cgpa" > 2.0 AND "cum_credit" >= 120),
                "cur_credit" INTEGER NOT NULL,
                "cum_credit" INTEGER NOT NULL,
                FOREIGN KEY ("student_id") REFERENCES "USERS"("id"),
                FOREIGN KEY ("advisor_id") REFERENCES "USERS"("id"),
                PRIMARY KEY("id" AUTOINCREMENT)
            );
            
            CREATE TABLE "TEACHER_ACCOUNT" (
                "id" INTEGER NOT NULL,
                "teacher_id" INTEGER NOT NULL,
                "dept_id" INTEGER NOT NULL,
                "dept" TEXT NOT NULL,
                FOREIGN KEY ("teacher_id") REFERENCES "USERS"("id"),
                FOREIGN KEY ("dept_id") REFERENCES "DEPARTMENTS"("id"),
                PRIMARY KEY("id" AUTOINCREMENT)
            );
            
            CREATE TABLE "COURSES" (
                "id" INTEGER NOT NULL,
                "teacher_id" INTEGER NOT NULL,
                "course" TEXT NOT NULL,
                "cr_cost" INTEGER NOT NULL,
                FOREIGN KEY ("teacher_id") REFERENCES USERS("id"),
                PRIMARY KEY("id" AUTOINCREMENT)
            );
            
            CREATE TABLE "STUDENT_COURSES" (
                "student_id" INTEGER NOT NULL,
                "course_id" INTEGER NOT NULL,
                "grade" REAL NOT NULL,
                "semester" TEXT NOT NULL,
                FOREIGN KEY ("student_id") REFERENCES USERS("id"),
                FOREIGN KEY ("course_id") REFERENCES COURSES("id")
            );
            
            CREATE TABLE "DEPARTMENTS" (
                "id" INTEGER NOT NULL,
                "dept_head" INTEGER NOT NULL,
                "name" TEXT NOT NULL,
                FOREIGN KEY ("dept_head") REFERENCES TEACHER_ACCOUNT("id"),
                PRIMARY KEY("id" AUTOINCREMENT)
            );
            COMMIT;
            "#,
            )?;
        }

        Ok(self)
    }
}
