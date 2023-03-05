#!/bin/zsh

tableland create "owner_name text not null, area text not null, value integer not null" --prefix homes
tableland write --file homes.sql

tableland create "name text not null, sex text not null, age integer not null" --prefix people
tableland write --file people.sql

tableland create "name text not null, type text not null, owner_name text not null" --prefix pets
tableland write --file pets.sql

tableland create "id text, first_name text, last_name text, birth_year integer, sentence_start integer, sentence_end integer" --prefix inmates
tableland write --file inmates.sql

tableland create "bioguide_id text, position text, state text, party text, first_name text, last_name text, birth_year integer, service_start integer, service_end integer" --prefix politicians
tableland write --file politicians.sql

tableland create "id integer not null primary key, name text not null, health int not null" --prefix players
tableland write --file players.sql