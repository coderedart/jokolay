/*
api->builder
    apikey()
    schema
    endpoints()

enum endpoint
*/

pub mod end_point;
const API_BASE_URL: &str = "https://api.guildwars2.com";
const API_BASE_V2_URL: &str = const_format::concatcp!(API_BASE_URL, "/v2");
// https://wiki.guildwars2.com/wiki/API:Changelog
#[allow(unused)]
const LATEST_SCHEMA: &str = "2021-04-06T21:00:00.000Z";
// Make sure to set the following options when you create client
// user_agent
// https only
//
