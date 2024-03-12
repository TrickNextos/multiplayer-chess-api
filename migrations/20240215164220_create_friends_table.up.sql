CREATE TABLE Friends(
  id int primary key auto_increment,
  friend1 int,
  friend2 int,

  foreign key (friend1) references User(id),
  foreign key (friend2) references User(id)
);

