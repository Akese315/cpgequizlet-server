-- This file should undo anything in `up.sql`
ALTER TABLE quizs DROP COLUMN difficulty;
ALTER TABLE quizs DROP COLUMN chapter;
ALTER TABLE quizs DROP COLUMN correct_answer_index;
ALTER TABLE quizs ADD COLUMN correct_answer VARCHAR(255) NOT NULL;