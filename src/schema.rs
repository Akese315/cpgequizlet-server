// @generated automatically by Diesel CLI.

diesel::table! {
    quiz_images (id) {
        id -> Integer,
        #[max_length = 255]
        quiz_id -> Varchar,
        #[max_length = 255]
        image_path -> Varchar,
        image_url -> Text,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    quizs (id) {
        id -> Integer,
        #[max_length = 255]
        uuid -> Varchar,
        #[max_length = 255]
        image_path -> Nullable<Varchar>,
        #[max_length = 255]
        theme -> Varchar,
        question -> Text,
        answers -> Text,
        created_at -> Timestamp,
        difficulty -> Integer,
        #[max_length = 255]
        chapter -> Varchar,
        correct_answer_index -> Integer,
        #[max_length = 255]
        subject -> Varchar,
        #[max_length = 36]
        user_id -> Varchar,
    }
}

diesel::table! {
    users (id) {
        #[max_length = 36]
        id -> Char,
        #[max_length = 255]
        username -> Varchar,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        password_hash -> Varchar,
        created_at -> Timestamp,
        #[max_length = 255]
        token -> Nullable<Varchar>,
        #[max_length = 255]
        first_name -> Varchar,
        #[max_length = 255]
        last_name -> Varchar,
        #[max_length = 32]
        role -> Varchar,
    }
}

diesel::allow_tables_to_appear_in_same_query!(quiz_images, quizs, users,);
