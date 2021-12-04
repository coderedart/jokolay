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

// Make sure to set the following options when you create client
// user_agent
// https only
//
