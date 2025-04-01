-- Your SQL goes here
CREATE TABLE users (
   id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
   email VARCHAR(255) NOT NULL UNIQUE,
   password VARCHAR(255) NULL DEFAULT NULL,
   locale VARCHAR(255) NULL DEFAULT NULL
);

INSERT INTO users (email, password)
# Password: "password";
VALUES ('admin@admin.example', '$argon2id$v=19$m=19456,t=2,p=1$iLHsKp9nVoAvoKgnCFyQGA$VfmHegV5Pb0tyZIQKqgzWmctmJ1mmFuigr4H4HYZkwY');