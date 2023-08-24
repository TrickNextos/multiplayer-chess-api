-- Add up migration script here

create table User (
  id int primary key auto_increment,
  username varchar(55) unique not null,
  password varchar(55) not null,
  country varchar(55)
);
