use crate::{Context, crypto_rsa, http_signature, Response};
use hyper::StatusCode;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::marker::PhantomData;
use serde::{Deserialize, Serialize, Deserializer};
use serde::de::{self, MapAccess, Visitor};
use std::time::{SystemTime, UNIX_EPOCH};
use sha1::{Sha1};
use sha2::{Digest};
use urlencoding;
use reqwest::header;
use chrono::{TimeZone, Utc};
use rsa::{RsaPrivateKey, RsaPublicKey};
//use rsa::pkcs1v15::{SigningKey, VerifyingKey};
//use rsa::signature::{Keypair, RandomizedSigner, SignatureEncoding, Verifier, Signer};
use dbif::{ActorRecord, FollowerRecord, ReplyRecord};
use base64::{Engine as _, engine::{general_purpose}};
//use rand::rngs::ThreadRng;
//use sha256::digest;
use core::str::FromStr;
use void::Void;


//Globals ----------------------------------------------------------------------------------------------------
const AP_DATABASE_FILE: &str = "database.db";

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
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Icon {
    r#type: String,
    mediaType: Option<String>,
    url: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TagObject {
    id: Option<String>,
    r#type: String,
    name: Option<String>,
    href: Option<String>,
    updated: Option<String>,
    icon: Option<Icon>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Attachment {
    name: Option<String>,
    r#type: String,
    value: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct NoteAttachment {
    name: Option<String>,
    r#type: Option<String>,
    value: Option<String>,
    mediaType: Option<String>,
    url: Option<String>,
    blurhash: Option<String>,
    width: Option<u64>,
    height: Option<u64>,
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
    discoverable: Option<bool>,
    indexable: Option<bool>,
    preferredUsername: Option<String>,
    published: Option<String>,
    memorial: Option<bool>,
    devices: Option<String>,
    //tag: Vec<String>,
    name: Option<String>,
    inbox: String,
    outbox: String,
    featured: Option<String>,
    followers: Option<String>,
    following: Option<String>,
    icon: Option<Icon>,
    summary: Option<String>,
    //alsoKnownAs: Option<String>,
    url: Option<String>,
    manuallyApprovesFollowers: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    attachment: Option<Vec<Attachment>>,
    publicKey: PublicKey,
    endpoints: Option<Endpoints>,
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
pub struct InboxRequestWithObject {
    id: String,
    r#type: String,
    actor: Option<String>,
    #[serde(default = "d_blank_inboxrequest", deserialize_with = "de_optional_string_or_struct")]
    object: InboxRequestObject,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InboxRequestObject {
    id: String,
    //"https://podcastindex.social/users/dave/statuses/111689975521776545"
    r#type: Option<String>,
    //"Note"
    actor: Option<String>,
    summary: Option<String>,
    //null
    inReplyTo: Option<String>,
    //"https://podcastindex.social/users/dave/statuses/111683789181428177"
    published: Option<String>,
    //"2024-01-03T03:49:36Z"
    url: Option<String>,
    //"https://podcastindex.social/@dave/111689975521776545"
    attributedTo: Option<String>,
    //"https://podcastindex.social/users/dave"
    to: Option<Vec<String>>,
    //["https://www.w3.org/ns/activitystreams#Public" ]
    cc: Option<Vec<String>>,
    //["https://podcastindex.social/users/dave/followers", "https://ap.podc...]"
    sensitive: Option<bool>,
    //false
    atomUri: Option<String>,
    //"https://podcastindex.social/users/dave/statuses/111689975521776545"
    inReplyToAtomUri: Option<String>,
    //"https://podcastindex.social/users/dave/statuses/111683789181428177"
    conversation: Option<String>,
    //"tag:ap.podcastindex.org,2023-12-30T05:02:50+00:00:objectId=podserve:f69b5c6c-6c16-43e5-a9d6-3f93a2756e48:objectType=Conversation",
    content: Option<String>,
    //"\u003cp\u003e\u003cspan class=\"h-card\" translate=\"no\"\u003e\u003ca href=\"https://ap.podcastindex.org/podcasts?id=6594066\" class=\"u-url mention\"\u003e@\u003cspan\u003e6594066\u003c/span\u003e\u003c/a\u003e\u003c/span\u003e Another test. \u003ca href=\"https://podcastindex.social/tags/ignore\" class=\"mention hashtag\" rel=\"tag\"\u003e#\u003cspan\u003eignore\u003c/span\u003e\u003c/a\u003e\u003c/p\u003e",
    attachment: Option<Vec<Attachment>>,
    tag: Option<Vec<TagObject>>,
    replies: Option<ReplyCollection>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReplyCollection {
    id: Option<String>,
    r#type: Option<String>,
    first: Option<ReplyCollectionPage>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReplyCollectionPage {
    id: Option<String>,
    r#type: Option<String>,
    next: Option<String>,
    partOf: Option<String>,
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
    attachment: Vec<NoteAttachment>,
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
    attachment: Option<Vec<NoteAttachment>>,
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
    pub funding: Option<PIFunding>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct PIFunding {
    url: Option<String>,
    message: Option<String>,
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
    pub feedUrl: String,
    pub podcastGuid: String,
    pub feedId: u64,
    pub feedItunesId: Option<u64>,
    pub socialInteract: Option<Vec<PISocialInteract>>,
    pub transcripts: Option<Vec<PITranscript>>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct PITranscript {
    pub url: Option<String>,
    pub r#type: Option<String>,
}

#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize)]
pub enum PISocialInteractProtocol {
    activitypub,
    xmpp,
    bluesky,
    nostr,
    twitter,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct PISocialInteract {
    pub uri: Option<String>,
    pub protocol: Option<PISocialInteractProtocol>,
    pub accountId: Option<String>,
    pub accountUrl: Option<String>,
    pub priority: Option<u64>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct PIEpisodes {
    pub status: String,
    pub items: Vec<PIItem>,
    pub count: u64,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct PIEpisode {
    pub status: String,
    pub episode: PIItem,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PILiveItems {
    pub status: String,
    pub liveItems: Vec<PILiveItem>,
    pub count: u64,
    pub query: String,
    pub description: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PILiveItem {
    pub id: u64,
    pub title: String,
    pub link: String,
    pub description: String,
    pub guid: String,
    pub datePublished: u64,
    pub datePublishedPretty: String,
    pub enclosureUrl: String,
    pub enclosureType: String,
    pub startTime: Option<u64>,
    pub endTime: Option<u64>,
    pub status: String,
    pub contentLink: String,
    pub image: String,
    pub feedImage: String,
    pub feedId: u64,
    pub feedUrl: String,
    pub podcastGuid: String,
    pub feedItunesId: Option<u64>,
}

#[derive(Debug)]
pub struct HydraError(String);

impl fmt::Display for HydraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fatal error: {}", self.0)
    }
}

impl Error for HydraError {}

impl FromStr for InboxRequestObject {
    // This implementation of `from_str` can never fail, so use the impossible
    // `Void` type as the error type.
    type Err = Void;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Ok(Build {
        //     context: s.to_string(),
        //     dockerfile: None,
        //     args: Map::new(),
        // })
        Ok(InboxRequestObject {
            id: s.to_string(),
            r#type: None,
            actor: None,
            summary: None,
            inReplyTo: None,
            published: None,
            url: None,
            attributedTo: None,
            to: None,
            cc: None,
            sensitive: None,
            atomUri: None,
            inReplyToAtomUri: None,
            conversation: None,
            content: None,
            attachment: None,
            tag: None,
            replies: None,
        })
    }
}


//Operational Functions ------------------------------------------------------------------------------------------------
fn d_blank_inboxrequest() -> InboxRequestObject {
    InboxRequestObject {
        id: "".to_string(),
        r#type: None,
        actor: None,
        summary: None,
        inReplyTo: None,
        published: None,
        url: None,
        attributedTo: None,
        to: None,
        cc: None,
        sensitive: None,
        atomUri: None,
        inReplyToAtomUri: None,
        conversation: None,
        content: None,
        attachment: None,
        tag: None,
        replies: None,
    }
}

fn de_optional_string_or_struct<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: Deserialize<'de> + FromStr<Err=Void>,
        D: Deserializer<'de>,
{
    // This is a Visitor that forwards string types to T's `FromStr` impl and
    // forwards map types to T's `Deserialize` impl. The `PhantomData` is to
    // keep the compiler from complaining about T being an unused generic type
    // parameter. We need T in order to know the Value type for the Visitor
    // impl.
    struct StringOrStruct<T>(PhantomData<fn() -> T>);

    impl<'de, T> Visitor<'de> for StringOrStruct<T>
        where
            T: Deserialize<'de> + FromStr<Err=Void>,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or map")
        }

        fn visit_str<E>(self, value: &str) -> Result<T, E>
            where
                E: de::Error,
        {
            Ok(FromStr::from_str(value).unwrap())
        }

        fn visit_map<M>(self, map: M) -> Result<T, M::Error>
            where
                M: MapAccess<'de>,
        {
            // `MapAccessDeserializer` is a wrapper that turns a `MapAccess`
            // into a `Deserializer`, allowing it to be used as the input to T's
            // `Deserialize` implementation. T then deserializes itself using
            // the entries from the map visitor.
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))
        }
    }

    deserializer.deserialize_any(StringOrStruct(PhantomData))
}


//Endpoints ------------------------------------------------------------------------------------------------------------
pub async fn webfinger(ctx: Context) -> Response {

    //Get query parameters
    let params: HashMap<String, String> = ctx.req.uri().query().map(|v| {
        url::form_urlencoded::parse(v.as_bytes()).into_owned().collect()
    }).unwrap_or_else(HashMap::new);

    //Determine HTTP action
    let http_action = ctx.req.method().to_string();
    println!("\n\n----------");
    println!("Request[{}]: {} from: {:#?}",
             http_action,
             ctx.req.uri(),
             ctx.req.headers().get("user-agent")
    );
    //println!("Context: {:#?}", ctx);

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
    let api_response = api_get_podcast(
        &ctx.pi_auth.key,
        &ctx.pi_auth.secret,
        &podcast_guid,
    ).await;
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

    //Determine HTTP action
    let http_action = ctx.req.method().to_string();
    println!("\n\n----------");
    println!("Request[{}]: {} from: {:#?}",
             http_action,
             ctx.req.uri(),
             ctx.req.headers().get("user-agent")
    );
    //println!("Context: {:#?}", ctx);

    //Make sure a podcast id was given
    let guid;
    let mut no_guid = false;
    match params.get("id") {
        Some(resource) => {
            println!("  Id: {}\n", resource);
            let parts = resource.replace("acct:", "");
            guid = parts.split("@").next().unwrap().to_string();
        }
        None => {
            guid = "0".to_string();
            no_guid = true;
            // println!("Invalid resource.\n");
            // return hyper::Response::builder()
            //     .status(StatusCode::from_u16(400).unwrap())
            //     .body(format!("No resource given.").into())
            //     .unwrap();
        }
    }
    let podcast_guid = guid.clone();

    //##: Lookup API of podcast
    let podcast_data: PIPodcast;
    if podcast_guid != "0" {
        let api_response = api_get_podcast(
            &ctx.pi_auth.key,
            &ctx.pi_auth.secret,
            &podcast_guid,
        ).await;
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
    } else {    //##: Actor instance
        podcast_data = PIPodcast {
            status: "true".to_string(),
            feed: PIFeed {
                id: 0,
                podcastGuid: "".to_string(),
                medium: "".to_string(),
                title: "Podcast Index ActivityPub Bridge".to_string(),
                url: "".to_string(),
                originalUrl: "".to_string(),
                link: "".to_string(),
                description: "Podcast Index ActivityPub Bridge".to_string(),
                author: "Podcastindex.org".to_string(),
                ownerName: "Podcastindex.org".to_string(),
                image: "https://noagendaassets.com/enc/1684513486.722_pcifeedimage.png".to_string(),
                artwork: "https://noagendaassets.com/enc/1684513486.722_pcifeedimage.png".to_string(),
                episodeCount: 0,
                funding: None,
            },
        }
    }

    //##: If no keypair exists, create one
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
    let mut actor_data;
    match ap_build_actor_object(podcast_data, actor_keys) {
        Ok(data) => {
            actor_data = data;
            if no_guid {
                actor_data.id = "https://ap.podcastindex.org/podcasts".to_string();
            }
            if podcast_guid == "0" {
                actor_data.r#type = "Application".to_string();
                actor_data.name = Some("ap.podcastindex.org".to_string());
                actor_data.preferredUsername = Some("ap.podcastindex.org".to_string());
            }
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

    //Determine HTTP action
    let http_action = ctx.req.method().to_string();
    println!("\n\n----------");
    println!("Request[{}]: {} from: {:#?}",
             http_action,
             ctx.req.uri(),
             ctx.req.headers().get("user-agent")
    );
    //println!("Context: {:#?}", ctx);

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
    let api_response = api_get_podcast(
        &ctx.pi_auth.key,
        &ctx.pi_auth.secret,
        &podcast_guid,
    ).await;
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

    //Determine HTTP action
    let http_action = ctx.req.method().to_string();
    println!("\n\n----------");
    println!("Request[{}]: {} from: {:#?}",
             http_action,
             ctx.req.uri(),
             ctx.req.headers().get("user-agent")
    );
    //println!("Context: {:#?}", ctx);

    //Make sure a session param was given
    let guid;
    match params.get("id") {
        Some(resource) => {
            println!("  Id: {}\n", resource);
            let parts = resource.replace("acct:", "");
            guid = parts.split("@").next().unwrap().to_string();
        }
        None => {
            guid = "0".to_string();
            // println!("Invalid resource.\n");
            // return hyper::Response::builder()
            //     .status(StatusCode::from_u16(400).unwrap())
            //     .body(format!("No resource given.").into())
            //     .unwrap();
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
    if podcast_guid != "0" {
        let api_response = api_get_episodes(
            &ctx.pi_auth.key,
            &ctx.pi_auth.secret,
            &podcast_guid,
        ).await;
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
    } else {
        //##: TODO - this should contain all episodes probably
        podcast_data = PIEpisodes {
            status: "true".to_string(),
            items: vec![],
            count: 0,
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

    //Get query parameters
    let params: HashMap<String, String> = ctx.req.uri().query().map(|v| {
        url::form_urlencoded::parse(v.as_bytes()).into_owned().collect()
    }).unwrap_or_else(HashMap::new);

    //Determine HTTP action
    let http_action = ctx.req.method().to_string();
    println!("\n\n----------");
    println!("Request[{}]: {} from: {:#?}",
             http_action,
             ctx.req.uri(),
             ctx.req.headers().get("user-agent")
    );
    //println!("Context: {:#?}", ctx);

    //Make sure a podcast id was given
    let guid;
    match params.get("id") {
        Some(resource) => {
            println!("  Id: {}\n", resource);
            let parts = resource.replace("acct:", "");
            guid = parts.split("@").next().unwrap().to_string();
        }
        None => {
            eprintln!("Invalid resource: [{}]: [{}] from: {:#?}",
                      http_action,
                      ctx.req.uri(),
                      ctx.req.headers().get("user-agent")
            );
            return hyper::Response::builder()
                .status(StatusCode::from_u16(400).unwrap())
                .body(format!("No resource given.").into())
                .unwrap();
        }
    }
    let podcast_guid = guid.clone();

    let (_parts, body) = ctx.req.into_parts();
    let body_bytes = hyper::body::to_bytes(body).await.unwrap();
    let body = std::str::from_utf8(&body_bytes).unwrap();

    //##: Decode the inbox request JSON
    let incoming_data;
    match serde_json::from_str::<InboxRequestWithObject>(body) {
        Ok(decoded_inbox_request) => {
            //TODO: validate the key signature before accepting request
            incoming_data = decoded_inbox_request;
        }
        Err(e) => {
            eprintln!("Could not parse incoming request: [{}].\n", e);
            eprintln!("  BODY: {}", body);
            return hyper::Response::builder()
                .status(StatusCode::from_u16(400).unwrap())
                .body(format!("Invalid request format.").into())
                .unwrap();
        }
    }

    //##: Handle the request based on its type
    match incoming_data.r#type.to_lowercase().as_str() {
        "delete" => {
            println!("--Delete request: {:#?}", incoming_data);
            println!("  BODY: {}", body);
        }

        "follow" => {
            println!("--Follow request");
            println!("  FROM: [{}]", incoming_data.actor.clone().unwrap());
            match ap_block_get_remote_actor(incoming_data.actor.clone().unwrap()) {
                Ok(remote_actor) => {
                    //##: Construct a response
                    println!("  Building follow accept json.");
                    let accept_data;
                    match ap_build_follow_accept(incoming_data, podcast_guid.parse::<u64>().unwrap()) {
                        Ok(data) => {
                            accept_data = data;
                        }
                        Err(e) => {
                            eprintln!("Build follow accept error: [{:#?}].\n", e);
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
                            eprintln!("Response prep error: [{:#?}].\n", e);
                            return hyper::Response::builder()
                                .status(StatusCode::from_u16(500).unwrap())
                                .body(format!("Accept encode error.").into())
                                .unwrap();
                        }
                    }

                    //##: Send the accept request to the follower inbox url
                    println!("  Send the follow accept request.");
                    match ap_send_follow_accept(
                        podcast_guid.parse::<u64>().unwrap(),
                        accept_data,
                        remote_actor.inbox.clone(),
                    ) {
                        Ok(_) => {
                            let instance_fqdn = get_host_from_url(remote_actor.inbox.clone());
                            let shared_inbox;
                            match remote_actor.endpoints {
                                Some(endpoints) => {
                                    shared_inbox = endpoints.sharedInbox.clone();
                                }
                                None => {
                                    shared_inbox = remote_actor.inbox.clone();
                                }
                            }
                            match dbif::add_follower_to_db(&AP_DATABASE_FILE.to_string(), FollowerRecord {
                                pcid: podcast_guid.parse::<u64>().unwrap(),
                                actor: remote_actor.id.clone(),
                                instance: instance_fqdn,
                                inbox: remote_actor.inbox,
                                shared_inbox: shared_inbox,
                                status: "active".to_string(),
                            }) {
                                Ok(_) => {
                                    println!("Saved follow: [{}|{}]", podcast_guid, remote_actor.id);
                                }
                                Err(e) => {
                                    eprintln!("Unable to save follow state: [{}].\n", e);
                                    return hyper::Response::builder()
                                        .status(StatusCode::from_u16(400).unwrap())
                                        .body(format!("Unable to save follow state.").into())
                                        .unwrap();
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Acknowledging failed: [{}].\n", e);
                            return hyper::Response::builder()
                                .status(StatusCode::from_u16(400).unwrap())
                                .body(format!("Acknowledging failed.").into())
                                .unwrap();
                        }
                    };
                }
                Err(e) => {
                    eprintln!("Bad actor: [{}].\n", e);
                    return hyper::Response::builder()
                        .status(StatusCode::from_u16(400).unwrap())
                        .body(format!("Bad actor.").into())
                        .unwrap();
                }
            }
        }

        "undo" => {
            println!("--Unfollow request");
            if incoming_data.object.r#type.is_some()
                && incoming_data.object.r#type.as_ref().unwrap().to_lowercase() == "follow"
            {
                let _ = dbif::remove_follower_from_db(&AP_DATABASE_FILE.to_string(), FollowerRecord {
                    pcid: podcast_guid.parse::<u64>().unwrap(),
                    actor: incoming_data.actor.unwrap(),
                    instance: "".to_string(),
                    inbox: "".to_string(),
                    shared_inbox: "".to_string(),
                    status: "".to_string(),
                });
            }
        }

        "create" => {
            //##: Create
            println!("--Create request: {:#?}", incoming_data);
            println!("  BODY: {}", body);

            //##: Replies
            //##: Parse out the inReplyTo so we can determine which podcast this belongs to
            if incoming_data.object.inReplyTo.is_some()
                && incoming_data.object.content.is_some()
            {
                let in_reply_to_url = incoming_data.object.inReplyTo.clone().unwrap();
                let parent_pcid = get_id_from_url(in_reply_to_url.clone());
                let parent_episode_guid = get_statusid_from_url(in_reply_to_url.clone());
                let received_time = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time mismatch.").as_secs();
                let reply_conversation = incoming_data.object.conversation.unwrap_or("".to_string());

                if parent_pcid != "" && parent_episode_guid != ""
                {
                    let _ = dbif::add_reply_to_db(&AP_DATABASE_FILE.to_string(), ReplyRecord {
                        pcid: parent_pcid.parse::<u64>().unwrap(),
                        statusid: parent_episode_guid,
                        objectid: incoming_data.object.id.clone(),
                        objecttype: incoming_data.object.r#type.unwrap_or("".to_string()),
                        attributedto: incoming_data.object.attributedTo.clone().unwrap_or("".to_string()),
                        content: incoming_data.object.content.clone().unwrap(),
                        sensitive: 0,
                        published: incoming_data.object.published.unwrap_or(received_time.to_string()),
                        received: received_time,
                        conversation: reply_conversation.clone(),
                    });
                } else {
                    let replies = dbif::get_a_reply_by_conversation(
                        &AP_DATABASE_FILE.to_string(),
                        reply_conversation.clone(),
                    );

                    for reply in replies.unwrap() {
                        let _ = dbif::add_reply_to_db(&AP_DATABASE_FILE.to_string(), ReplyRecord {
                            pcid: reply.pcid,
                            statusid: reply.statusid,
                            objectid: incoming_data.object.id.clone(),
                            objecttype: incoming_data.object.r#type.unwrap_or("".to_string()),
                            attributedto: incoming_data.object.attributedTo.clone().unwrap_or("".to_string()),
                            content: incoming_data.object.content.clone().unwrap(),
                            sensitive: 0,
                            published: incoming_data.object.published.unwrap_or(received_time.to_string()),
                            received: received_time,
                            conversation: reply_conversation.clone(),
                        });
                        break;
                    }
                }
            }

            //##: PI Action Requests
            //##: We only deem the request an action if it's direct to the actor and not a reply
            if incoming_data.object.cc.is_some()
                && incoming_data.object.inReplyTo.is_none()
                && incoming_data.object.content.is_some()
            {
                for cc in incoming_data.object.cc.unwrap() {
                    let mut parent_pcid = 0;

                    //##: Check for a valid podcast id
                    match get_id_from_url(cc).parse::<u64>() {
                        Ok(pcid) => {
                            parent_pcid = pcid;
                        }
                        Err(e) => {
                            eprintln!("Failed to ascertain podcast id: [{}].\n", e);
                        }
                    }
                    if parent_pcid <= 0 {
                        continue;
                    }

                    //##: Handle the action requests
                    match incoming_data.object.content.clone().unwrap().trim() {

                        //##: Re-scan action
                        x if x.contains("rescan") => {
                            println!("  PI Action request: [{}|{}].", x, parent_pcid);
                            let _ = api_hub_rescan(
                                &ctx.pi_auth.key,
                                &ctx.pi_auth.secret,
                                &parent_pcid.to_string().as_str(),
                            ).await;

                            //##: If this request came from an actor, look them up and reply back
                            if incoming_data.object.attributedTo.clone().is_some() {
                                match ap_block_get_remote_actor(incoming_data.object.attributedTo.clone().unwrap()) {
                                    Ok(sending_actor) => {
                                        let _ = ap_block_send_note(
                                            parent_pcid,
                                            sending_actor.inbox,
                                            "Done.".to_string(),
                                            Some(incoming_data.object.id.clone()),
                                        );
                                    }
                                    Err(e) => {
                                        println!("  Can't get actor from action request: [{:#?}|\n{}]", e, parent_pcid);
                                    }
                                }
                            }
                        }

                        //##: Send me the latest episode
                        x if x.contains("latest") => {
                            //##: If this request came from an actor, look them up and reply back
                            if incoming_data.object.attributedTo.clone().is_some() {
                                match ap_block_get_remote_actor(incoming_data.object.attributedTo.clone().unwrap()) {
                                    Ok(sending_actor) => {
                                        match api_block_get_episodes(
                                            &ctx.pi_auth.key,
                                            &ctx.pi_auth.secret,
                                            &parent_pcid.to_string().as_str(),
                                        ) {
                                            Ok(response_body) => {
                                                match serde_json::from_str(response_body.as_str()) {
                                                    Ok(data) => {
                                                        let podcast_data: PIEpisodes = data;
                                                        //##: TODO Get this code out of this deep level of nesting
                                                        let latest_episode = podcast_data.items.get(0);
                                                        if latest_episode.is_some() {
                                                            let latest_episode_details = latest_episode.unwrap();

                                                            let _ = ap_block_send_episode_note(
                                                                parent_pcid,
                                                                latest_episode_details,
                                                                sending_actor.inbox,
                                                                true,
                                                                Some(incoming_data.object.id),
                                                            );
                                                        }
                                                    }
                                                    Err(e) => {
                                                        //##: TODO - refactor this deep nesting
                                                        eprintln!("  API response prep error: [{:#?}] actor guid: [{}].\n",
                                                                  e,
                                                                  parent_pcid.to_string().as_str()
                                                        );
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                //##: TODO - refactor this deep nesting
                                                eprintln!("  PI API call error: [{:#?}].\n", e);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        println!("  Can't get actor from action request: [{:#?}|\n{}]", e, parent_pcid);
                                    }
                                }
                            }
                        }

                        //##: Unhandled action
                        _ => {
                            println!("  Unsupported PI action request: [|{}]", parent_pcid);
                        }
                    }

                    break;
                }
            }
        }

        _ => {
            println!("--Unhandled request: {:#?}", incoming_data);
            println!("  BODY: {}", body);
        }
    }

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

    //Determine HTTP action
    let http_action = ctx.req.method().to_string();
    println!("\n\n----------");
    println!("Request[{}]: {} from: {:#?}",
             http_action,
             ctx.req.uri(),
             ctx.req.headers().get("user-agent")
    );
    //println!("Context: {:#?}", ctx);

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

    //Determine HTTP action
    let http_action = ctx.req.method().to_string();
    println!("\n\n----------");
    println!("Request[{}]: {} from: {:#?}",
             http_action,
             ctx.req.uri(),
             ctx.req.headers().get("user-agent")
    );
    //println!("Context: {:#?}", ctx);

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
    let episode_json;
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
            attachment: Some(vec!()),
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
    } else {
        //##: Lookup episode in the PI API
        let pi_data: PIEpisode;
        let api_response = api_get_episode(
            &ctx.pi_auth.key,
            &ctx.pi_auth.secret,
            &podcast_guid,
            &episode_guid,
        ).await;

        match api_response {
            Ok(response_body) => {
                //eprintln!("{:#?}", response_body);
                match serde_json::from_str(response_body.as_str()) {
                    Ok(data) => {
                        pi_data = data;

                        let timestamp_param = format!(
                            "&ts={}",
                            pi_data.episode.datePublished
                        );
                        let episode_object = build_episode_note_object(
                            &pi_data.episode,
                            podcast_guid.parse::<u64>().unwrap(),
                            "Title".to_string(),
                            timestamp_param
                        ).unwrap();

                        let episode_data = Status {
                            at_context: vec!(
                                "https://www.w3.org/ns/activitystreams".to_string(),
                            ),
                            id: format!(
                                "https://ap.podcastindex.org/episodes?id={}&statusid={}",
                                podcast_guid,
                                episode_guid
                            ).to_string(),
                            r#type: "Note".to_string(),
                            summary: None,
                            inReplyTo: None,
                            published: iso8601(pi_data.episode.datePublished.clone()),
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
                                "https://ap.podcastindex.org/contexts?id={}&statusid={}",
                                podcast_guid,
                                episode_guid
                            ).to_string(),
                            content: episode_object.content,
                            attachment: Some(episode_object.attachment),
                            actor: format!("https://ap.podcastindex.org/podcasts?id={}", podcast_guid).to_string(),
                            tag: vec!(),
                            replies: None,
                        };

                        //##: Are there replies to include?


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
    }

    return hyper::Response::builder()
        .status(StatusCode::OK)
        .header("Content-type", "application/activity+json; charset=utf-8")
        .body(format!("{}", episode_json).into())
        .unwrap();
}

pub async fn contexts(ctx: Context) -> Response {

    //Determine HTTP action
    let http_action = ctx.req.method().to_string();
    println!("\n\n----------");
    println!("Request[{}]: {} from: {:#?}",
             http_action,
             ctx.req.uri(),
             ctx.req.headers().get("user-agent")
    );
    //println!("Context: {:#?}", ctx);

    return hyper::Response::builder()
        .status(StatusCode::OK)
        .header("Content-type", "application/activity+json; charset=utf-8")
        .body(format!("").into())
        .unwrap();
}

pub async fn followers(ctx: Context) -> Response {

    //Determine HTTP action
    let http_action = ctx.req.method().to_string();
    println!("\n\n----------");
    println!("Request[{}]: {} from: {:#?}",
             http_action,
             ctx.req.uri(),
             ctx.req.headers().get("user-agent")
    );
    //println!("Context: {:#?}", ctx);

    return hyper::Response::builder()
        .status(StatusCode::OK)
        .header("Content-type", "application/activity+json; charset=utf-8")
        .body(format!("").into())
        .unwrap();
}


//API calls --------------------------------------------------------------------------------------------------
pub async fn api_get_podcast(key: &str, secret: &str, query: &str) -> Result<String, Box<dyn Error>> {
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
    headers.insert("X-Auth-Key", header::HeaderValue::from_str(api_key).unwrap());
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

pub async fn api_hub_rescan(key: &str, secret: &str, query: &str) -> Result<String, Box<dyn Error>> {
    println!("PI HUB Request: /pubnotify?id={}", query);

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
    let url: String = format!("https://hub.podcastindex.org/pubnotify?id={}", urlencoding::encode(query));

    //##: Build the query with the required headers
    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", header::HeaderValue::from_static("Rust-podcastindex-org-example/v1.0"));
    headers.insert("X-Auth-Date", header::HeaderValue::from_str(api_time.as_str()).unwrap());
    headers.insert("X-Auth-Key", header::HeaderValue::from_str(api_key).unwrap());
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

pub async fn api_get_episodes(key: &str, secret: &str, query: &str) -> Result<String, Box<dyn Error>> {
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
    headers.insert("X-Auth-Key", header::HeaderValue::from_str(api_key).unwrap());
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

pub async fn api_get_episode(key: &str, secret: &str, query: &str, guid: &str) -> Result<String, Box<dyn Error>> {
    println!("  PI API Request: /episodes/byguid");

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
    let url: String = format!(
        "https://api.podcastindex.org/api/1.0/episodes/byguid?guid={}&feedid={}",
        urlencoding::encode(guid),
        urlencoding::encode(query)
    );

    println!("  PI API Request: [{}]", url);

    //##: Build the query with the required headers
    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", header::HeaderValue::from_static("Rust-podcastindex-org-example/v1.0"));
    headers.insert("X-Auth-Date", header::HeaderValue::from_str(api_time.as_str()).unwrap());
    headers.insert("X-Auth-Key", header::HeaderValue::from_str(api_key).unwrap());
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

pub fn api_block_get_episodes(key: &str, secret: &str, query: &str) -> Result<String, Box<dyn Error>> {
    //println!("  PI API Request: /episodes/byfeedid");

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
    headers.insert("X-Auth-Key", header::HeaderValue::from_str(api_key).unwrap());
    headers.insert("Authorization", header::HeaderValue::from_str(api_hash.as_str()).unwrap());
    let client = reqwest::blocking::Client::builder().default_headers(headers).build().unwrap();

    //##: Send the request and display the results or the error
    let res = client.get(url.as_str()).send();
    match res {
        Ok(res) => {
            //println!("  Response: [{}]", res.status());
            return Ok(res.text().unwrap());
        }
        Err(e) => {
            //eprintln!("  Error: [{}]", e);
            return Err(Box::new(HydraError(format!("Error running SQL query: [{}]", e).into())));
        }
    }
}

pub fn api_block_get_live_items(key: &str, secret: &str, query: &str) -> Result<String, Box<dyn Error>> {
    println!("  PI API Request: /live/byfeedurl");

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
    let url: String = format!("https://api.podcastindex.org/api/1.0/live/byfeedurl?url={}", urlencoding::encode(query));

    //##: Build the query with the required headers
    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", header::HeaderValue::from_static("Rust-podcastindex-org-example/v1.0"));
    headers.insert("X-Auth-Date", header::HeaderValue::from_str(api_time.as_str()).unwrap());
    headers.insert("X-Auth-Key", header::HeaderValue::from_str(api_key).unwrap());
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
            return Err(Box::new(HydraError(format!("Error making pi api request: [{}]", e).into())));
        }
    }
}


//ActivityPub helper functions -------------------------------------------------------------------------------
fn ap_build_actor_object(podcast_data: PIPodcast, actor_keys: ActorKeys) -> Result<Actor, Box<dyn Error>> {
    let podcast_guid = podcast_data.feed.id;

    let mut actor = Actor {
        at_context: vec!(
            "https://www.w3.org/ns/activitystreams".to_string(),
            "https://w3id.org/security/v1".to_string(),
        ),
        id: format!("https://ap.podcastindex.org/podcasts?id={}", podcast_guid).to_string(),
        r#type: "Service".to_string(),
        discoverable: Some(true),
        preferredUsername: Some(podcast_guid.to_string()),
        name: Some(format!("{:.48}", podcast_data.feed.title).to_string()),
        inbox: format!("https://ap.podcastindex.org/inbox?id={}", podcast_guid).to_string(),
        outbox: format!("https://ap.podcastindex.org/outbox?id={}", podcast_guid).to_string(),
        featured: Some(format!("https://ap.podcastindex.org/featured?id={}", podcast_guid).to_string()),
        followers: Some(format!("https://ap.podcastindex.org/followers?id={}", podcast_guid).to_string()),
        following: Some(format!("https://ap.podcastindex.org/following?id={}", podcast_guid).to_string()),
        icon: Some(Icon {
            r#type: "Image".to_string(),
            mediaType: None,
            url: format!("{}", podcast_data.feed.image).to_string(),
        }),
        summary: Some(format!("{:.96}", podcast_data.feed.description)),
        attachment: None,
        publicKey: PublicKey {
            id: format!("https://ap.podcastindex.org/podcasts?id={}#main-key", podcast_guid).to_string(),
            owner: format!("https://ap.podcastindex.org/podcasts?id={}", podcast_guid).to_string(),
            publicKeyPem: actor_keys.pem_public_key,
        },
        endpoints: Some(Endpoints {
            sharedInbox: "https://ap.podcastindex.org/inbox?id=0".to_string(),
        }),
        url: Some(format!("https://podcastindex.org/podcast/{}", podcast_guid).to_string()),
        manuallyApprovesFollowers: Some(false),
        indexable: Some(true),
        memorial: Some(false),
        published: Some("2023-11-09T15:56:28.495803Z".to_string()),
        devices: None,
        //alsoKnownAs: None,
        //tag: vec!(),
    };

    //##: Attachments
    let mut attachments = vec!(
        Attachment {
            name: Some("Index".to_string()),
            r#type: "PropertyValue".to_string(),
            value: Some(format!(
                "<a href='https://podcastindex.org/podcast/{}' rel='ugc'>https://podcastindex.org/podcast/{}</a>",
                podcast_guid,
                podcast_guid,
            ).to_string()),
        },
        Attachment {
            name: Some("Website".to_string()),
            r#type: "PropertyValue".to_string(),
            value: Some(format!(
                "<a href='{}' rel='ugc'>{}</a>",
                podcast_data.feed.link,
                podcast_data.feed.link,
            ).to_string()),
        },
        Attachment {
            name: Some("RSS".to_string()),
            r#type: "PropertyValue".to_string(),
            value: Some(format!(
                "<a href='{}' rel='ugc'>{}</a>",
                podcast_data.feed.url,
                podcast_data.feed.url
            ).to_string()),
        },
        Attachment {
            name: Some("Podcast Guid".to_string()),
            r#type: "PropertyValue".to_string(),
            value: Some(format!(
                "{}",
                podcast_data.feed.podcastGuid,
            ).to_string()),
        },
    );
    //##: Funding tag present?
    match podcast_data.feed.funding {
        Some(funding) => {
            let funding_url = funding.url.unwrap_or("".to_string());
            let funding_message = funding.message.unwrap_or(funding_url.clone());

            attachments.push(
                Attachment {
                    name: Some("Funding".to_string()),
                    r#type: "PropertyValue".to_string(),
                    value: Some(format!(
                        "<a href='{}' rel='ugc'>{}</a>",
                        funding_url,
                        funding_message
                    ).to_string()),
                }
            );
        }
        None => {}
    }
    actor.attachment = Some(attachments);

    return Ok(actor);
}

fn ap_build_follow_accept(follow_request: InboxRequestWithObject, podcast_guid: u64) -> Result<InboxRequestAccept, Box<dyn Error>> {
    return Ok(
        InboxRequestAccept {
            at_context: "https://www.w3.org/ns/activitystreams".to_string(),
            id: format!("https://ap.podcastindex.org/podcasts?id={}&context=accept", podcast_guid).to_string(),
            r#type: "Accept".to_string(),
            actor: follow_request.object.id.clone(),
            object: InboxRequest {
                id: follow_request.id.clone(),
                r#type: follow_request.r#type.clone(),
                actor: follow_request.actor.clone().unwrap(),
                object: follow_request.object.id.clone(),
            },
        }
    );
}

fn ap_get_actor_keys(podcast_guid: u64) -> Result<ActorKeys, Box<dyn Error>> {
    println!("  Getting actor keys for: [{}]", podcast_guid);

    let actor_keys;
    let pem_pub_key;
    let pem_priv_key;
    match dbif::get_actor_from_db(&AP_DATABASE_FILE.to_string(), podcast_guid) {
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

            let _ = dbif::add_actor_to_db(&AP_DATABASE_FILE.to_string(), ActorRecord {
                pcid: podcast_guid,
                guid: "".to_string(),
                pem_private_key: pem_priv_key.clone(),
                pem_public_key: pem_pub_key.clone(),
                last_episode_guid: "".to_string(),
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

pub fn ap_send_follow_accept(podcast_guid: u64, inbox_accept: InboxRequestAccept, inbox_url: String) -> Result<String, Box<dyn Error>> {
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
            eprintln!("Error decoding private key: [{}] for actor: [{}]", e, podcast_guid);
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
            eprintln!("Error building post body: [{}]", e);
            return Err(Box::new(HydraError(format!("Error building post body: [{}]", e).into())));
        }
    }
    println!("  POST BODY: {}", post_body);

    let key_id = format!("https://ap.podcastindex.org/podcasts?id={}#main-key", podcast_guid);
    let http_signature_headers;
    match http_signature::create_http_signature(
        http::Method::POST,
        &inbox_url,
        &post_body.clone(),
        &private_key,
        &key_id,
    ) {
        Ok(sig_headers) => {
            http_signature_headers = sig_headers;
        }
        Err(e) => {
            eprintln!("Could not build http signature headers: [{}]", e);
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
    println!("  ACCEPT SENT: [{}]", inbox_url.as_str());
    match client
        .post(inbox_url.as_str())
        .body(post_body)
        .send()
    {
        Ok(res) => {
            println!("  Response: [{:#?}]", res);
            if res.status() >= StatusCode::from_u16(200)? && res.status() <= StatusCode::from_u16(299)? {
                return Ok("".to_string());
            } else {
                let res_body = res.text().unwrap();
                eprintln!("  Body: [{:#?}]", res_body);
                return Err(Box::new(HydraError(format!("Accepting the follow request failed.").into())));
            }
        }
        Err(e) => {
            eprintln!("  Error: [{}]", e);
            return Err(Box::new(HydraError(format!("Error sending follow accept request: [{}]", e).into())));
        }
    }
}

pub fn ap_block_send_note(podcast_guid: u64, inbox_url: String, note: String, in_reply_to: Option<String>) -> Result<String, Box<dyn Error>> {
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

    //##: Construct the note object to send
    let mut create_action_object = Create {
        at_context: "https://www.w3.org/ns/activitystreams".to_string(),
        id: format!(
            "https://ap.podcastindex.org/notes?id={}&statusid={}&resource=activity",
            podcast_guid,
            SystemTime::now().duration_since(UNIX_EPOCH).expect("Time mismatch.").as_secs()
        ).to_string(),
        r#type: "Create".to_string(),
        actor: format!("https://ap.podcastindex.org/podcasts?id={}", podcast_guid).to_string(),
        published: iso8601(SystemTime::now().duration_since(UNIX_EPOCH).expect("Time mismatch.").as_secs()),
        to: vec!(
            "https://www.w3.org/ns/activitystreams#Public".to_string()
        ),
        cc: None,
        object: Object {
            id: format!(
                "https://ap.podcastindex.org/notes?id={}&statusid={}&resource=post",
                podcast_guid,
                SystemTime::now().duration_since(UNIX_EPOCH).expect("Time mismatch.").as_secs()
            ).to_string(),
            r#type: "Note".to_string(),
            summary: None,
            inReplyTo: None,
            published: iso8601(SystemTime::now().duration_since(UNIX_EPOCH).expect("Time mismatch.").as_secs()),
            url: format!(
                "https://ap.podcastindex.org/notes?id={}&statusid={}&resource=public",
                podcast_guid,
                SystemTime::now().duration_since(UNIX_EPOCH).expect("Time mismatch.").as_secs()
            ).to_string(),
            attributedTo: format!("https://ap.podcastindex.org/podcasts?id={}", podcast_guid).to_string(),
            to: vec!(
                "https://www.w3.org/ns/activitystreams#Public".to_string()
            ),
            cc: None,
            sensitive: false,
            conversation: format!(
                "tag:ap.podcastindex.org,{}:objectId={}:objectType=Conversation",
                iso8601(SystemTime::now().duration_since(UNIX_EPOCH).expect("Time mismatch.").as_secs()),
                SystemTime::now().duration_since(UNIX_EPOCH).expect("Time mismatch.").as_secs()
            ).to_string(),
            content: format!("{}", note),
            attachment: vec!(),
        },
    };

    //##: Is this in reply to another post
    if in_reply_to.is_some() {
        create_action_object.object.inReplyTo = Some(in_reply_to.unwrap());
    }

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
    let http_signature_headers;
    match http_signature::create_http_signature(
        http::Method::POST,
        &inbox_url,
        &create_json.clone(),
        &private_key,
        &key_id,
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
    println!("  NOTE SENT: [{}|{}|{}]", podcast_guid, note, inbox_url.as_str());
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

pub fn ap_block_send_episode_note(
    podcast_guid: u64,
    episode: &PIItem,
    inbox_url: String,
    requested: bool,
    in_reply_to: Option<String>,
) -> Result<String, Box<dyn Error>> {
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

    //##: If this object was built by user requesting it then it needs a timestamp parameter in it's object id so
    //##: that it will show up as a new object in their AP timeline.  Otherwise it just comes in as an old post and
    //##: gets sorted below the fold
    let mut intro_text = "New Episode".to_string();
    let mut timestamp_param = "".to_string();
    if requested {
        timestamp_param = format!(
            "&ts={}",
            SystemTime::now().duration_since(UNIX_EPOCH).expect("Time mismatch.").as_secs()
        );
        intro_text = "Latest Episode".to_string();
    }

    //##: Build the episode object
    let episode_object = build_episode_note_object(
        episode,
        podcast_guid,
        intro_text,
        timestamp_param.clone()
    ).unwrap();

    //##: Build the Create action that will hold the episode object
    let mut create_action_object = Create {
        at_context: "https://www.w3.org/ns/activitystreams".to_string(),
        id: format!(
            "https://ap.podcastindex.org/episodes?id={}&statusid={}&resource=activity{}",
            podcast_guid,
            episode.guid,
            timestamp_param
        ).to_string(),
        r#type: "Create".to_string(),
        actor: format!("https://ap.podcastindex.org/podcasts?id={}", podcast_guid).to_string(),
        published: iso8601(episode.datePublished),
        to: vec!(
            "https://www.w3.org/ns/activitystreams#Public".to_string()
        ),
        cc: None,
        object: episode_object,
    };

    //##: Is this in reply to another post
    if in_reply_to.is_some() {
        create_action_object.object.inReplyTo = Some(in_reply_to.unwrap());
    }

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
    let http_signature_headers;
    match http_signature::create_http_signature(
        http::Method::POST,
        &inbox_url,
        &create_json.clone(),
        &private_key,
        &key_id,
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

pub fn ap_block_send_live_note(podcast_guid: u64, episode: &PILiveItem, inbox_url: String) -> Result<String, Box<dyn Error>> {
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
            "https://ap.podcastindex.org/episodes?id={}&statusid={}&date={}&type=live&resource=activity",
            podcast_guid,
            episode.guid,
            episode.datePublished
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
                "https://ap.podcastindex.org/episodes?id={}&statusid={}&date={}&type=live&resource=post",
                podcast_guid,
                episode.guid,
                episode.datePublished
            ).to_string(),
            r#type: "Note".to_string(),
            summary: None,
            inReplyTo: None,
            published: iso8601(episode.datePublished),
            url: format!(
                "https://ap.podcastindex.org/episodes?id={}&statusid={}&date={}&type=live&resource=public",
                podcast_guid,
                episode.guid,
                episode.datePublished
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
                "<p>Listen Live!</p>\
                 <p>{:.256}</p>\
                 <p>\
                 <a href=\"https://curiocaster.com/podcast/pi{}\">CurioCaster</a><br>\
                 <a href=\"https://fountain.fm/show/{}\">Fountain</a><br>\
                 <a href=\"https://podcastaddict.com/feed/{}\">Podcast Addict</a><br>\
                 <a href=\"https://app.podcastguru.io/podcast/{}\">Podcast Guru</a><br>\
                 <a href=\"https://api.podverse.fm/api/v1/podcast/podcastindex/{}\">Podverse</a><br>\
                 <a href=\"https://truefans.fm/{}\">Truefans</a>\
                 </p>\
                 <p>Or <a href=\"{}\">Stream</a> here.</p>",
                episode.title,
                episode.feedId,
                episode.feedId,
                episode.feedUrl,
                episode.feedItunesId.unwrap_or(0),
                episode.feedId,
                episode.podcastGuid,
                episode.enclosureUrl,
            ),
            attachment: vec!(
                NoteAttachment {
                    r#type: Some("Document".to_string()),
                    mediaType: None,
                    url: Some(episode.feedImage.clone()),
                    name: None,
                    blurhash: None,
                    width: Some(640),
                    height: None,
                    value: None,
                }
            ),
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
    let http_signature_headers;
    match http_signature::create_http_signature(
        http::Method::POST,
        &inbox_url,
        &create_json.clone(),
        &private_key,
        &key_id,
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

pub fn ap_block_get_remote_actor(actor_url: String) -> Result<Actor, Box<dyn Error>> {
    println!("  AP Get Remote Actor: {}", actor_url);

    //##: Build the query with the required headers
    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", header::HeaderValue::from_static("Podcast Index AP/v0.1.2a"));
    headers.insert("Accept", header::HeaderValue::from_static("application/activity+json"));
    let client = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    //##: Send the Accept request
    println!("  ACTOR REQUEST SENT: [{}]", actor_url.as_str());
    let res = client
        .get(actor_url.as_str())
        .send();
    match res {
        Ok(response) => {
            println!("  Response: [{:#?}]", response);
            if response.status() >= StatusCode::from_u16(200)? && response.status() <= StatusCode::from_u16(299)? {
                match serde_json::from_str::<Actor>(response.text().unwrap().as_str()) {
                    Ok(actor_data) => {
                        return Ok(actor_data);
                    }
                    Err(e) => {
                        eprintln!("Could not parse incoming request: [{}].\n", e);
                        return Err(Box::new(HydraError(format!("Error parsing remote actor data: [{}]", e).into())));
                    }
                }
            } else {
                let res_body = response.text().unwrap();
                eprintln!("  Actor Body: [{:#?}]", res_body);
                return Err(Box::new(HydraError(format!("Getting remote actor failed.").into())));
            }
        }
        Err(e) => {
            eprintln!("  Error: [{}]", e);
            return Err(Box::new(HydraError(format!("Error getting remote actor: [{}]", e).into())));
        }
    }
}

//##: Construct an ActivityPub note object from a PI API episode object
pub fn build_episode_note_object(
    episode: &PIItem,
    podcast_guid: u64,
    intro_text: String,
    timestamp_param: String
) -> Result<Object, Box<dyn Error>> {

    //##: If a social interact url exists, show it as a clickable link
    let mut episode_social_interact_display = "".to_string();
    match &episode.socialInteract {
        Some(social_interacts) => {
            for social_interact in social_interacts {
                let episode_social_interact_uri = social_interact.uri.clone().unwrap_or("".to_string());
                episode_social_interact_display = format!(
                    "<p><a href=\"{}\">Comments Thread</a></p>",
                    episode_social_interact_uri
                );
                break;
            }
        }
        None => {
            episode_social_interact_display = "".to_string();
        }
    }

    //##: If a transcript url exists, show it as a link to a transcript viewer service
    let mut episode_transcript_display = "".to_string();
    match &episode.transcripts {
        Some(transcripts) => {
            for _transcript in transcripts {
                episode_transcript_display = format!(
                    "<p><a href=\"https://steno.fm/show/{}/episode/{}\">Transcript</a></p>",
                    episode.podcastGuid,
                    general_purpose::STANDARD.encode(&episode.guid.clone().as_bytes())
                );
                break;
            }
        }
        None => {
            episode_transcript_display = "".to_string();
        }
    }

    //##: Show episode art, or feed art if no episode level art exists
    let episode_image = match episode.image.as_str() {
        "" => {
            episode.feedImage.clone()
        }
        _ => {
            episode.image.clone()
        }
    };

    //##: Put it all together
    return Ok(Object {
        id: format!(
            "https://ap.podcastindex.org/episodes?id={}&statusid={}&resource=post{}",
            podcast_guid,
            episode.guid,
            timestamp_param
        ).to_string(),
        r#type: "Note".to_string(),
        summary: None,
        inReplyTo: None,
        published: iso8601(episode.datePublished),
        url: format!(
            "https://ap.podcastindex.org/episodes?id={}&statusid={}&resource=public{}",
            podcast_guid,
            episode.guid,
            timestamp_param
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
            "<p>{}: <a href=\"https://podcastindex.org/podcast/{}?episode={}\">{:.256}</a></p>\
                 <p>Shownotes:<br>{:.256}</p>\
                 {}\
                 {}\
                 <p>\
                   <a href=\"https://antennapod.org/deeplink/subscribe?url={}\">AntennaPod</a> | \
                   <a href=\"https://anytimeplayer.app/subscribe?url={}\">Anytime Player</a> | \
                   <a href=\"https://podcasts.apple.com/podcast/id{}\">Apple Podcasts</a> | \
                   <a href=\"https://castamatic.com/guid/{}\">Castamatic</a> | \
                   <a href=\"https://curiocaster.com/podcast/pi{}\">CurioCaster</a> | \
                   <a href=\"https://fountain.fm/show/{}\">Fountain</a> | \
                   <a href=\"https://gpodder.net/subscribe?url={}\">gPodder</a> | \
                   <a href=\"https://overcast.fm/itunes{}\">Overcast</a> | \
                   <a href=\"https://pcasts.in/feed/{}\">Pocket Casts</a> | \
                   <a href=\"https://podcastaddict.com/feed/{}\">Podcast Addict</a> | \
                   <a href=\"https://app.podcastguru.io/podcast/{}\">Podcast Guru</a> | \
                   <a href=\"https://podnews.net/podcast/pi{}\">Podnews</a> | \
                   <a href=\"https://api.podverse.fm/api/v1/podcast/podcastindex/{}\">Podverse</a> | \
                   <a href=\"https://truefans.fm/{}\">Truefans</a>\
                 </p>\
                 <p>Or <a href=\"{}\">Listen</a> right here.</p>\
                ",
            intro_text,
            episode.feedId,
            episode.id,
            episode.title,
            episode.description,
            episode_social_interact_display,
            episode_transcript_display,
            episode.feedUrl,
            episode.feedUrl,
            episode.feedItunesId.unwrap_or(0),
            episode.podcastGuid,
            episode.feedId,
            episode.feedId,
            episode.feedUrl,
            episode.feedItunesId.unwrap_or(0),
            episode.feedUrl,
            episode.feedUrl,
            episode.feedItunesId.unwrap_or(0),
            episode.feedId,
            episode.feedId,
            episode.podcastGuid,
            episode.enclosureUrl
        ),
        attachment: vec!(
            NoteAttachment {
                r#type: Some("Document".to_string()),
                mediaType: None,
                url: Some(episode_image),
                name: None,
                blurhash: None,
                width: Some(640),
                height: None,
                value: None,
            }
        ),
    });
}

//Utilities --------------------------------------------------------------------------------------------------
fn iso8601(utime: u64) -> String {

    // Create DateTime from SystemTime
    let datetime = Utc.timestamp_opt(utime as i64, 0).unwrap();

    // Formats the combined date and time with the specified format string.
    datetime.format("%+").to_string()
}

fn get_host_from_url(url: String) -> String {
    let request_url_object = url::Url::parse(&url).unwrap();
    request_url_object.host_str()
        .ok_or(url::ParseError::EmptyHost)
        .unwrap()
        .to_string()
}

fn get_id_from_url(url: String) -> String {
    let url = url::Url::parse(&url).unwrap();
    for pair in url.query_pairs() {
        if pair.0 == "id" {
            return pair.1.to_string();
        }
    }

    return "".to_string();
}

fn get_statusid_from_url(url: String) -> String {
    let url = url::Url::parse(&url).unwrap();
    for pair in url.query_pairs() {
        if pair.0 == "statusid" {
            return pair.1.to_string();
        }
    }

    return "".to_string();
}
