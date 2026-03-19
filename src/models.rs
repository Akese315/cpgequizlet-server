use actix_multipart::form::{MultipartForm, tempfile::TempFile, text::Text};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::quizs)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Quizs {
    pub id: i32,
    pub uuid: String,
    pub image_path: Option<String>,
    pub theme: String,
    pub question: String,
    pub answers: String,
    pub created_at: NaiveDateTime,
    pub difficulty: i32,
    pub chapter: String,
    pub correct_answer_index: i32,
    pub subject: String,
    pub user_id: String,
    pub explication: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::quiz_images)]
pub struct NewQuizImage {
    pub quiz_id: String,
    pub image_path: String,
    pub image_url: String,
}

#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::quiz_images)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct QuizImage {
    pub id: i32,
    pub quiz_id: String,
    pub image_path: String,
    pub image_url: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::quizs)]
pub struct NewQuiz {
    pub uuid: String,
    pub image_path: Option<String>,
    pub theme: String,
    pub question: String,
    pub answers: String,
    pub difficulty: i32,
    pub correct_answer_index: i32,
    pub subject: String,
    pub chapter: String,
    pub user_id: String,
    pub explication: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub token: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
}

#[derive(Queryable, Selectable, Deserialize, Serialize)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Users {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
    pub token: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
}

#[derive(Debug, MultipartForm)]
pub struct UploadQuizForm {
    #[multipart(limit = "10MB")]
    pub image: Option<TempFile>,
    pub theme: Text<String>,
    pub question: Text<String>,
    pub answers: Vec<Text<String>>,
    pub correct_answer_index: Text<i32>,
    pub difficulty: Text<i32>,
    pub subject: Text<String>,
    pub chapter: Text<String>,
    pub user_id: Text<String>,
    pub user_token: Text<String>,
    pub explication: Option<Text<String>>,
}

#[derive(Deserialize)]
pub struct QuizParams {
    pub id: Option<String>,
    pub subject: Option<String>,
    pub chapter: Option<String>,
}

#[derive(Deserialize)]
pub struct JsonQuiz {
    #[serde(default)]
    pub theme: String,
    #[serde(default)]
    pub question: String,
    #[serde(default)]
    pub answers: Vec<String>,
    #[serde(default)]
    pub difficulty: i32,
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub correct_answer_index: i32,
    #[serde(default)]
    pub image_path: Option<String>,
    #[serde(default)]
    pub chapter: String,
    #[serde(default)]
    pub explication: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RegisterForm {
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub last_name: String,
    pub first_name: String,
}

#[derive(Deserialize)]
pub struct UserInfoQuery {
    pub user_id: String,
    pub user_token: String,
}
#[derive(Deserialize)]
pub struct GetThemeQuery {
    pub subject: Option<String>,
}

#[derive(Deserialize)]
pub struct GetChapterQuery {
    pub subject: String,
}
