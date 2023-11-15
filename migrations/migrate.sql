create table feeds (
    id bigserial primary key,
    title text not null,
    feed_uri text unique not null,
    site_uri text,
    updated_at timestamp with time zone not null
);

create table items (
    id bigserial primary key,
    feed_id bigint not null references feeds(id) on delete cascade,
    hash text unique not null,
    title text not null,
    author text not null,
    content text not null,
    created_at timestamp with time zone not null,
    updated_at timestamp with time zone not null,
    read bool not null default false,
    star bool not null default false
);

create table tags (
    id bigserial primary key,
    name text unique not null
);

insert into tags(name) values ('default');

create table taggings (
    feed_id bigint references feeds(id) on delete cascade,
    tag_id bigint references tags(id) on delete cascade,
    unique (feed_id, tag_id)
);

create table session (
    password text not null,
    token text not null
);