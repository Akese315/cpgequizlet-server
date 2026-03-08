pub fn read_quiz_datas(dir_path: &str) -> Vec<crate::models::JsonQuiz> {
    let mut all_quizzes = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(contents) = std::fs::read_to_string(&path) {
                    if let Ok(quizzes) =
                        serde_json::from_str::<Vec<crate::models::JsonQuiz>>(&contents)
                    {
                        all_quizzes.extend(quizzes);
                    } else {
                        println!("Failed to parse JSON in file: {:?}", path);
                    }
                }
            }
        }
    } else {
        println!("Failed to read directory: {}", dir_path);
    }

    all_quizzes
}
