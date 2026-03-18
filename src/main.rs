pub mod constants;
pub mod database;
pub mod folder_manager;
pub mod models;
pub mod schema;
pub mod tools;
use crate::database::{
    DbPool, establish_pool, get_image_by_url, get_quiz_by_uuid, get_random_quiz_uuid,
    get_user_by_id, get_user_email_or_pseudo, insert_quiz, insert_quiz_image, insert_user,
    is_database_empty, is_email_already_used, is_token_valid, select_chapters, select_subjects,
    select_themes, update_user_token,
};

use actix_files::{Files, NamedFile};

use crate::models::{
    GetChapterQuery, GetThemeQuery, LoginForm, NewQuiz, NewQuizImage, NewUser, QuizParams,
    RegisterForm, UploadQuizForm, UserInfoQuery,
};
use actix_cors::Cors;
use actix_multipart::form::MultipartForm;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};
use folder_manager::{add_image, download_and_save_image, read_image};

use tools::read_quiz_datas;
use uuid::Uuid;

async fn init_database(pool: &DbPool) {
    let mut conn = pool.get().expect("Failed to get DB connection");

    let is_empty = is_database_empty(&mut conn).unwrap_or(false);

    if is_empty {
        println!(
            "Database is empty, initializing from directory {}...",
            crate::constants::DATA_DIRECTORY
        );
        let path = crate::constants::DATA_DIRECTORY;

        let quizzes = read_quiz_datas(path);

        if quizzes.is_empty() {
            println!("No quizzes found in data directory. Skipping initialization.");
            return;
        }

        for quiz_data in quizzes {
            let uuid = Uuid::now_v7().to_string();
            let mut image_path = None;

            if let Some(url) = quiz_data.image_path {
                let image = get_image_by_url(&mut conn, &url).unwrap_or(None);

                if image.is_some() {
                    image_path = Some(image.unwrap().image_path);
                } else {
                    match download_and_save_image(&url, &uuid).await {
                        Ok(path) => {
                            image_path = Some(path);
                        }
                        Err(e) => {
                            println!("Failed to download image from {}: {}", url, e);
                        }
                    }
                }
                let clone_image_path = image_path.clone();
                match clone_image_path {
                    Some(path) => {
                        let new_image = NewQuizImage {
                            quiz_id: uuid.clone(),
                            image_path: path.clone(),
                            image_url: url.clone(),
                        };

                        match insert_quiz_image(&mut conn, new_image) {
                            Ok(_) => {}
                            Err(e) => {
                                println!("Failed to insert image: {}", e);
                            }
                        }
                    }
                    None => {}
                }
            }

            let new_quiz = NewQuiz {
                uuid: uuid.clone(),
                image_path: image_path.clone(),
                theme: quiz_data.theme,
                question: quiz_data.question,
                answers: quiz_data.answers.join("||||"),
                difficulty: quiz_data.difficulty,
                subject: quiz_data.subject,
                correct_answer_index: quiz_data.correct_answer_index,
                chapter: quiz_data.chapter,
                user_id: String::from("Native"),
            };

            if let Err(e) = insert_quiz(&mut conn, new_quiz) {
                println!("Failed to insert quiz: {}", e);
            } else {
                println!("Inserted quiz: {}", uuid);
            }
        }
        println!("Database initialization complete.");
    }
}

#[post("/auth/register")]
async fn register(pool: web::Data<DbPool>, form: web::Json<RegisterForm>) -> impl Responder {
    let mut conn = pool.get().unwrap();
    let email = form.email.clone();
    let password = form.password_hash.clone();
    let username = form.username.clone();
    let first_name = form.first_name.clone();
    let last_name = form.last_name.clone();

    if email.is_empty()
        || username.is_empty()
        || first_name.is_empty()
        || last_name.is_empty()
        || password.is_empty()
    {
        return HttpResponse::BadRequest().body("All fields are required");
    }

    match is_email_already_used(&mut conn, &email) {
        Ok(true) => return HttpResponse::Conflict().body("Email already in use"),
        Ok(false) => {} // Email is not used, continue
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Database error: {}", e));
        }
    }

    let new_token = Uuid::now_v7().to_string();
    let user_id = Uuid::now_v7().to_string();

    let new_user = NewUser {
        id: user_id.clone(),
        username,
        email,
        password_hash: password,
        first_name,
        last_name,
        token: Some(new_token.clone()),
        role: String::from("student"),
    };

    match insert_user(&mut conn, new_user) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "user_token": new_token,
            "user_id": user_id
        })),

        Err(e) => HttpResponse::InternalServerError().body(format!("Failed to register: {}", e)),
    }
}

