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
use urlencoding;
use reqwest::header;
use chrono::{TimeZone, Utc};


//Globals ----------------------------------------------------------------------------------------------------
//TODO: These secrets need to be moved into the environment
const API_KEY: &str = "B899NK69ERMRE2M6HD3B";
const API_SECRET: &str = "J3v9m$4b6NCD9ENV4QEKYb^DnWdcGR$^Gq7#5uwS";
const AP_PUBKEY: &str = "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEArYGPVCRtdZXQLAYANU6R\nhH5e5bPQ8ImW7AxOkFRIoAhK0+zJOHsn6UIrpdXK7JcIdkR3pPEG620BVHUkZOVC\nYUsnW7gNWAPyeXMVUPO0h2okCyUIeOSoRuIto8AfsfaQOeLeCIt0bqHymX4FueRi\n6y3fpUlGkNMLJ6T1tfLXElwcNxNnzEV4dvCpwHh9lZwQKersbKFgpVFl5VO9+ZG+\nhO2ym6KMGeD09oPK7lvvhjItfEkqmzOVCkH4PRXaHwcst9lBNwSNKeUkNWfIg8Bd\nqFFCZMx6+VmwBeaFIa8ia9jMT2ofTZ56Whlx7Jo9j7wtTGNeo/HC0v4Uvkbw+o+v\nfQIDAQAB\n-----END PUBLIC KEY-----";

