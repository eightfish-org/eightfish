-- CREATE DATABASE spin_dev;

-- \c spin_dev;

CREATE TABLE article (
	id varchar PRIMARY KEY,
	title varchar(80) NOT NULL,
	content text NOT NULL,
	authorname varchar(40) NOT NULL
);

CREATE TABLE article_idhash (
	id varchar PRIMARY KEY,
	hash varchar NOT NULL
);


