CREATE TABLE IF NOT EXISTS user_profile (
    username text,
    password varchar(100) NOT NULL,
    objectbasepath text NOT NULL,
    PRIMARY KEY (username)
);

CREATE TABLE IF NOT EXISTS image (
    id uuid,
    name text,
    extension text,
    username text NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (username) REFERENCES user_profile (username)
);

CREATE TABLE IF NOT EXISTS image_version (
    imageid uuid,
    version text,
    ts timestamptz NOT NULL,
    latest boolean NOT NULL,
    PRIMARY KEY (imageid, version),
    FOREIGN KEY (imageid) REFERENCES image (id)
);