#[post("/auth/login")]
async fn login(pool: web::Data<DbPool>, form: web::Json<LoginForm>) -> impl Responder {
    let mut conn = pool.get().unwrap();
    let _email = form.email.clone();
    let _password = form.password.clone();
    let _username = form.username.clone();

    if _email.is_none() && _username.is_none() {
        return HttpResponse::BadRequest().body("Email or username is required");
    }

    let user = match get_user_email_or_pseudo(&mut conn, _email, _username) {
        Ok(user) => user,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to get users"),
    };

    if user.password_hash.eq(&_password) {
        let new_token = Uuid::now_v7().to_string();

        match update_user_token(&mut conn, &user.id, &new_token) {
            Ok(_) => {
                return HttpResponse::Ok().json(serde_json::json!({
                    "user_token": new_token,
                    "user_id": user.id
                }));
            }
            Err(_) => return HttpResponse::InternalServerError().body("Failed to update token"),
        }
    }

    HttpResponse::Unauthorized().body("Wrong Password")
}

#[get("/subjects")]
async fn get_subjects(pool: web::Data<DbPool>) -> impl Responder {
    let mut conn = pool.get().unwrap();
    let subjects = match select_subjects(&mut conn) {
        Ok(subjects) => subjects,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to get subjects"),
    };
    HttpResponse::Ok().json(subjects)
}

#[post("/user/info")]
async fn get_user_info(
    pool: web::Data<DbPool>,
    auth_info: web::Json<UserInfoQuery>,
) -> impl Responder {
    let mut conn = pool.get().unwrap();
    let _user_token = auth_info.user_token.clone();
    let _user_id = auth_info.user_id.clone();

    match is_token_valid(&mut conn, &_user_token, &_user_id) {
        Ok(_) => Some(true),
        Err(_) => return HttpResponse::Unauthorized().body("Token Expired or incorrect"),
    };

    let user_info = match get_user_by_id(&mut conn, &_user_id) {
        Ok(user_info) => user_info,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to get user infos"),
    };

    return HttpResponse::Ok().json(serde_json::json!({
        "user_token": user_info.token,
        "user_id": user_info.id,
        "first_name" : user_info.first_name,
        "last_name" : user_info.last_name,
        "role": user_info.role,
        "email": user_info.email,
        "username":user_info.username
    }));
}

#[get("/themes")]
async fn get_themes(pool: web::Data<DbPool>, query: web::Query<GetThemeQuery>) -> impl Responder {
    let mut conn = pool.get().unwrap();
    let query_data = query.into_inner();

    if query_data.subject.is_none() {
        return HttpResponse::BadRequest().body("Please provide a subject");
    }

    let subject = query_data.subject.unwrap().replace('_', " ");

    let themes = match select_themes(&mut conn, &subject) {
        Ok(themes) => themes,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to get themes"),
    };
    HttpResponse::Ok().json(themes)
}

#[get("/chapters")]
async fn get_chapters(
    pool: web::Data<DbPool>,
    query: web::Query<GetChapterQuery>,
) -> impl Responder {
    let mut conn = pool.get().unwrap();
    let query_data = query.into_inner();

    let subject_clean = query_data.subject.replace('_', " ");
    let chapters = match select_chapters(&mut conn, &subject_clean) {
        Ok(chapters) => chapters,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to get chapters"),
    };
    HttpResponse::Ok().json(chapters)
}

#[get("/quiz")]
async fn get_quiz(query: web::Query<QuizParams>, pool: web::Data<DbPool>) -> impl Responder {
    let mut conn = pool.get().unwrap();
    let query_data = query.into_inner();
    println!(
        "ID: {:?}, Subject: {:?} Chapter: {:?}",
        query_data.id, query_data.subject, query_data.chapter
    );
    let result = if let Some(target_id) = query_data.id {
        // 1. Si on a un ID, on cherche par ID
        get_quiz_by_uuid(&mut conn, target_id)
    } else {
        // 2 & 3. Si on a un thème ou chapitre, on prend un random par ce filtre, sinon un random total
        let subject_clean = query_data.subject.map(|s| s.replace('_', " "));
        let chapter_clean = query_data.chapter.map(|c| c.replace('_', " "));
        get_random_quiz_uuid(&mut conn, subject_clean, chapter_clean)
    };

    match result {
        Ok(u) => HttpResponse::Ok().json(serde_json::json!({
            "uuid": u.uuid,
            "theme": u.theme,
            "question": u.question,
            "answers": u.answers.split("||||").map(|s| s.to_string()).collect::<Vec<String>>(),
            "correct_answer_index": u.correct_answer_index,
            "difficulty": u.difficulty,
            "subject": u.subject,
            "chapter": u.chapter,
        })),
        Err(_) => HttpResponse::NotFound().body("Aucun quiz correspondant"),
    }
}

