create extension if not exists "uuid-ossp";
create table if not exists "users"
(
    id          uuid                        not null default uuid_generate_v4() primary key,
    name        varchar(100)                not null unique,
    pwd         varchar(100)                not null,
    phone       varchar(20)                 not null unique,
    role        varchar(20)                 not null default 'user',
    create_time timestamp without time zone not null default now()
);
-----------------------------------------------
create table if not exists "courts"
(
    id       uuid         not null default uuid_generate_v4() primary key,
    admin    uuid         not null references users (id),
    name     varchar(100) not null,
    location varchar(200) not null,
    class    varchar(20)  not null,
    unique (admin, name)
);
create index on courts (admin, name);
-----------------------------------------------
create table if not exists "orders"
(
    id        serial primary key          not null,
    userid    uuid references users (id)  not null,
    courtid   uuid references courts (id) not null,
    ordertime timestamp without time zone not null default now(),
    apt_start timestamp without time zone not null,
    apt_end   timestamp without time zone not null,
    remark    varchar(300),--备注
    check ( apt_start < orders.apt_end )
);
create index on orders (userid);
create index on orders (courtid);