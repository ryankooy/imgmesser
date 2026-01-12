--CREATE USER im_admin WITH PASSWORD 'xyz'; -- Uncomment and replace dummy password
CREATE DATABASE imgmesser OWNER im_admin;
GRANT ALL PRIVILEGES ON DATABASE imgmesser TO im_admin;
