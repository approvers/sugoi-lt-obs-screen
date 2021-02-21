-- Your SQL goes here

create table presentors (
    presentor_id integer primary key autoincrement not null,
    display_name text not null,
    twitter_id text not null,
    icon text not null
);

create table presentations (
    presentation_id integer primary key autoincrement not null,
    title text not null,
    presentor_id integer not null,
    foreign key(presentor_id) references presentors(presentor_id)
);