//Structs ----------------------------------------------------------------------------------------------------
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct Link {
    rel: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    href: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    template: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Webfinger {
    subject: String,
    aliases: Vec<String>,
    links: Vec<Link>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct PublicKey {
    id: String,
    owner: String,
    publicKeyPem: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct Icon {
    r#type: String,
    url: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct Attachment {
    name: String,
    r#type: String,
    value: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct Actor {
    #[serde(rename = "@context")]
    context: Vec<String>,
    id: String,
    r#type: String,
    discoverable: bool,
    preferredUsername: String,
    name: String,
    inbox: String,
    outbox: String,
    featured: String,
    followers: String,
    following: String,
    icon: Icon,
    summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    attachment: Option<Vec<Attachment>>,
    publicKey: PublicKey,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct OutboxConfig {
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
struct Object {
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
struct Item {
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
struct OutboxPaged {
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
struct Featured {
    #[serde(rename = "@context")]
    context: String,
    id: String,
    r#type: String,
    totalItems: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    orderedItems: Option<Vec<FeaturedItem>>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct FeaturedItem {
    #[serde(rename = "@context")]
    at_context: String,
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
struct PIFeed {
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
struct PIPodcast {
    status: String,
    feed: PIFeed,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct PIItem {
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
struct PIEpisodes {
    status: String,
    items: Vec<PIItem>,
    count: u64,
}

#[derive(Debug)]
struct HydraError(String);

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

    println!("----------");
    println!("Request: {} from: {:#?}", ctx.req.uri(), ctx.req.headers().get("user-agent"));

    //Make sure a session param was given
    let guid;
    match params.get("resource") {
        Some(resource) => {
            println!("  Resource: {}\n", resource);
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

    println!("----------");
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

    //Construct a response
    let actor_data = Actor {
        context: vec!(
            "https://www.w3.org/ns/activitystreams".to_string(),
            "https://w3id.org/security/v1".to_string(),
        ),
        id: format!("https://ap.podcastindex.org/podcasts?id={}", podcast_guid).to_string(),
        r#type: "Person".to_string(),
        discoverable: true,
        preferredUsername: podcast_guid.clone(),
        name: format!("{:.48}", podcast_data.feed.title).to_string(),
        inbox: format!("https://ap.podcastindex.org/inbox?id={}", podcast_guid).to_string(),
        outbox: format!("https://ap.podcastindex.org/outbox?id={}", podcast_guid).to_string(),
        featured: format!("https://ap.podcastindex.org/featured?id={}", podcast_guid).to_string(),
        followers: format!("https://ap.podcastindex.org/followers?id={}", podcast_guid).to_string(),
        following: format!("https://ap.podcastindex.org/following?id={}", podcast_guid).to_string(),
        icon: Icon {
            r#type: "Image".to_string(),
            url: format!("{}", podcast_data.feed.image).to_string(),
        },
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
            publicKeyPem: AP_PUBKEY.to_string(),
        },
    };

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

    println!("----------");
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

    println!("----------");
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
            println!("Got a page value: {}\n", page);
            if page == "true" {
                paging = true;
            }
        }
        None => {
            println!("No page param present.");
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
                    "https://ap.podcastindex.org/statuses?id={}&statusid={}&resource=activity",
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
                        "https://ap.podcastindex.org/statuses?id={}&statusid={}&resource=post",
                        podcast_guid,
                        episode.guid
                    ).to_string(),
                    r#type: "Note".to_string(),
                    summary: None,
                    inReplyTo: None,
                    published: iso8601(episode.datePublished),
                    url: format!(
                        "https://ap.podcastindex.org/statuses?id={}&statusid={}&resource=public",
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

    //Get query parameters
    let params: HashMap<String, String> = ctx.req.uri().query().map(|v| {
        url::form_urlencoded::parse(v.as_bytes()).into_owned().collect()
    }).unwrap_or_else(HashMap::new);

    println!("----------");
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

    println!("----------");
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
        at_context: "https://www.w3.org/ns/activitystreams".to_string(),
        actor: format!("https://ap.podcastindex.org/podcasts?id={}", podcast_guid).to_string(),
        attachment: vec!(),
        attributedTo: format!("https://ap.podcastindex.org/podcasts?id={}", podcast_guid).to_string(),
        cc: None,
        content: "This mastodon account is a podcast.  Follow to see new episodes.".to_string(),
        context: format!(
            "https://ap.podcastindex.org/statuses?id={}&statusid=0",
            podcast_guid
        ).to_string(),
        conversation: format!(
            "https://ap.podcastindex.org/statuses?id={}&statusid=0",
            podcast_guid
        ).to_string(),
        id: format!(
            "https://ap.podcastindex.org/statuses?id={}&statusid=0",
            podcast_guid
        ).to_string(),
        published: iso8601(get_sys_time_in_secs()),
        repliesCount: 0,
        sensitive: false,
        source: "This mastodon account is a podcast.  Follow to see new episodes.".to_string(),
        summary: Some("".to_string()),
        tag: vec!(),
        to: vec!(
            "https://www.w3.org/ns/activitystreams#Public".to_string()
        ),
        r#type: "Note".to_string(),
    });
    let outbox_data = Featured {
        context: "https://www.w3.org/ns/activitystreams".to_string(),
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
    println!("Data to hash: [{}]", data4hash);
    let mut hasher = Sha1::new();
    hasher.update(data4hash);
    let authorization_token = hasher.finalize();
    let api_hash: String = format!("{:X}", authorization_token).to_lowercase();
    println!("Hash String: [{}]", api_hash);

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
            println!("Response Status: [{}]", res.status());
            return Ok(res.text().await.unwrap());
        }
        Err(e) => {
            eprintln!("API response error: [{}]", e);
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
    println!("Data to hash: [{}]", data4hash);
    let mut hasher = Sha1::new();
    hasher.update(data4hash);
    let authorization_token = hasher.finalize();
    let api_hash: String = format!("{:X}", authorization_token).to_lowercase();
    println!("Hash String: [{}]", api_hash);

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
            println!("Response Status: [{}]", res.status());
            return Ok(res.text().await.unwrap());
        }
        Err(e) => {
            eprintln!("API response error: [{}]", e);
            return Err(Box::new(HydraError(format!("Error running SQL query: [{}]", e).into())));
        }
    }
}


//Utilities --------------------------------------------------------------------------------------------------
fn iso8601(utime: u64) -> String {

    // Create DateTime from SystemTime
    let datetime = Utc.timestamp_opt(utime as i64, 0).unwrap();

    // Formats the combined date and time with the specified format string.
    datetime.format("%+").to_string()
}

fn get_sys_time_in_secs() -> u64 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}