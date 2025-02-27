-- Your SQL goes here
CREATE TABLE users (
   id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
   username VARCHAR(255) NOT NULL UNIQUE,
   password VARCHAR(255) NULL DEFAULT NULL
);

INSERT INTO users (username, password)
VALUES ('admin', 'password');