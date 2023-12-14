use crate::{Context, Response};
use hyper::StatusCode;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
//use std::fs;
use serde::{Deserialize, Serialize};
//use serde_json::{Value};
use std::time::{SystemTime, UNIX_EPOCH};
use sha1::{Sha1, Digest};
use sha2::{Sha256};
use urlencoding;
use reqwest::header;
use chrono::{TimeZone, Utc};
use rsa::{RsaPrivateKey, RsaPublicKey};
use dbif::{ActorRecord};
use base64::{Engine as _, engine::{self, general_purpose}, alphabet};
use secp256k1::hashes::{sha256, Hash};
use secp256k1::Message;
use sigh::{Key, PrivateKey, SigningConfig};
use sigh::alg::RsaSha256;


//Globals ----------------------------------------------------------------------------------------------------
//TODO: These secrets need to be moved into the environment
const API_KEY: &str = "B899NK69ERMRE2M6HD3B";
const API_SECRET: &str = "J3v9m$4b6NCD9ENV4QEKYb^DnWdcGR$^Gq7#5uwS";


//Structs ----------------------------------------------------------------------------------------------------
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct Link {
    rel: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    href: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    template: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Webfinger {
    subject: String,
    aliases: Vec<String>,
    links: Vec<Link>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct PublicKey {
    id: String,
    owner: String,
    publicKeyPem: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Icon {
    r#type: String,
    mediaType: Option<String>,
    url: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct TagObject {
    id: String,
    r#type: String,
    name: Option<String>,
    updated: Option<String>,
    icon: Option<Icon>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Attachment {
    name: String,
    r#type: String,
    value: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Endpoints {
    sharedInbox: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct ActorKeys {
    pem_private_key: String,
    pem_public_key: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Actor {
    #[serde(rename = "@context", skip_deserializing)]
    at_context: Vec<String>,
    id: String,
    r#type: String,
    discoverable: bool,
    indexable: Option<bool>,
    preferredUsername: String,
    published: String,
    memorial: Option<bool>,
    devices: Option<String>,
    tag: Vec<String>,
    name: String,
    inbox: String,
    outbox: String,
    featured: String,
    followers: String,
    following: String,
    icon: Option<Icon>,
    summary: String,
    url: String,
    manuallyApprovesFollowers: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    attachment: Option<Vec<Attachment>>,
    publicKey: PublicKey,
    endpoints: Endpoints,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct InboxRequest {
    id: String,
    r#type: String,
    actor: String,
    object: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct InboxRequestAccept {
    #[serde(rename = "@context", skip_deserializing)]
    at_context: String,
    id: String,
    r#type: String,
    actor: String,
    object: InboxRequest,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct OutboxConfig {
    #[serde(rename = "@context")]
    context: String,
    id: String,
    r#type: String,
    totalItems: u64,
    first: String,
    last: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct Object {
    id: String,
    r#type: String,
    summary: Option<String>,
    inReplyTo: Option<String>,
    published: String,
    url: String,
    attributedTo: String,
    to: Vec<String>,
    cc: Option<Vec<String>>,
    sensitive: bool,
    conversation: String,
    content: String,
    attachment: Vec<String>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct Item {
    id: String,
    r#type: String,
    actor: String,
    published: String,
    directMessage: bool,
    to: Vec<String>,
    object: Object,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct OutboxPaged {
    #[serde(rename = "@context")]
    context: String,
    id: String,
    r#type: String,
    next: String,
    prev: String,
    partOf: String,
    totalItems: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    orderedItems: Option<Vec<Item>>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct Featured {
    #[serde(rename = "@context")]
    at_context: Vec<String>,
    id: String,
    r#type: String,
    totalItems: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    orderedItems: Option<Vec<FeaturedItem>>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct FeaturedItem {
    #[serde(rename = "@context")]
    at_context: Vec<String>,
    actor: String,
    attachment: Vec<String>,
    attributedTo: String,
    cc: Option<Vec<String>>,
    content: String,
    context: String,
    conversation: String,
    id: String,
    published: String,
    repliesCount: u64,
    sensitive: bool,
    source: String,
    summary: Option<String>,
    tag: Vec<String>,
    to: Vec<String>,
    r#type: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct Status {
    #[serde(rename = "@context")]
    at_context: Vec<String>,
    id: String,
    r#type: String,
    summary: Option<String>,
    inReplyTo: Option<String>,
    published: String,
    url: Option<String>,
    attributedTo: String,
    to: Vec<String>,
    cc: Option<Vec<String>>,
    sensitive: bool,
    conversation: String,
    content: String,
    attachment: Vec<String>,
    actor: String,
    tag: Vec<String>,
    replies: Option<String>,    //TODO: This should refer to some sort of Collection pub struct
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct PIFeed {
    id: u64,
    podcastGuid: String,
    medium: String,
    title: String,
    url: String,
    originalUrl: String,
    link: String,
    description: String,
    author: String,
    ownerName: String,
    image: String,
    artwork: String,
    episodeCount: u64,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct PIPodcast {
    status: String,
    feed: PIFeed,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct PIItem {
    id: u64,
    title: String,
    link: String,
    description: String,
    guid: String,
    datePublished: u64,
    datePublishedPretty: String,
    enclosureUrl: String,
    enclosureType: String,
    duration: u64,
    image: String,
    feedImage: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct PIEpisodes {
    status: String,
    items: Vec<PIItem>,
    count: u64,
}

#[derive(Debug)]
pub struct HydraError(String);

impl fmt::Display for HydraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fatal error: {}", self.0)
    }
}

impl Error for HydraError {}


//Functions --------------------------------------------------------------------------------------------------
pub async fn webfinger(ctx: Context) -> Response {

    //Get query parameters
    let params: HashMap<String, String> = ctx.req.uri().query().map(|v| {
        url::form_urlencoded::parse(v.as_bytes()).into_owned().collect()
    }).unwrap_or_else(HashMap::new);

    println!("\n\n----------");
    println!("Request: {} from: {:#?}", ctx.req.uri(), ctx.req.headers().get("user-agent"));

    //Make sure a session param was given
    let guid;
    match params.get("resource") {
        Some(resource) => {
            println!("  Id: {}\n", resource);
            let parts = resource.replace("acct:", "");
            guid = parts.split("@").next().unwrap().to_string();
        }
        None => {
            println!("Invalid resource.\n");
            return hyper::Response::builder()
                .status(StatusCode::from_u16(400).unwrap())
                .body(format!("No resource given.").into())
                .unwrap();
        }
    }

    let podcast_guid = guid.clone();

    //Lookup API of podcast
    let podcast_data: PIPodcast;
    let api_response = api_get_podcast(API_KEY, API_SECRET, &podcast_guid).await;
    match api_response {
        Ok(response_body) => {
            //eprintln!("{:#?}", response_body);
            match serde_json::from_str(response_body.as_str()) {
                Ok(data) => {
                    podcast_data = data;
                    println!("{}", podcast_data.feed.image);
                }
                Err(e) => {
                    println!("Response prep error: [{:#?}].\n", e);
                    return hyper::Response::builder()
                        .status(StatusCode::from_u16(501).unwrap())
                        .body(format!("Response prep error.").into())
                        .unwrap();
                }
            }
        }
        Err(e) => {
            println!("Response prep error: [{:#?}].\n", e);
            return hyper::Response::builder()
                .status(StatusCode::from_u16(501).unwrap())
                .body(format!("Response prep error.").into())
                .unwrap();
        }
    }

    //Construct a response
    let webfinger_data = Webfinger {
        subject: format!("acct:{}@ap.podcastindex.org", podcast_guid).to_string(),
        aliases: vec!(
            format!("https://podcastindex.org/podcast/{}", podcast_guid).to_string()
        ),
        links: vec!(
            Link {
                rel: "http://webfinger.net/rel/profile-page".to_string(),
                r#type: Some("text/html".to_string()),
                href: Some(format!("https://ap.podcastindex.org/profiles?id={}", podcast_guid).to_string()),
                template: None,
            },
            Link {
                rel: "self".to_string(),
                r#type: Some("application/activity+json".to_string()),
                href: Some(format!("https://ap.podcastindex.org/podcasts?id={}", podcast_guid).to_string()),
                template: None,
            },
            Link {
                rel: "http://webfinger.net/rel/avatar".to_string(),
                r#type: Some("image/png".to_string()),
                href: Some(format!("{}", podcast_data.feed.image).to_string()),
                template: None,
            },
            Link {
                rel: "http://ostatus.org/schema/1.0/subscribe".to_string(),
                r#type: None,
                href: None,
                template: Some("https://ap.podcastindex.org/ostatus_subscribe?acct={uri}".to_string()),
            },
        ),
    };

    let webfinger_json;
    match serde_json::to_string_pretty(&webfinger_data) {
        Ok(json_result) => {
            webfinger_json = json_result;
        }
        Err(e) => {
            println!("Response prep error: [{:#?}].\n", e);
            return hyper::Response::builder()
                .status(StatusCode::from_u16(500).unwrap())
                .body(format!("Response prep error.").into())
                .unwrap();
        }
    }

    return hyper::Response::builder()
        .status(StatusCode::OK)
        .header("Content-type", "application/json; charset=utf-8")
        .body(format!("{}", webfinger_json).into())
        .unwrap();
}

pub async fn podcasts(ctx: Context) -> Response {

    //Get query parameters
    let params: HashMap<String, String> = ctx.req.uri().query().map(|v| {
        url::form_urlencoded::parse(v.as_bytes()).into_owned().collect()
    }).unwrap_or_else(HashMap::new);

    println!("\n\n----------");
    println!("Request: {} from: {:#?}", ctx.req.uri(), ctx.req.headers().get("user-agent"));

    //Make sure a session param was given
    let guid;
    match params.get("id") {
        Some(resource) => {
            println!("  Id: {}\n", resource);
            let parts = resource.replace("acct:", "");
            guid = parts.split("@").next().unwrap().to_string();
        }
        None => {
            println!("Invalid resource.\n");
            return hyper::Response::builder()
                .status(StatusCode::from_u16(400).unwrap())
                .body(format!("No resource given.").into())
                .unwrap();
        }
    }
    let podcast_guid = guid.clone();

    //Lookup API of podcast
    let podcast_data: PIPodcast;
    let api_response = api_get_podcast(API_KEY, API_SECRET, &podcast_guid).await;
    match api_response {
        Ok(response_body) => {
            //eprintln!("{:#?}", response_body);
            match serde_json::from_str(response_body.as_str()) {
                Ok(data) => {
                    podcast_data = data;
                    println!("{}", podcast_data.feed.image);
                }
                Err(e) => {
                    println!("Response prep error: [{:#?}].\n", e);
                    return hyper::Response::builder()
                        .status(StatusCode::from_u16(501).unwrap())
                        .body(format!("Response prep error.").into())
                        .unwrap();
                }
            }
        }
        Err(e) => {
            println!("Response prep error: [{:#?}].\n", e);
            return hyper::Response::builder()
                .status(StatusCode::from_u16(501).unwrap())
                .body(format!("Response prep error.").into())
                .unwrap();
        }
    }

    //If no keypair exists, create one
    let actor_keys;
    match ap_get_actor_keys(podcast_guid.parse::<u64>().unwrap()) {
        Ok(keys) => {
            actor_keys = keys;
        }
        Err(e) => {
            return hyper::Response::builder()
                .status(StatusCode::from_u16(500).unwrap())
                .body(format!("Key error.").into())
                .unwrap();
        }
    }

    //Construct a response
    let actor_data;
    match ap_build_actor_object(podcast_data, actor_keys) {
        Ok(data) => {
            actor_data = data;
        }
        Err(e) => {
            return hyper::Response::builder()
                .status(StatusCode::from_u16(500).unwrap())
                .body(format!("Actor obect error.").into())
                .unwrap();
        }
    }
    let actor_json;
    match serde_json::to_string_pretty(&actor_data) {
        Ok(json_result) => {
            actor_json = json_result;
        }
        Err(e) => {
            println!("Response prep error: [{:#?}].\n", e);
            return hyper::Response::builder()
                .status(StatusCode::from_u16(500).unwrap())
                .body(format!("Response prep error.").into())
                .unwrap();
        }
    }

    return hyper::Response::builder()
        .status(StatusCode::OK)
        .header("Content-type", "application/activity+json; charset=utf-8")
        .body(format!("{}", actor_json).into())
        .unwrap();
}

pub async fn profiles(ctx: Context) -> Response {

    //Get query parameters
    let params: HashMap<String, String> = ctx.req.uri().query().map(|v| {
        url::form_urlencoded::parse(v.as_bytes()).into_owned().collect()
    }).unwrap_or_else(HashMap::new);

    println!("\n\n----------");
    println!("Request: {} from: {:#?}", ctx.req.uri(), ctx.req.headers().get("user-agent"));

    //Make sure a session param was given
    let guid;
    match params.get("id") {
        Some(resource) => {
            println!("  Id: {}\n", resource);
            let parts = resource.replace("acct:", "");
            guid = parts.split("@").next().unwrap().to_string();
        }
        None => {
            println!("Invalid resource.\n");
            return hyper::Response::builder()
                .status(StatusCode::from_u16(400).unwrap())
                .body(format!("No resource given.").into())
                .unwrap();
        }
    }

    let podcast_guid = guid.clone();

    //Lookup API of podcast
    let podcast_data: PIPodcast;
    let api_response = api_get_podcast(API_KEY, API_SECRET, &podcast_guid).await;
    match api_response {
        Ok(response_body) => {
            //eprintln!("{:#?}", response_body);
            match serde_json::from_str(response_body.as_str()) {
                Ok(data) => {
                    podcast_data = data;
                    println!("{}", podcast_data.feed.image);
                }
                Err(e) => {
                    println!("Response prep error: [{:#?}].\n", e);
                    return hyper::Response::builder()
                        .status(StatusCode::from_u16(501).unwrap())
                        .body(format!("Response prep error.").into())
                        .unwrap();
                }
            }
        }
        Err(e) => {
            println!("Response prep error: [{:#?}].\n", e);
            return hyper::Response::builder()
                .status(StatusCode::from_u16(501).unwrap())
                .body(format!("Response prep error.").into())
                .unwrap();
        }
    }

    //Build HTML profile page
    return hyper::Response::builder()
        .status(StatusCode::OK)
        .header("Content-type", "text/html")
        .body(
            format!("<!DOCTYPE html>
<html lang='en'>
  <head>
    <meta charset='utf-8' />
    <meta content='{}' property='og:title' />
    <meta content='{}' property='og:url' />
    <meta content='{}' property='og:description' />
    <meta content='article' property='og:type' />
    <meta content='{}' property='og:image' />
    <meta content='150' property='og:image:width' />
    <meta content='150' property='og:image:height' />
  </head>
  <body>
    Empty
  </body>
  </html>",
                    podcast_data.feed.title,
                    format!("https://ap.podcastindex.org/podcasts?id={}", podcast_guid).to_string(),
                    podcast_data.feed.description,
                    podcast_data.feed.image
            ).into()
        )
        .unwrap();
}

pub async fn outbox(ctx: Context) -> Response {

    //Get query parameters
    let params: HashMap<String, String> = ctx.req.uri().query().map(|v| {
        url::form_urlencoded::parse(v.as_bytes()).into_owned().collect()
    }).unwrap_or_else(HashMap::new);

    println!("\n\n----------");
    println!("Request: {} from: {:#?}", ctx.req.uri(), ctx.req.headers().get("user-agent"));

    //Make sure a session param was given
    let guid;
    match params.get("id") {
        Some(resource) => {
            println!("  Id: {}\n", resource);
            let parts = resource.replace("acct:", "");
            guid = parts.split("@").next().unwrap().to_string();
        }
        None => {
            println!("Invalid resource.\n");
            return hyper::Response::builder()
                .status(StatusCode::from_u16(400).unwrap())
                .body(format!("No resource given.").into())
                .unwrap();
        }
    }
    let podcast_guid = guid.clone();

    let mut paging = false;
    match params.get("page") {
        Some(page) => {
            println!("  Got a page value: {}\n", page);
            if page == "true" {
                paging = true;
            }
        }
        None => {
            println!("  Non-paged request.");
        }
    }

    //Lookup API of podcast
    let podcast_data: PIEpisodes;
    let api_response = api_get_episodes(API_KEY, API_SECRET, &podcast_guid).await;
    match api_response {
        Ok(response_body) => {
            //eprintln!("{:#?}", response_body);
            match serde_json::from_str(response_body.as_str()) {
                Ok(data) => {
                    podcast_data = data;
                }
                Err(e) => {
                    println!("Response prep error: [{:#?}].\n", e);
                    return hyper::Response::builder()
                        .status(StatusCode::from_u16(501).unwrap())
                        .body(format!("Response prep error.").into())
                        .unwrap();
                }
            }
        }
        Err(e) => {
            println!("Response prep error: [{:#?}].\n", e);
            return hyper::Response::builder()
                .status(StatusCode::from_u16(501).unwrap())
                .body(format!("Response prep error.").into())
                .unwrap();
        }
    }

    //If no page=true was given, just give the outbox configuration
    let outbox_json;
    if !paging {
        let outbox_data = OutboxConfig {
            context: "https://www.w3.org/ns/activitystreams".to_string(),
            id: format!("https://ap.podcastindex.org/outbox?id={}", podcast_guid).to_string(),
            r#type: "OrderedCollection".to_string(),
            totalItems: podcast_data.count,
            first: format!("https://ap.podcastindex.org/outbox?id={}&page=true", podcast_guid).to_string(),
            last: format!("https://ap.podcastindex.org/outbox?id={}&page=true&min_id=0", podcast_guid).to_string(),
        };

        match serde_json::to_string_pretty(&outbox_data) {
            Ok(json_result) => {
                outbox_json = json_result;
            }
            Err(e) => {
                println!("Response prep error: [{:#?}].\n", e);
                return hyper::Response::builder()
                    .status(StatusCode::from_u16(500).unwrap())
                    .body(format!("Response prep error.").into())
                    .unwrap();
            }
        }

        //Otherwise give back a listing of episodes
    } else {
        let mut ordered_items = Vec::new();
        for episode in podcast_data.items {
            ordered_items.push(Item {
                id: format!(
                    "https://ap.podcastindex.org/episodes?id={}&statusid={}&resource=activity",
                    podcast_guid,
                    episode.guid
                ).to_string(),
                r#type: "Create".to_string(),
                actor: format!("https://ap.podcastindex.org/podcasts?id={}", podcast_guid).to_string(),
                published: iso8601(episode.datePublished),
                directMessage: false,
                to: vec!(
                    "https://www.w3.org/ns/activitystreams#Public".to_string()
                ),
                object: Object {
                    id: format!(
                        "https://ap.podcastindex.org/episodes?id={}&statusid={}&resource=post",
                        podcast_guid,
                        episode.guid
                    ).to_string(),
                    r#type: "Note".to_string(),
                    summary: None,
                    inReplyTo: None,
                    published: iso8601(episode.datePublished),
                    url: format!(
                        "https://ap.podcastindex.org/episodes?id={}&statusid={}&resource=public",
                        podcast_guid,
                        episode.guid
                    ).to_string(),
                    attributedTo: format!("https://ap.podcastindex.org/podcasts?id={}", podcast_guid).to_string(),
                    to: vec!(
                        "https://www.w3.org/ns/activitystreams#Public".to_string()
                    ),
                    cc: None,
                    sensitive: false,
                    conversation: format!(
                        "tag:ap.podcastindex.org,{}:objectId={}:objectType=Conversation",
                        iso8601(episode.datePublished),
                        episode.guid
                    ).to_string(),
                    content: format!(
                        "<p>{:.128}</p><p>{:.128}</p><p>Listen: {}</p>",
                        episode.title,
                        episode.description,
                        episode.enclosureUrl
                    ),
                    attachment: vec!(),
                },
            })
        }
        let outbox_data = OutboxPaged {
            context: "https://www.w3.org/ns/activitystreams".to_string(),
            id: "https://www.w3.org/ns/activitystreams".to_string(),
            r#type: "OrderedCollectionPage".to_string(),
            next: format!("https://ap.podcastindex.org/outbox?id={}&page=true&max_id=999999", podcast_guid).to_string(),
            prev: format!("https://ap.podcastindex.org/outbox?id={}&page=true&min_id=0", podcast_guid).to_string(),
            partOf: format!("https://ap.podcastindex.org/outbox?id={}", podcast_guid).to_string(),
            totalItems: podcast_data.count,
            orderedItems: Some(ordered_items),
        };

        match serde_json::to_string_pretty(&outbox_data) {
            Ok(json_result) => {
                outbox_json = json_result;
            }
            Err(e) => {
                println!("Response prep error: [{:#?}].\n", e);
                return hyper::Response::builder()
                    .status(StatusCode::from_u16(500).unwrap())
                    .body(format!("Response prep error.").into())
                    .unwrap();
            }
        }
    }

    return hyper::Response::builder()
        .status(StatusCode::OK)
        .header("Content-type", "application/activity+json; charset=utf-8")
        .body(format!("{}", outbox_json).into())
        .unwrap();
}

pub async fn inbox(ctx: Context) -> Response {

    //Determine HTTP action
    let http_action = ctx.req.method().to_string();

    //Get query parameters
    let params: HashMap<String, String> = ctx.req.uri().query().map(|v| {
        url::form_urlencoded::parse(v.as_bytes()).into_owned().collect()
    }).unwrap_or_else(HashMap::new);

    println!("\n\n----------");
    println!("Request[{}]: {} from: {:#?}", http_action, ctx.req.uri(), ctx.req.headers().get("user-agent"));
    println!("Context: {:#?}", ctx);

    //Make sure a session param was given
    let guid;
    match params.get("id") {
        Some(resource) => {
            println!("  Id: {}\n", resource);
            let parts = resource.replace("acct:", "");
            guid = parts.split("@").next().unwrap().to_string();
        }
        None => {
            println!("Invalid resource.\n");
            return hyper::Response::builder()
                .status(StatusCode::from_u16(400).unwrap())
                .body(format!("No resource given.").into())
                .unwrap();
        }
    }
    let podcast_guid = guid.clone();


    //Is this a POST?
    if http_action.to_lowercase() == "post" {
        //let following_actor;
        let (parts, body) = ctx.req.into_parts();
        let body_bytes = hyper::body::to_bytes(body).await.unwrap();
        let body = std::str::from_utf8(&body_bytes).unwrap();

        println!("{}", body);

        let inbox_request = serde_json::from_str::<InboxRequest>(body);
        println!("Incoming request: {:#?}", inbox_request);
        match inbox_request {
            Ok(incoming_data) => {
                //TODO: This should all be in separate functions
                //TODO: If this incoming request is a verb other than follow, a different struct should be used
                //TODO: ...for decoding, like a Create struct for the object data
                if incoming_data.r#type.to_lowercase() == "follow" {
                    println!("--Follow request");
                    let client = reqwest::Client::new();
                    let response = client
                        .get(&incoming_data.actor)
                        .header(reqwest::header::USER_AGENT, "Podcast Index AP/v0.1.2a")
                        .header(reqwest::header::ACCEPT, "application/activity+json")
                        .send()
                        .await;
                    match response {
                        Ok(response) => {
                            match response.text().await {
                                Ok(response_text) => {
                                    let actor_data = serde_json::from_str::<Actor>(response_text.as_str()).unwrap();
                                    //println!("{:#?}", actor_data);

                                    //Construct a response
                                    println!("  Building follow accept json.");
                                    let accept_data;
                                    match ap_build_follow_accept(incoming_data, podcast_guid.parse::<u64>().unwrap()) {
                                        Ok(data) => {
                                            accept_data = data;
                                        }
                                        Err(e) => {
                                            return hyper::Response::builder()
                                                .status(StatusCode::from_u16(500).unwrap())
                                                .body(format!("Accept build error.").into())
                                                .unwrap();
                                        }
                                    }
                                    let accept_json;
                                    match serde_json::to_string_pretty(&accept_data) {
                                        Ok(json_result) => {
                                            accept_json = json_result;
                                        }
                                        Err(e) => {
                                            println!("Response prep error: [{:#?}].\n", e);
                                            return hyper::Response::builder()
                                                .status(StatusCode::from_u16(500).unwrap())
                                                .body(format!("Accept encode error.").into())
                                                .unwrap();
                                        }
                                    }

                                    //##: Send the accept request to the follower inbox url
                                    println!("  Send the follow accept request.");
                                    ap_send_follow_accept(
                                        podcast_guid.parse::<u64>().unwrap(),
                                        accept_data,
                                        actor_data.inbox
                                    ).await;

                                }
                                Err(e) => {
                                    println!("Bad actor.\n");
                                    return hyper::Response::builder()
                                        .status(StatusCode::from_u16(400).unwrap())
                                        .body(format!("Bad actor.").into())
                                        .unwrap();
                                }
                            }
                        }
                        Err(e) => {
                            println!("Bad actor.\n");
                            return hyper::Response::builder()
                                .status(StatusCode::from_u16(400).unwrap())
                                .body(format!("Bad actor.").into())
                                .unwrap();
                        }
                    }
                }
            }
            Err(e) => {
                println!("Invalid request.\n");
                return hyper::Response::builder()
                    .status(StatusCode::from_u16(400).unwrap())
                    .body(format!("Invalid request.").into())
                    .unwrap();
            }
        }
    }


    // //Make sure a session param was given
    // let guid;
    // match params.get("id") {
    //     Some(resource) => {
    //         println!("  Id: {}\n", resource);
    //         let parts = resource.replace("acct:", "");
    //         guid = parts.split("@").next().unwrap().to_string();
    //     }
    //     None => {
    //         println!("Invalid resource.\n");
    //         return hyper::Response::builder()
    //             .status(StatusCode::from_u16(400).unwrap())
    //             .body(format!("No resource given.").into())
    //             .unwrap();
    //     }
    // }
    // let podcast_guid = guid.clone();

    //TODO: validate the key signature before accepting request

    return hyper::Response::builder()
        .status(StatusCode::OK)
        .header("Content-type", "application/activity+json; charset=utf-8")
        .body(format!("").into())
        .unwrap();
}

pub async fn featured(ctx: Context) -> Response {

    //Get query parameters
    let params: HashMap<String, String> = ctx.req.uri().query().map(|v| {
        url::form_urlencoded::parse(v.as_bytes()).into_owned().collect()
    }).unwrap_or_else(HashMap::new);

    println!("\n\n----------");
    println!("Request: {} from: {:#?}", ctx.req.uri(), ctx.req.headers().get("user-agent"));

    //Make sure a session param was given
    let guid;
    match params.get("id") {
        Some(resource) => {
            println!("  Id: {}\n", resource);
            let parts = resource.replace("acct:", "");
            guid = parts.split("@").next().unwrap().to_string();
        }
        None => {
            println!("Invalid resource.\n");
            return hyper::Response::builder()
                .status(StatusCode::from_u16(400).unwrap())
                .body(format!("No resource given.").into())
                .unwrap();
        }
    }
    let podcast_guid = guid.clone();

    //If no page=true was given, just give the outbox configuration
    let outbox_json;
    let mut ordered_items = Vec::new();
    ordered_items.push(FeaturedItem {
        at_context: vec!(
            "https://www.w3.org/ns/activitystreams".to_string(),
        ),
        actor: format!("https://ap.podcastindex.org/podcasts?id={}", podcast_guid).to_string(),
        attachment: vec!(),
        attributedTo: format!("https://ap.podcastindex.org/podcasts?id={}", podcast_guid).to_string(),
        cc: Some(vec!(
            format!(
                "https://ap.podcastindex.org/followers?id={}",
                podcast_guid
            ).to_string()
        )),
        content: "This account is a podcast.  Follow to see new episodes.".to_string(),
        context: format!(
            "https://ap.podcastindex.org/contexts?id={}&statusid=0",
            podcast_guid
        ).to_string(),
        conversation: format!(
            "https://ap.podcastindex.org/contexts?id={}&statusid=0",
            podcast_guid
        ).to_string(),
        id: format!(
            "https://ap.podcastindex.org/episodes?id={}&statusid=0",
            podcast_guid
        ).to_string(),
        published: "2023-11-09T15:56:28.495803Z".to_string(),
        repliesCount: 0,
        sensitive: false,
        source: "This account is a podcast.  Follow to see new episodes.".to_string(),
        summary: Some("".to_string()),
        tag: vec!(),
        to: vec!(
            "https://www.w3.org/ns/activitystreams#Public".to_string()
        ),
        r#type: "Note".to_string(),
    });
    let outbox_data = Featured {
        at_context: vec!(
            "https://www.w3.org/ns/activitystreams".to_string(),
        ),
        id: "https://www.w3.org/ns/activitystreams".to_string(),
        r#type: "OrderedCollection".to_string(),
        totalItems: 1,
        orderedItems: Some(ordered_items),
    };

    match serde_json::to_string_pretty(&outbox_data) {
        Ok(json_result) => {
            outbox_json = json_result;
        }
        Err(e) => {
            println!("Response prep error: [{:#?}].\n", e);
            return hyper::Response::builder()
                .status(StatusCode::from_u16(500).unwrap())
                .body(format!("Response prep error.").into())
                .unwrap();
        }
    }


    return hyper::Response::builder()
        .status(StatusCode::OK)
        .header("Content-type", "application/activity+json; charset=utf-8")
        .body(format!("{}", outbox_json).into())
        .unwrap();
}

pub async fn episodes(ctx: Context) -> Response {

    //Get query parameters
    let params: HashMap<String, String> = ctx.req.uri().query().map(|v| {
        url::form_urlencoded::parse(v.as_bytes()).into_owned().collect()
    }).unwrap_or_else(HashMap::new);

    println!("\n\n----------");
    println!("Request: {} from: {:#?}", ctx.req.uri(), ctx.req.headers().get("user-agent"));

    //Make sure a session param was given
    let podcast_guid;
    match params.get("id") {
        Some(resource) => {
            println!("  Id: {}\n", resource);
            let parts = resource.replace("acct:", "");
            podcast_guid = parts.split("@").next().unwrap().to_string();
        }
        None => {
            println!("Invalid resource.\n");
            return hyper::Response::builder()
                .status(StatusCode::from_u16(400).unwrap())
                .body(format!("No resource given.").into())
                .unwrap();
        }
    }

    //Get an episode guid, which will be a status
    let episode_guid;
    match params.get("statusid") {
        Some(resource) => {
            println!("  Status Id: {}\n", resource);
            episode_guid = resource;
        }
        None => {
            println!("Invalid status id.\n");
            return hyper::Response::builder()
                .status(StatusCode::from_u16(400).unwrap())
                .body(format!("No status id given.").into())
                .unwrap();
        }
    }

    //If the status id was zero, then this is the pinned post
    let mut episode_json = "".to_string();
    if episode_guid == "0" {
        let episode_data = Status {
            at_context: vec!(
                "https://www.w3.org/ns/activitystreams".to_string(),
            ),
            id: format!(
                "https://ap.podcastindex.org/episodes?id={}&statusid=0",
                podcast_guid
            ).to_string(),
            r#type: "Note".to_string(),
            summary: None,
            inReplyTo: None,
            published: "2023-11-09T15:56:28.495803Z".to_string(),
            url: None,
            attributedTo: format!("https://ap.podcastindex.org/podcasts?id={}", podcast_guid).to_string(),
            to: vec!(
                "https://www.w3.org/ns/activitystreams#Public".to_string()
            ),
            cc: Some(vec!(
                format!(
                    "https://ap.podcastindex.org/followers?id={}",
                    podcast_guid
                ).to_string()
            )),
            sensitive: false,
            conversation: format!(
                "https://ap.podcastindex.org/contexts?id={}&statusid=0",
                podcast_guid
            ).to_string(),
            content: "This account is a podcast.  Follow to see new episodes.".to_string(),
            attachment: vec!(),
            actor: format!("https://ap.podcastindex.org/podcasts?id={}", podcast_guid).to_string(),
            tag: vec!(),
            replies: None,
        };

        match serde_json::to_string_pretty(&episode_data) {
            Ok(json_result) => {
                episode_json = json_result;
            }
            Err(e) => {
                println!("Response prep error: [{:#?}].\n", e);
                return hyper::Response::builder()
                    .status(StatusCode::from_u16(500).unwrap())
                    .body(format!("Response prep error.").into())
                    .unwrap();
            }
        }
    }

    return hyper::Response::builder()
        .status(StatusCode::OK)
        .header("Content-type", "application/activity+json; charset=utf-8")
        .body(format!("{}", episode_json).into())
        .unwrap();
}

pub async fn contexts(ctx: Context) -> Response {

    //Get query parameters
    // let params: HashMap<String, String> = ctx.req.uri().query().map(|v| {
    //     url::form_urlencoded::parse(v.as_bytes()).into_owned().collect()
    // }).unwrap_or_else(HashMap::new);

    println!("\n\n----------");
    println!("Request: {} from: {:#?}", ctx.req.uri(), ctx.req.headers().get("user-agent"));

    //Make sure a session param was given
    // let guid;
    // match params.get("id") {
    //     Some(resource) => {
    //         println!("  Id: {}\n", resource);
    //         let parts = resource.replace("acct:", "");
    //         guid = parts.split("@").next().unwrap().to_string();
    //     }
    //     None => {
    //         println!("Invalid resource.\n");
    //         return hyper::Response::builder()
    //             .status(StatusCode::from_u16(400).unwrap())
    //             .body(format!("No resource given.").into())
    //             .unwrap();
    //     }
    // }
    // let podcast_guid = guid.clone();

    return hyper::Response::builder()
        .status(StatusCode::OK)
        .header("Content-type", "application/activity+json; charset=utf-8")
        .body(format!("").into())
        .unwrap();
}

pub async fn followers(ctx: Context) -> Response {

    //Get query parameters
    // let params: HashMap<String, String> = ctx.req.uri().query().map(|v| {
    //     url::form_urlencoded::parse(v.as_bytes()).into_owned().collect()
    // }).unwrap_or_else(HashMap::new);

    println!("\n\n----------");
    println!("Request: {} from: {:#?}", ctx.req.uri(), ctx.req.headers().get("user-agent"));

    //Make sure a session param was given
    // let guid;
    // match params.get("id") {
    //     Some(resource) => {
    //         println!("  Id: {}\n", resource);
    //         let parts = resource.replace("acct:", "");
    //         guid = parts.split("@").next().unwrap().to_string();
    //     }
    //     None => {
    //         println!("Invalid resource.\n");
    //         return hyper::Response::builder()
    //             .status(StatusCode::from_u16(400).unwrap())
    //             .body(format!("No resource given.").into())
    //             .unwrap();
    //     }
    // }
    // let podcast_guid = guid.clone();

    return hyper::Response::builder()
        .status(StatusCode::OK)
        .header("Content-type", "application/activity+json; charset=utf-8")
        .body(format!("").into())
        .unwrap();
}



//API calls --------------------------------------------------------------------------------------------------
pub async fn api_get_podcast(key: &'static str, secret: &'static str, query: &str) -> Result<String, Box<dyn Error>> {
    println!("PI API Request: /podcasts/byfeedid");

    let api_key = key;
    let api_secret = secret;

    //##: ======== Required values ========
    //##: WARNING: don't publish these to public repositories or in public places!
    //##: NOTE: values below are sample values, to get your own values go to https://api.podcastindex.org
    let api_time = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time mismatch.").as_secs().to_string();

    //##: Create the authorization token.
    //##: The auth token is built by creating an sha1 hash of the key, secret and current time (as a string)
    //##: concatenated together. The hash is a lowercase string.
    let data4hash: String = format!("{}{}{}", api_key, api_secret, api_time);
    //println!("Data to hash: [{}]", data4hash);
    let mut hasher = Sha1::new();
    hasher.update(data4hash);
    let authorization_token = hasher.finalize();
    let api_hash: String = format!("{:X}", authorization_token).to_lowercase();
    //println!("Hash String: [{}]", api_hash);

    //##: Set up the parameters and the api endpoint url to call and make sure all params are
    //##: url encoded before sending.
    let url: String = format!("https://api.podcastindex.org/api/1.0/podcasts/byfeedid?id={}", urlencoding::encode(query));

    //##: Build the query with the required headers
    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", header::HeaderValue::from_static("Rust-podcastindex-org-example/v1.0"));
    headers.insert("X-Auth-Date", header::HeaderValue::from_str(api_time.as_str()).unwrap());
    headers.insert("X-Auth-Key", header::HeaderValue::from_static(api_key));
    headers.insert("Authorization", header::HeaderValue::from_str(api_hash.as_str()).unwrap());
    let client = reqwest::Client::builder().default_headers(headers).build().unwrap();

    //##: Send the request and display the results or the error
    let res = client.get(url.as_str()).send();
    match res.await {
        Ok(res) => {
            println!("  Response: [{}]", res.status());
            return Ok(res.text().await.unwrap());
        }
        Err(e) => {
            eprintln!("  Error: [{}]", e);
            return Err(Box::new(HydraError(format!("Error running SQL query: [{}]", e).into())));
        }
    }
}

pub async fn api_get_episodes(key: &'static str, secret: &'static str, query: &str) -> Result<String, Box<dyn Error>> {
    println!("  PI API Request: /episodes/byfeedid");

    let api_key = key;
    let api_secret = secret;

    //##: ======== Required values ========
    //##: WARNING: don't publish these to public repositories or in public places!
    //##: NOTE: values below are sample values, to get your own values go to https://api.podcastindex.org
    let api_time = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time mismatch.").as_secs().to_string();

    //##: Create the authorization token.
    //##: The auth token is built by creating an sha1 hash of the key, secret and current time (as a string)
    //##: concatenated together. The hash is a lowercase string.
    let data4hash: String = format!("{}{}{}", api_key, api_secret, api_time);
    //println!("Data to hash: [{}]", data4hash);
    let mut hasher = Sha1::new();
    hasher.update(data4hash);
    let authorization_token = hasher.finalize();
    let api_hash: String = format!("{:X}", authorization_token).to_lowercase();
    //println!("Hash String: [{}]", api_hash);

    //##: Set up the parameters and the api endpoint url to call and make sure all params are
    //##: url encoded before sending.
    let url: String = format!("https://api.podcastindex.org/api/1.0/episodes/byfeedid?id={}", urlencoding::encode(query));

    //##: Build the query with the required headers
    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", header::HeaderValue::from_static("Rust-podcastindex-org-example/v1.0"));
    headers.insert("X-Auth-Date", header::HeaderValue::from_str(api_time.as_str()).unwrap());
    headers.insert("X-Auth-Key", header::HeaderValue::from_static(api_key));
    headers.insert("Authorization", header::HeaderValue::from_str(api_hash.as_str()).unwrap());
    let client = reqwest::Client::builder().default_headers(headers).build().unwrap();

    //##: Send the request and display the results or the error
    let res = client.get(url.as_str()).send();
    match res.await {
        Ok(res) => {
            println!("  Response: [{}]", res.status());
            return Ok(res.text().await.unwrap());
        }
        Err(e) => {
            eprintln!("  Error: [{}]", e);
            return Err(Box::new(HydraError(format!("Error running SQL query: [{}]", e).into())));
        }
    }
}


//ActivityPub helper functions -------------------------------------------------------------------------------
fn ap_build_actor_object(podcast_data: PIPodcast, actor_keys: ActorKeys) -> Result<Actor, Box<dyn Error>> {
    let podcast_guid = podcast_data.feed.id;

    return Ok(Actor {
        at_context: vec!(
            "https://www.w3.org/ns/activitystreams".to_string(),
            "https://w3id.org/security/v1".to_string(),
        ),
        id: format!("https://ap.podcastindex.org/podcasts?id={}", podcast_guid).to_string(),
        r#type: "Person".to_string(),
        discoverable: true,
        preferredUsername: podcast_guid.to_string(),
        name: format!("{:.48}", podcast_data.feed.title).to_string(),
        inbox: format!("https://ap.podcastindex.org/inbox?id={}", podcast_guid).to_string(),
        outbox: format!("https://ap.podcastindex.org/outbox?id={}", podcast_guid).to_string(),
        featured: format!("https://ap.podcastindex.org/featured?id={}", podcast_guid).to_string(),
        followers: format!("https://ap.podcastindex.org/followers?id={}", podcast_guid).to_string(),
        following: format!("https://ap.podcastindex.org/following?id={}", podcast_guid).to_string(),
        icon: Some(Icon {
            r#type: "Image".to_string(),
            mediaType: None,
            url: format!("{}", podcast_data.feed.image).to_string(),
        }),
        summary: format!("{:.96}", podcast_data.feed.description),
        attachment: Some(vec!(
            Attachment {
                name: "Index".to_string(),
                r#type: "PropertyValue".to_string(),
                value: format!(
                    "<a href='https://podcastindex.org/podcast/{}' rel='ugc'>https://podcastindex.org/podcast/{}</a>",
                    podcast_guid,
                    podcast_guid,
                ).to_string(),
            },
            Attachment {
                name: "Website".to_string(),
                r#type: "PropertyValue".to_string(),
                value: format!(
                    "<a href='{}' rel='ugc'>{}</a>",
                    podcast_data.feed.link,
                    podcast_data.feed.link,
                ).to_string(),
            },
            Attachment {
                name: "Podcast Guid".to_string(),
                r#type: "PropertyValue".to_string(),
                value: format!(
                    "{}",
                    podcast_data.feed.podcastGuid,
                ).to_string(),
            },
        )),
        publicKey: PublicKey {
            id: format!("https://ap.podcastindex.org/podcasts?id={}#main-key", podcast_guid).to_string(),
            owner: format!("https://ap.podcastindex.org/podcasts?id={}", podcast_guid).to_string(),
            publicKeyPem: actor_keys.pem_public_key,
        },
        endpoints: Endpoints {
            sharedInbox: "https://ap.podcastindex.org/inbox?id=0".to_string(),
        },
        url: format!("https://podcastindex.org/podcast/{}", podcast_guid).to_string(),
        manuallyApprovesFollowers: false,
        indexable: Some(true),
        memorial: Some(false),
        published: "2023-11-09T15:56:28.495803Z".to_string(),
        devices: None,
        tag: vec!(),
    });
}

fn ap_build_follow_accept(follow_request: InboxRequest, podcast_guid: u64) -> Result<InboxRequestAccept, Box<dyn Error>> {

    return Ok(
        InboxRequestAccept {
            at_context: "https://www.w3.org/ns/activitystreams".to_string(),
            id: format!("https://ap.podcastindex.org/podcasts?id={}&context=accept", podcast_guid).to_string(),
            r#type: "Accept".to_string(),
            actor: follow_request.object.clone(),
            object: follow_request,
        }
    );
}

fn ap_get_actor_keys(podcast_guid: u64) -> Result<ActorKeys, Box<dyn Error>> {

    println!("  Getting actor keys for: [{}]", podcast_guid);

    let actor_keys;
    let pem_pub_key;
    let pem_priv_key;
    match dbif::get_actor_from_db(&"ap.db".to_string(), podcast_guid) {
        Ok(actor_record) => {
            pem_pub_key = actor_record.pem_public_key;
            pem_priv_key = actor_record.pem_private_key;

            actor_keys = ActorKeys {
                pem_private_key: pem_priv_key.clone(),
                pem_public_key: pem_pub_key.clone(),
            }
        }
        Err(e) => {
            eprintln!("get_actor_from_db error: [{:#?}]", e);

            //TODO: wip
            let priv_key;
            let pub_key;
            {
                let mut rng = rand::thread_rng();
                let bits = 2048;
                priv_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate key");
                pub_key = RsaPublicKey::from(&priv_key);
            }
            match pkcs1::EncodeRsaPrivateKey::to_pkcs1_pem(&priv_key, pkcs1::LineEnding::LF) {
                Ok(pem_encoded_privkey) => {
                    pem_priv_key = pem_encoded_privkey.to_string();
                }
                Err(e) => {
                    return Err(Box::new(HydraError(format!("Error encoding private key: [{}]", e).into())));
                }
            }
            println!("Private key: {:.40}", pem_priv_key);
            match pkcs1::EncodeRsaPublicKey::to_pkcs1_pem(&pub_key, pkcs1::LineEnding::LF) {
                Ok(pem_encoded_pubkey) => {
                    pem_pub_key = pem_encoded_pubkey.to_string();
                }
                Err(e) => {
                    return Err(Box::new(HydraError(format!("Error encoding public key: [{}]", e).into())));
                }
            }
            println!("Public key: {:.40}", pem_pub_key);

            let _ = dbif::add_actor_to_db(&"ap.db".to_string(), ActorRecord {
                pcid: podcast_guid,
                guid: "".to_string(),
                pem_private_key: pem_priv_key.clone(),
                pem_public_key: pem_pub_key.clone(),
            });
            println!("Saved actor to DB");

            actor_keys = ActorKeys {
                pem_private_key: pem_priv_key.clone(),
                pem_public_key: pem_pub_key.clone(),
            }
        }
    }

    return Ok(actor_keys);
}

pub async fn ap_send_follow_accept(podcast_guid: u64, inbox_accept: InboxRequestAccept, inbox_url: String) -> Result<String, Box<dyn Error>> {
    println!("  AP Accepting Follow request from: {}", inbox_accept.object.actor);

    //##: Get actor keys for guid
    let actor_keys = ap_get_actor_keys(podcast_guid).unwrap();

    //##: Decode the private key
    let private_key = sigh::PrivateKey::from_pem(actor_keys.pem_private_key.as_bytes()).unwrap();

    //Construct the POST body
    let post_body;
    match serde_json::to_string_pretty(&inbox_accept) {
        Ok(json_result) => {
            post_body = json_result;
        }
        Err(e) => {
            return Err(Box::new(HydraError(format!("Error building post body: [{}]", e).into())));
        }
    }
    println!("  SIG - POST BODY: {}", post_body);
    
    //##: ======== Required values ========
    //##: WARNING: don't publish these to public repositories or in public places!
    //##: NOTE: values below are sample values, to get your own values go to https://api.podcastindex.org
    let headers_to_hash = "(request-target) host date digest";
    let hash_algorithm = "rsa-sha256";
    let header_date = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time mismatch.").as_secs().to_string();
    let url_parts = url::Url::parse(inbox_url.as_str());
    match url_parts {
        Ok(_) => {}
        Err(e) => {
            return Err(Box::new(HydraError(format!("Invalid inbox url: [{}]", e).into())));
        }
    }
    let parts = url_parts.unwrap();
    let header_host = parts.host_str().unwrap();
    let header_path = parts.path();
    let mut hasher = Sha256::new();
    hasher.update(post_body);
    let digest_hash = hasher.finalize();
    let digest_string = general_purpose::STANDARD.encode(digest_hash);
    println!("{}", digest_string);

    //##: Create the authorization token.
    //##: The auth token is built by creating an sha1 hash of the key, secret and current time (as a string)
    //##: concatenated together. The hash is a lowercase string.
    let headers_for_hashing: String = format!(
        "(request-target): post {}\nhost: {}\ndate: {}\ndigest: sha-256={}",
        header_path,
        header_host,
        header_date,
        digest_string
    );
    //println!("Data to hash: [{}]", data4hash);
    let mut hasher = Sha256::new();
    hasher.update(headers_for_hashing);
    let signature_string = hasher.finalize();
    println!("Signature string: [{:x}]", signature_string);

    //##: The url to send to must be the follower actors inbox url
    let url = format!("{}", urlencoding::encode(&inbox_url));

    //##: Calculate the signature
    let request_signature = format!(
        "keyId=\"https://ap.podcastindex.org/podcasts?id={}#main-key\",algorithm=\"{}\",headers=\"{}\",signature=\"{:x}\"",
        podcast_guid,
        hash_algorithm,
        headers_to_hash,
        signature_string
    );
    let signature_header = request_signature.clone();
    let signature_header_string = signature_header.as_str();

    //##: Build the query with the required headers
    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", header::HeaderValue::from_static("Podcast Index AP/v0.1.2a"));
    headers.insert("Accept", header::HeaderValue::from_static("application/activity+json"));
    headers.insert("Content-type", header::HeaderValue::from_static("application/json"));
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    //##: Send the request and display the results or the error
    //let mut request = client.post(url.as_str()).body(post_body).build().unwrap();
    // let mut request = http::Request::builder()
    //     .method("POST")
    //     .uri(url.as_str())
    //     .header("User-Agent", "Podcast Index AP/v0.1.2a")
    //     .header("Accept", "application/activity+json")
    //     .header("Content-type", "application/json")
    //     .body(post_body)
    //     .unwrap();
    // //Sign the request
    // // SigningConfig::new(
    // //     RsaSha256,
    // //     &private_key,
    // //     format!("https://ap.podcastindex.org/podcasts?id={}#main-key", podcast_guid).to_string()
    // // ).sign(&mut request);
    // sign_request(&mut request, &actor_keys.pem_private_key.as_bytes());

    //println!("{:#?}", request.into_parts());

    //Send the request
    println!("  URL: [{}]", inbox_url.as_str());
    let res = client.post(inbox_url.as_str()).send();
    match res.await {
        Ok(res) => {
            println!("  Response: [{}]", res.status());
            return Ok(res.text().await.unwrap());
        }
        Err(e) => {
            eprintln!("  Error: [{}]", e);
            return Err(Box::new(HydraError(format!("Error sending follow accept request: [{}]", e).into())));
        }
    }

    // return Ok("".to_string());

}



//Utilities --------------------------------------------------------------------------------------------------
fn iso8601(utime: u64) -> String {

    // Create DateTime from SystemTime
    let datetime = Utc.timestamp_opt(utime as i64, 0).unwrap();

    // Formats the combined date and time with the specified format string.
    datetime.format("%+").to_string()
}

// fn sign_request<B>(request: &mut http::Request<B>, private_key_pem: &[u8]) -> Result<(), sigh::Error> {
//     let private_key = PrivateKey::from_pem(private_key_pem)?;
//     SigningConfig::new(RsaSha256, &private_key, "my-key-id")
//         .sign(request)
// }

// fn get_sys_time_in_secs() -> u64 {
//     match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
//         Ok(n) => n.as_secs(),
//         Err(_) => panic!("SystemTime before UNIX EPOCH!"),
//     }
// }