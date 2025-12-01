ImgMesser
=========

ImgMesser is a website intended for uploading and editing photos. It's written in Rust (backend) and Svelte (frontend). This project is a WIP, and much of its primary functionality is yet to be implemented.

### Current functionality:
* User registration, login, and logout
* Image upload
* Gallery view of uploaded images
* Download an uploaded image

Basic Run Instructions
----------------------

#### API

One time (from repo root):
```
sudo apt install postgresql
cargo install sqlx-cli
sudo service postgresql start
cargo sqlx prepare --workspace
cargo build

```

Each run (from repo root):
```
cargo run
```

#### Client

One time:
```
cd client
npm install
```

Each run (from `client` directory):
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
