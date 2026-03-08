use crate::models::{NewQuiz, NewQuizImage, NewUser, QuizImage, Quizs, Users};
use crate::schema::quiz_images::dsl::*;
use crate::schema::quizs::dsl::*;
use crate::schema::users::dsl::{id as user_id, *};
use diesel::dsl::sql;
use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenvy::dotenv;
use std::env;

pub type DbPool = r2d2::Pool<ConnectionManager<MysqlConnection>>;

pub fn establish_pool() -> DbPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<MysqlConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Impossible de créer le pool de connexions.")
}

pub fn get_image_by_url(
    conn: &mut MysqlConnection,
    url: &str,
) -> Result<Option<QuizImage>, diesel::result::Error> {
    quiz_images
        .filter(image_url.eq(url))
        .first::<QuizImage>(conn)
        .optional()
}

pub fn insert_quiz_image(
    conn: &mut MysqlConnection,
    new_quiz_image: NewQuizImage,
) -> Result<(), diesel::result::Error> {
    diesel::insert_into(quiz_images)
        .values(&new_quiz_image)
        .execute(conn)?;
    Ok(())
}

pub fn insert_quiz(
    conn: &mut MysqlConnection,
    new_quiz: NewQuiz,
) -> Result<(), diesel::result::Error> {
    diesel::insert_into(quizs).values(&new_quiz).execute(conn)?;
    Ok(())
}

pub fn get_random_quiz_uuid(
    conn: &mut MysqlConnection,
    subject_theme: Option<String>,
) -> Result<Quizs, diesel::result::Error> {
    let uuid_string = if let Some(t) = subject_theme {
        quizs
            .filter(subject.eq(t))
            .order(sql::<diesel::sql_types::Integer>("RAND()"))
            .first::<Quizs>(conn)
            .map(|quiz_record| quiz_record.uuid)?
    } else {
        quizs
            .order(sql::<diesel::sql_types::Integer>("RAND()"))
            .first::<Quizs>(conn)
            .map(|quiz_record| quiz_record.uuid)?
    };

    get_quiz_by_uuid(conn, uuid_string)
}

pub fn select_users(conn: &mut MysqlConnection) -> Result<Vec<Users>, diesel::result::Error> {
    users.load::<Users>(conn)
}

pub fn select_subjects(conn: &mut MysqlConnection) -> Result<Vec<String>, diesel::result::Error> {
    quizs.select(subject).distinct().load(conn)
}

pub fn select_themes(
    conn: &mut MysqlConnection,
    subject_str: &str,
) -> Result<Vec<String>, diesel::result::Error> {
    quizs
        .select(theme)
        .filter(subject.eq(subject_str))
        .distinct()
        .load(conn)
}

pub fn select_chapters(
    conn: &mut MysqlConnection,
    subject_str: &str,
) -> Result<Vec<String>, diesel::result::Error> {
    quizs
        .select(chapter)
        .filter(subject.eq(subject_str))
        .distinct()
        .load(conn)
}

pub fn get_quiz_by_uuid(
    conn: &mut MysqlConnection,
    uuid_string: String,
) -> Result<Quizs, diesel::result::Error> {
    quizs
        .filter(crate::schema::quizs::dsl::uuid.eq(uuid_string))
        .first::<Quizs>(conn)
}

pub fn is_database_empty(conn: &mut MysqlConnection) -> Result<bool, diesel::result::Error> {
    let result = quizs.first::<Quizs>(conn).optional()?;
    Ok(result.is_none())
}

pub fn insert_user(
    conn: &mut MysqlConnection,
    new_user: NewUser,
) -> Result<(), diesel::result::Error> {
    diesel::insert_into(users).values(&new_user).execute(conn)?;
    Ok(())
}

pub fn get_user_by_id(
    conn: &mut MysqlConnection,
    id_string: &str,
) -> Result<Users, diesel::result::Error> {
    users.filter(user_id.eq(id_string)).first::<Users>(conn)
}

pub fn is_email_already_used(
    conn: &mut MysqlConnection,
    email_string: &str,
) -> Result<bool, diesel::result::Error> {
    let result = users
        .filter(email.eq(email_string))
        .first::<Users>(conn)
        .optional()?;
    Ok(result.is_some())
}

pub fn get_user_email_or_pseudo(
    conn: &mut MysqlConnection,
    email_str: Option<String>,
    username_str: Option<String>,
) -> Result<Users, diesel::result::Error> {
    match (email_str, username_str) {
        (Some(e), Some(u)) => users
            .filter(email.eq(e).or(username.eq(u)))
            .first::<Users>(conn),
        (Some(e), None) => users.filter(email.eq(e)).first::<Users>(conn),
        (None, Some(u)) => users.filter(username.eq(u)).first::<Users>(conn),
        (None, None) => Err(diesel::result::Error::NotFound),
    }
}

pub fn update_user_token(
    conn: &mut MysqlConnection,
    id_string: &str,
    token_str: &str,
) -> Result<(), diesel::result::Error> {
    diesel::update(users.filter(user_id.eq(id_string)))
        .set(token.eq(token_str))
        .execute(conn)?;
    Ok(())
}

pub fn is_token_valid(
    conn: &mut MysqlConnection,
    token_str: &str,
    id_string: &str,
) -> Result<bool, diesel::result::Error> {
    let result = users
        .filter(user_id.eq(id_string))
        .filter(token.eq(token_str))
        .first::<Users>(conn)
        .optional();

    if result.is_err() {
        return Ok(false);
    }

    let uuid_obj = match ::uuid::Uuid::parse_str(token_str) {
        Ok(u) => u,
        Err(_) => return Ok(false),
    };
    let timestamp = match uuid_obj.get_timestamp() {
        Some(t) => t,
        None => return Ok(false),
    };
    let (secs, nanos) = timestamp.to_unix();
    let timestamp_object = chrono::DateTime::from_timestamp(secs as i64, nanos)
        .unwrap()
        .naive_utc();

    let now = chrono::Utc::now().naive_utc();
    if timestamp_object < now - chrono::Duration::hours(1) {
        return Ok(false);
    }

    Ok(true)
}
