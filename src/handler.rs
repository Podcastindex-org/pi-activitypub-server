use crate::{Context, crypto_rsa, http_signature, Response};
use hyper::StatusCode;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use sha1::{Sha1};
use sha2::{Sha256, Digest};
use urlencoding;
use reqwest::header;
use chrono::{TimeZone, Utc};
use rsa::{RsaPrivateKey, RsaPublicKey};
use rsa::pkcs1v15::{SigningKey, VerifyingKey};
use rsa::signature::{Keypair, RandomizedSigner, SignatureEncoding, Verifier, Signer};
use dbif::{ActorRecord};
use base64::{Engine as _, engine::{general_purpose}};
use rand::rngs::ThreadRng;
use sha256::digest;


//Globals ----------------------------------------------------------------------------------------------------
//TODO: These secrets need to be moved into the environment
pub const API_KEY: &str = "B899NK69ERMRE2M6HD3B";
pub const API_SECRET: &str = "J3v9m$4b6NCD9ENV4QEKYb^DnWdcGR$^Gq7#5uwS";


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
pub struct Create {
    #[serde(rename = "@context", skip_deserializing)]
    at_context: String,
    id: String,
    r#type: String,
    actor: String,
    published: String,
    to: Vec<String>,
    cc: Option<Vec<String>>,
    object: Object,
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
    pub id: u64,
    pub podcastGuid: String,
    pub medium: String,
    pub title: String,
    pub url: String,
    pub originalUrl: String,
    pub link: String,
    pub description: String,
    pub author: String,
    pub ownerName: String,
    pub image: String,
    pub artwork: String,
    pub episodeCount: u64,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct PIPodcast {
    pub status: String,
    pub feed: PIFeed,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct PIItem {
    pub id: u64,
    pub title: String,
    pub link: String,
    pub description: String,
    pub guid: String,
    pub datePublished: u64,
    pub datePublishedPretty: String,
    pub enclosureUrl: String,
    pub enclosureType: String,
    pub duration: u64,
    pub image: String,
    pub feedImage: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct PIEpisodes {
    pub status: String,
    pub items: Vec<PIItem>,
    pub count: u64,
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
            println!("Actor keys retreival error: [{:#?}].\n", e);
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
            println!("Actor object build error: [{:#?}].\n", e);
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
        let (_parts, body) = ctx.req.into_parts();
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
                                            println!("Build follow accept error: [{:#?}].\n", e);
                                            return hyper::Response::builder()
                                                .status(StatusCode::from_u16(500).unwrap())
                                                .body(format!("Accept build error.").into())
                                                .unwrap();
                                        }
                                    }
                                    let _accept_json;
                                    match serde_json::to_string_pretty(&accept_data) {
                                        Ok(json_result) => {
                                            _accept_json = json_result;
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
                                    println!("Bad actor: [{}].\n", e);
                                    return hyper::Response::builder()
                                        .status(StatusCode::from_u16(400).unwrap())
                                        .body(format!("Bad actor.").into())
                                        .unwrap();
                                }
                            }
                        }
                        Err(e) => {
                            println!("Bad actor: [{}].\n", e);
                            return hyper::Response::builder()
                                .status(StatusCode::from_u16(400).unwrap())
                                .body(format!("Bad actor.").into())
                                .unwrap();
                        }
                    }
                }
            }
            Err(e) => {
                println!("Invalid request: [{}].\n", e);
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


pub fn api_block_get_episodes(key: &'static str, secret: &'static str, query: &str) -> Result<String, Box<dyn Error>> {
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
    let client = reqwest::blocking::Client::builder().default_headers(headers).build().unwrap();

    //##: Send the request and display the results or the error
    let res = client.get(url.as_str()).send();
    match res {
        Ok(res) => {
            println!("  Response: [{}]", res.status());
            return Ok(res.text().unwrap());
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

    //##: Decode the private key for the podcast actor
    let private_key;
    match crypto_rsa::rsa_private_key_from_pkcs1_pem(&actor_keys.pem_private_key) {
        Ok(pem_decoded_privkey) => {
            private_key = pem_decoded_privkey;
        }
        Err(e) => {
            return Err(Box::new(HydraError(format!("Error decoding private key: [{}]", e).into())));
        }
    }

    //##: Construct the follow "accept" POST body to send
    let post_body;
    match serde_json::to_string_pretty(&inbox_accept) {
        Ok(json_result) => {
            post_body = json_result;
        }
        Err(e) => {
            return Err(Box::new(HydraError(format!("Error building post body: [{}]", e).into())));
        }
    }
    println!("  POST BODY: {}", post_body);

    let key_id = format!("https://ap.podcastindex.org/podcasts?id={}#main-key", podcast_guid);
    let http_signature_headers ;
    match http_signature::create_http_signature(
        http::Method::POST,
        &inbox_url,
        &post_body.clone(),
        &private_key,
        &key_id
    ) {
        Ok(sig_headers) => {
            http_signature_headers = sig_headers;
        }
        Err(e) => {
            return Err(Box::new(HydraError(format!("Could not build http signature headers: [{}]", e).into())));
        }
    }

    //##: Build the query with the required headers
    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", header::HeaderValue::from_static("Podcast Index AP/v0.1.2a"));
    headers.insert("Accept", header::HeaderValue::from_static("application/activity+json"));
    headers.insert("Content-type", header::HeaderValue::from_static("application/activity+json"));
    headers.insert("date", header::HeaderValue::from_str(&http_signature_headers.date).unwrap());
    headers.insert("host", header::HeaderValue::from_str(&http_signature_headers.host).unwrap());
    headers.insert("digest", header::HeaderValue::from_str(&http_signature_headers.digest.unwrap()).unwrap());
    headers.insert("signature", header::HeaderValue::from_str(&http_signature_headers.signature).unwrap());
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    //##: Send the Accept request
    println!("  ACCEPT SENT: [{}]", inbox_url.as_str());
    let res = client
        .post(inbox_url.as_str())
        .body(post_body)
        .send();
    match res.await {
        Ok(res) => {
            println!("  Response: [{:#?}]", res);
            let res_body = res.text().await?;
            println!("  Body: [{:#?}]", res_body);
            return Ok(res_body);
        }
        Err(e) => {
            eprintln!("  Error: [{}]", e);
            return Err(Box::new(HydraError(format!("Error sending follow accept request: [{}]", e).into())));
        }
    }
}

pub fn ap_block_send_note(podcast_guid: u64, episode: &PIItem, inbox_url: String) -> Result<String, Box<dyn Error>> {

    println!("  AP Sending create episode note from actor: {}", podcast_guid);

    //##: Get actor keys for guid
    let actor_keys = ap_get_actor_keys(podcast_guid).unwrap();

    //##: Decode the private key for the podcast actor
    let private_key;
    match crypto_rsa::rsa_private_key_from_pkcs1_pem(&actor_keys.pem_private_key) {
        Ok(pem_decoded_privkey) => {
            private_key = pem_decoded_privkey;
        }
        Err(e) => {
            return Err(Box::new(HydraError(format!("Error decoding private key: [{}]", e).into())));
        }
    }

    //##: Construct the episode note object to send
    let create_action_object = Create {
        at_context: "https://www.w3.org/ns/activitystreams".to_string(),
        id: format!(
            "https://ap.podcastindex.org/episodes?id={}&statusid={}&resource=activity",
            podcast_guid,
            episode.guid
        ).to_string(),
        r#type: "Create".to_string(),
        actor: format!("https://ap.podcastindex.org/podcasts?id={}", podcast_guid).to_string(),
        published: iso8601(episode.datePublished),
        to: vec!(
            "https://www.w3.org/ns/activitystreams#Public".to_string()
        ),
        cc: None,
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
                "<p>{:.128}</p><p>{:.128}</p><p>Listen: <a href=\"{}\">{}</a></p>",
                episode.title,
                episode.description,
                episode.enclosureUrl,
                episode.enclosureUrl,
            ),
            attachment: vec!(),
        },
    };
    //##: Convert the note create action to JSON and send
    let create_json;
    match serde_json::to_string_pretty(&create_action_object) {
        Ok(json_result) => {
            create_json = json_result;
        }
        Err(e) => {
            eprintln!("Response prep error: [{:#?}].\n", e);
            return Err(Box::new(HydraError(format!("Error building create note request json: [{}]", e).into())));
        }
    }

    //##: Build the http signing headers
    let key_id = format!("https://ap.podcastindex.org/podcasts?id={}#main-key", podcast_guid);
    let http_signature_headers ;
    match http_signature::create_http_signature(
        http::Method::POST,
        &inbox_url,
        &create_json.clone(),
        &private_key,
        &key_id
    ) {
        Ok(sig_headers) => {
            http_signature_headers = sig_headers;
        }
        Err(e) => {
            return Err(Box::new(HydraError(format!("Could not build http signature headers: [{}]", e).into())));
        }
    }

    //##: Build the query with the required headers
    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", header::HeaderValue::from_static("Podcast Index AP/v0.1.2a"));
    headers.insert("Accept", header::HeaderValue::from_static("application/activity+json"));
    headers.insert("Content-type", header::HeaderValue::from_static("application/activity+json"));
    headers.insert("date", header::HeaderValue::from_str(&http_signature_headers.date).unwrap());
    headers.insert("host", header::HeaderValue::from_str(&http_signature_headers.host).unwrap());
    headers.insert("digest", header::HeaderValue::from_str(&http_signature_headers.digest.unwrap()).unwrap());
    headers.insert("signature", header::HeaderValue::from_str(&http_signature_headers.signature).unwrap());
    let client = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    //##: Send the Accept request
    println!("  EPISODE NOTE SENT: [{}|{}|{}]", podcast_guid, episode.guid, inbox_url.as_str());
    let res = client
        .post(inbox_url.as_str())
        .body(create_json)
        .send();
    match res {
        Ok(res) => {
            println!("  Response: [{:#?}]", res);
            let res_body = res.text()?;
            println!("  Body: [{:#?}]", res_body);
            return Ok(res_body);
        }
        Err(e) => {
            eprintln!("  Error: [{}]", e);
            return Err(Box::new(HydraError(format!("Error sending episode create note request: [{}]", e).into())));
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