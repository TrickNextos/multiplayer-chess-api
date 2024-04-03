create table Games(
  id int primary key auto_increment,
  white int not null,
  black int not null,
  game_file_uuid varchar(64) not null,
  num_of_moves int,
  win enum('white', 'black', 'draw') not null,
  win_description enum('checkmate', 'time', 'resign'),
  singleplayer bool not null,
  
  played_at timestamp default CURRENT_TIMESTAMP,

  foreign key (white) references User(id),
  foreign key (black) references User(id)
);
