ImgMesser
=========

ImgMesser is a website intended for uploading and editing photos.
It's written in Rust (backend) and Svelte (frontend).
This project is a WIP, and much of its primary functionality is
yet to be implemented.

### Current functionality:
* User registration, login, and logout
* Image upload
* Gallery view of uploaded images
* Download an uploaded image

Local Run Instructions
----------------------

#### API

These instructions assume that one has an AWS account with an
S3 bucket (the name of which can be configured in `.env.dev`)
as well as the AWS CLI tool and an SSO profile.

**One time setup (from repo root):**
- In `api/db/db/create.sql`, uncomment the user creation statement
and update the password value.
- Create the user and database:
    ```
    sudo service postgresql start
    psql -h localhost -U postgres < api/db/db/create.sql
    ```
- Install `sqlx`:
    ```
    cargo install sqlx-cli
    ```
- Create `sqlx` migrations:
    ```
    cd api/db
    mkdir migrations
    sqlx migrate add -r create_schema
    cat db/schema.up.sql > migrations/[YYYYmmddHHMMSS]_create_schema.up.sql
    cat db/schema.down.sql > migrations/[YYYYmmddHHMMSS]_create_schema.down.sql
    source ../../.env.dev
    DATABASE_URL="${DATABASE_URL}" sqlx migrate run
    ```
- Save query metadata to `./sqlx/`.
    ```
    cd ../..
    DATABASE_URL="${DATABASE_URL}" cargo sqlx prepare --workspace
    ```
- Build the project.
    ```
    cargo build
    ```

**Each run (from repo root):**
```
ENV=dev cargo run
```

#### Client

**One time:**
```
cd client
npm install
```

**Each run (from `client` directory):**
```
npm run dev
```

To Do
-----

- [ ] Basic image editing
- [ ] Advanced image processing

Completed Tasks
---------------

- [x] Create Rust backend with Axum server
- [x] Create Svelte/Vite frontend
- [x] Use AWS S3 for image storage
- [x] Use JWT for user authentication
- [x] Store access and refresh tokens client-side using a service worker and IndexedDB
- [x] Use PostgreSQL and Rust crate `sqlx` for storing user, image, and refresh token data
- [x] Image deletion
- [x] Image reversion

