ImgMesser
=========

ImgMesser is a website intended for uploading, viewing, editing,
and transforming photos.
It's written in Rust (backend) and Svelte (frontend) and uses
PostgreSQL for the database, AWS S3 for image storage, and JWT
for user authentication.
This project is a WIP, and much of its functionality is
yet to be implemented.

### Current functionality:
* User registration, login, and logout
* Image upload
* Gallery view of uploaded images
* Download an uploaded image
* View, pan, and zoom in/out on an image
* Delete an image
* Rename an image
* Save a copy of an image
* Apply transformations such as rotating, cropping, resizing,
and applying grayscale and/or sepia filters
* Undo/redo changes
* View, save, and delete different versions of an image

To Do
-----

- [ ] Implement image overlays/blends
- [ ] Implement more image lighting and color transformations
- [ ] Revamp user login

Assorted Completed Tasks
---------------

- [x] Create Rust backend with Axum server
- [x] Create Svelte/Vite frontend
- [x] Use AWS S3 for image storage
- [x] Use JWT for user authentication
- [x] Store access and refresh tokens client-side using a service worker and IndexedDB
- [x] Use PostgreSQL and Rust crate `sqlx` for storing user, image, and refresh token data
- [x] Image deletion
- [x] Image reversion
- [x] Web deployment using Docker

