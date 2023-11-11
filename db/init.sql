create
    extension if not exists "uuid-ossp";
create table if not exists "users"
(
    user_id   uuid        not null default uuid_generate_v4() primary key,
    user_name varchar(30) not null unique,
    user_pwd  varchar     not null,
    phone     varchar(20) not null unique,
    --是否为球场管理员
    is_admin  bool        not null
);

-----------------------------------------------
create table if not exists "courts"
(
    court_id       uuid         not null default uuid_generate_v4() primary key,
    admin_id       uuid         not null references users (user_id),
    --球场名称
    court_name     varchar(100) not null,
    --球场标签
    label          varchar(100) not null,
    --球场位置
    location       varchar(300) not null,
    price_per_hour float8       not null check ( price_per_hour > 0 ),
    open_time      time         not null,
    close_time     time         not null,
    check (open_time < close_time),
    unique (admin_id, court_name)
);
create index on courts (admin_id, court_id);
-----------------------------------------------
create table if not exists "orders"
(
    order_id    uuid primary key                  not null default uuid_generate_v4(),
    user_id     uuid references users (user_id)   not null,
    court_id    uuid references courts (court_id) not null,
    --订单发出时间
    create_time timestamp without time zone       not null default now(),
    --订单开始时间
    apt_start   timestamp without time zone       not null,
    --订单结束时间
    apt_end     timestamp without time zone       not null,
    cost        float8                            not null check ( cost > 0 ),
    check ( create_time < apt_start ),
    check ( apt_start < apt_end )
);
create index on orders (user_id);
create index on orders (court_id);