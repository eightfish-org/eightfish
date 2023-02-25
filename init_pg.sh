sudo -u postgres psql -c "alter user postgres with password '123456';"
sudo -u postgres psql -c "create database spin_dev;"
sudo -u postgres PGPASSWORD=123456 psql -h 127.0.0.1 -d spin_dev -c "CREATE TABLE article (
id varchar PRIMARY KEY,
title varchar(80) NOT NULL,
content text NOT NULL,
authorname varchar(40) NOT NULL
);
CREATE TABLE article_idhash (
id varchar PRIMARY KEY,
hash varchar NOT NULL
);"