#[get("/quiz/image/{uuid}")]
async fn get_image(uuid: web::Path<String>, pool: web::Data<DbPool>) -> impl Responder {
    let mut conn = pool.get().unwrap();
    let image_path = match get_quiz_by_uuid(&mut conn, uuid.into_inner()) {
        Ok(quiz) => match quiz.image_path {
            Some(path) => path,
            None => return HttpResponse::NotFound().body("Image not found"),
        },
        Err(_) => return HttpResponse::NotFound().body("Image not found"),
    };

    match read_image(&image_path) {
        Ok(image) => HttpResponse::Ok().content_type("image/png").body(image),
        Err(_) => HttpResponse::NotFound().body("Image not found"),
    }
}

#[post("/quiz")]
async fn create_quiz(
    MultipartForm(form): MultipartForm<UploadQuizForm>,
    pool: web::Data<DbPool>,
) -> impl Responder {
    let _file = form.image;
    let uuid = Uuid::now_v7().to_string();
    let mut conn = pool.get().unwrap();

    let theme = form.theme.into_inner();
    let question = form.question.into_inner();
    let subject = form.subject.into_inner();
    let chapter = form.chapter.into_inner();
    let user_id = form.user_id.into_inner();
    let user_token = form.user_token.into_inner();

    let _answers = form
        .answers
        .into_iter()
        .map(|x| x.into_inner())
        .collect::<Vec<String>>();
    let _correct_answer_index = form.correct_answer_index.into_inner();
    let _difficulty = form.difficulty.into_inner();

    if theme.is_empty()
        || question.is_empty()
        || _answers.is_empty()
        || _correct_answer_index < 0
        || _difficulty < 0
        || subject.is_empty()
        || chapter.is_empty()
    {
        return HttpResponse::BadRequest()
            .body("Theme, question, answers and correct answer are required");
    }

    match is_token_valid(&mut conn, &user_token, &user_id) {
        Ok(_) => Some(true),
        Err(_) => return HttpResponse::Unauthorized().body("Token Expired or incorrect"),
    };

    let user_info = match get_user_by_id(&mut conn, &user_id) {
        Ok(user_info) => user_info,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to get user infos"),
    };

    if user_info.role == String::from("student") {
        return HttpResponse::Unauthorized().body("Unauthorized for students");
    }

    let mut image_path = None;

    if _file.is_some() {
        match add_image(_file.unwrap(), &uuid) {
            Ok(path) => image_path = Some(path),
            Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
        }
    }

    let new_quiz = NewQuiz {
        uuid: uuid.clone(),
        image_path,
        theme: theme.clone(),
        question: question.clone(),
        answers: _answers.join("|||"),
        difficulty: _difficulty.clone(),
        subject: subject.clone(),
        correct_answer_index: _correct_answer_index.clone(),
        chapter: chapter.clone(),
        user_id: user_id.clone(),
    };

    match insert_quiz(&mut conn, new_quiz) {
        Ok(_) => HttpResponse::Ok().body(format!(
            "Created Quiz with theme: {}, question: {}, answers: {}, correct_answer_index: {}, difficulty: {}",
            theme, question, _answers.join(","), _correct_answer_index, _difficulty
        )),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = establish_pool();

    init_database(&pool).await;

    println!("Server started on http://127.0.0.1:8080");
    HttpServer::new(move || {
        // On ajoute 'move' pour capturer le pool
        App::new()
            .app_data(web::Data::new(pool.clone())) // Injection du State
            .wrap(Cors::permissive())
            .service(
                web::scope("/api")
                    .service(get_user_info)
                    .service(register)
                    .service(login)
                    .service(get_quiz)
                    .service(create_quiz)
                    .service(get_image)
                    .service(get_subjects)
                    .service(get_themes)
                    .service(get_chapters),
            )
            .service(Files::new("/", "./client").index_file("index.html"))
            .default_service(web::route().to(|| async { NamedFile::open("./client/index.html") }))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
