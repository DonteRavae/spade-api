CREATE TABLE user_profiles(
    id VARCHAR(100) PRIMARY KEY,
    avatar TEXT,
    username VARCHAR(255) NOT NULL UNIQUE,
    last_modified TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);

CREATE TABLE expression_posts(
    id VARCHAR(255) PRIMARY KEY,
    title TEXT NOT NULL,
    subtitle TEXT,
    author VARCHAR(100) NOT NULL,
    content_type ENUM('text', 'image') NOT NULL,
    content_value TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_modified TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (author) REFERENCES user_profiles(id) ON DELETE CASCADE
);

CREATE TABLE replies (
    id VARCHAR(100) PRIMARY KEY,
    author VARCHAR(100) NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_modified TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (author) REFERENCES user_profiles(id) ON DELETE CASCADE
);

CREATE TABLE likes (
    parent_id VARCHAR(255) NOT NULL,
    author VARCHAR(100) NOT NULL
);
