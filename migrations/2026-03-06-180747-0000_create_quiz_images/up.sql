-- Your SQL goes here
CREATE TABLE quiz_images (
    id INTEGER AUTO_INCREMENT PRIMARY KEY NOT NULL,
    quiz_id varchar(255) NOT NULL,
    image_path VARCHAR(255) NOT NULL,
    image_url TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
