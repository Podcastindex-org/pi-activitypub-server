use hyper::{body::to_bytes, service::{make_service_fn, service_fn}, Body, Request, Server};
use route_recognizer::Params;
use router::Router;
use std::sync::Arc;
use hyper::server::conn::AddrStream;
use std::env;
use std::string::ToString;
use std::thread;
use std::time::{Duration};
use serde::{Deserialize, Serialize};
use crate::handler::{
    api_block_get_episodes,
    ap_block_send_episode_note,
    ap_block_send_live_note,
    PIEpisodes,
    PILiveItems,
    api_block_get_live_items
};
use url::Url;
use tungstenite::{connect};

//Globals ----------------------------------------------------------------------------------------------------
mod handler;
mod router;
mod http_signature;
mod crypto_rsa;
mod base64;

const LOOP_TIMER_MILLISECONDS: u64 = 60000;
const AP_DATABASE_FILE: &str = "database.db";
const USER_AGENT_PARAM: &str = concat!("PodcastIndexAPBridge_v", env!("CARGO_PKG_VERSION"));
//const USER_AGENT_HEADER: &str = concat!("Podcast Index AP-Bridge/v", env!("CARGO_PKG_VERSION"));

type Response = hyper::Response<hyper::Body>;
type Error = Box<dyn std::error::Error + Send + Sync + 'static>;


//Structs ----------------------------------------------------------------------------------------------------
#[derive(Clone, Debug)]
pub struct AppState {
    pub state_thing: String,
    pub remote_ip: String,
}

#[derive(Debug)]
pub struct Context {
    pub state: AppState,
    pub req: Request<Body>,
    pub params: Params,
    pub pi_auth: PIAuth,
    pub version: String,
    body_bytes: Option<hyper::body::Bytes>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SocketPayload {
    pub a: String,
    pub n: u64,
    pub o: u64,
    pub p: Vec<PodpingPayload>,
    pub t: String,
    pub v: u64,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PodpingPayload {
    pub a: String,
    pub i: String,
    pub p: Podping,
    pub t: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Podping {
    pub iris: Vec<String>,
    pub medium: Option<String>,
    pub reason: String,
    pub sessionId: Option<String>,
    pub timestampNs: Option<u64>,
    pub version: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Clone, Debug)]
pub struct PIAuth {
    pub key: String,
    pub secret: String,
}


//Functions --------------------------------------------------------------------------------------------------
#[tokio::main]
async fn main() {
    //##: Get what version we are
    let version = env!("CARGO_PKG_VERSION");
    println!("Version: {}", version);
    println!("--------------------");

    let args: Vec<String> = env::args().collect();
    let arg_port = &args[1];
    //let arg_chatid = &args[2];

    let env_pi_api_key;
    match std::env::var("PI_API_KEY") {
        Ok(key) => {
            env_pi_api_key = key;
        }
        Err(_) => {
            eprintln!("PI_API_KEY environment variable not set.");
            std::process::exit(1);
        }
    }
    let env_pi_api_secret;
    match std::env::var("PI_API_SECRET") {
        Ok(secret) => {
            env_pi_api_secret = secret;
        }
        Err(_) => {
            eprintln!("PI_API_SECRET environment variable not set.");
            std::process::exit(1);
        }
    }

    //##: TODO: these must handle errors better
    //##: Make sure we have a good database
    if dbif::create_database(&AP_DATABASE_FILE.to_string()).is_err() {
        eprintln!("Error initializing the database file.");
    }

    //##: Start threads to track podcast new episodes and also podping
    let env_tracker_pi_api_key = env_pi_api_key.clone();
    let env_tracker_pi_api_secret = env_pi_api_secret.clone();
    thread::spawn(move || {
        loop {
            let key = env_tracker_pi_api_key.clone();
            let secret = env_tracker_pi_api_secret.clone();
            let thread_handle = thread::spawn(move || {
                episode_tracker(key, secret);
            });
            match thread_handle.join() {
                Ok(_) => {
                    println!("*****Episode Tracker Thread Exited*****");
                }
                Err(e) => {
                    eprintln!("*****Episode Tracker Thread Exited*****:  [{:#?}]", e);
                }
            }
        }
    });

    let env_live_pi_api_key = env_pi_api_key.clone();
    let env_live_pi_api_secret = env_pi_api_secret.clone();
    thread::spawn(move || {
        loop {
            let key = env_live_pi_api_key.clone();
            let secret = env_live_pi_api_secret.clone();
            let thread_handle = thread::spawn(move || {
                live_item_tracker(key, secret);
            });
            match thread_handle.join() {
                Ok(_) => {
                    println!("*****Live Tracker Thread Exited*****");
                }
                Err(e) => {
                    eprintln!("*****Live Tracker Thread Exited*****:  [{:#?}]", e);
                }
            }
        }
    });

    let some_state = "state".to_string();

    let mut router: Router = Router::new();
    router.get("/profiles", Box::new(handler::profiles)); //##: User profile html page
    router.get("/podcasts", Box::new(handler::podcasts)); //##: Actor profile page OUT
    router.post("/podcasts", Box::new(handler::podcasts)); //##: Actor profile page IN
    router.get("/inbox", Box::new(handler::inbox)); //##: User inbox OUT
    router.post("/inbox", Box::new(handler::inbox)); //##: User inbox IN
    router.get("/outbox", Box::new(handler::outbox)); //##: User outbox OUT
    router.post("/outbox", Box::new(handler::outbox)); //##: User outbox IN
    router.get("/featured", Box::new(handler::featured)); //##: Featured posts
    router.get("/episodes", Box::new(handler::episodes)); //##: Statuses
    router.get("/contexts", Box::new(handler::contexts)); //##: Contexts
    router.get("/followers", Box::new(handler::followers)); //##: Followers
    router.get("/.well-known/webfinger", Box::new(handler::webfinger)); //##: Webfinger

    let shared_router = Arc::new(router);
    let new_service = make_service_fn(move |conn: &AddrStream| {
        let app_state = AppState {
            state_thing: some_state.clone(),
            remote_ip: conn.remote_addr().to_string().clone(),
        };

        let router_capture = shared_router.clone();
        let pi_auth = PIAuth {
            key: env_pi_api_key.clone(),
            secret: env_pi_api_secret.clone(),
        };
        let main_version = version.to_string();
        async {
            Ok::<_, Error>(service_fn(move |req| {
                route(
                    router_capture.clone(),
                    req,
                    app_state.clone(),
                    pi_auth.clone(),
                    main_version.clone(),
                )
            }))
        }
    });

    let binding = format!("0.0.0.0:{}", arg_port);
    let addr = binding.parse().expect("address creation works");
    let server = Server::bind(&addr).serve(new_service);
    println!("Listening on http://{}", addr);

    let _ = server.await;
}

async fn route(
    router: Arc<Router>,
    req: Request<hyper::Body>,
    app_state: AppState,
    pi_auth: PIAuth,
    version: String
) -> Result<Response, Error> {
    let found_handler = router.route(req.uri().path(), req.method());
    let resp = found_handler
        .handler
        .invoke(Context::new(
            app_state,
            req,
            found_handler.params,
            pi_auth,
            version
        ))
        .await;
    Ok(resp)
}

impl Context {
    pub fn new(
        state: AppState,
        req: Request<Body>,
        params: Params,
        pi_auth: PIAuth,
        version: String,
    ) -> Context {
        Context {
            state,
            req,
            params,
            pi_auth,
            version,
            body_bytes: None,
        }
    }

    pub async fn body_json<T: serde::de::DeserializeOwned>(&mut self) -> Result<T, Error> {
        let body_bytes = match self.body_bytes {
            Some(ref v) => v,
            _ => {
                let body = to_bytes(self.req.body_mut()).await?;
                self.body_bytes = Some(body);
                self.body_bytes.as_ref().expect("body_bytes was set above")
            }
        };
        Ok(serde_json::from_slice(&body_bytes)?)
    }
}

fn episode_tracker(api_key: String, api_secret: String) {
    loop {
        thread::sleep(Duration::from_millis(LOOP_TIMER_MILLISECONDS));

        println!("TRACKER: Polling podcast data.");

        let actors;
        match dbif::get_actors_from_db(&AP_DATABASE_FILE.to_string()) {
            Ok(actor_list) => {
                actors = actor_list;
            }
            Err(e) => {
                eprintln!("  Error getting actors from the database: [{:#?}]", e);
                continue;
            }
        }

        let mut actor_count = 0;
        for actor in actors {
            //##: Skip instance actor
            if actor.pcid == 0 {
                continue;
            }

            match dbif::get_followers_from_db(&AP_DATABASE_FILE.to_string(), actor.pcid) {
                Ok(followers) => {
                    if followers.len() > 0 {
                        actor_count += 1;
                    }

                    //##: Lookup API of podcast
                    println!("  Podcast API Call - [{}]", actor.pcid);
                    match api_block_get_episodes(
                        &api_key,
                        &api_secret,
                        &actor.pcid.to_string()
                    ) {
                        Ok(response_body) => {
                            match serde_json::from_str(response_body.as_str()) {
                                Ok(data) => {
                                    let podcast_data: PIEpisodes = data;
                                    //##: TODO Get this code out of this deep level of nesting
                                    let latest_episode = podcast_data.items.get(0);
                                    if latest_episode.is_some() {
                                        let latest_episode_details = latest_episode.unwrap();
                                        if actor.last_episode_guid != latest_episode_details.guid {
                                            let mut shared_inboxes_called = Vec::new();
                                            for follower in followers {
                                                if !shared_inboxes_called.contains(&follower.shared_inbox) {
                                                    let _ = ap_block_send_episode_note(
                                                        actor.pcid,
                                                        latest_episode_details,
                                                        follower.shared_inbox.clone(),
                                                        false,
                                                        None
                                                    );
                                                    shared_inboxes_called.push(follower.shared_inbox.clone());
                                                }
                                            }

                                            let _ = dbif::update_actor_last_episode_guid_in_db(
                                                &AP_DATABASE_FILE.to_string(),
                                                actor.pcid,
                                                latest_episode_details.guid.clone(),
                                            );
                                        }
                                    }
                                }
                                Err(e) => {
                                    //##: TODO - refactor this deep nesting
                                    eprintln!("  API response prep error: [{:#?}] actor guid: [{}].\n", e, actor.pcid);
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
                    //##: TODO - refactor this deep nesting
                    eprintln!("  Error getting followers from the database: [{:#?}]", e);
                    continue;
                }
            }

            thread::sleep(Duration::from_millis(500));
        }

        println!("TRACKER: [{}] podcasts being followed.", actor_count);
    }
}

fn live_item_tracker(api_key: String, api_secret: String) {

    println!("PODPING: Connected to podping socket.");

    //##: TODO - reconnect socket if it falls down
    let (mut socket, _response) = connect(
        Url::parse(format!("wss://api.livewire.io/ws/podping?agent={}", USER_AGENT_PARAM).as_str()).unwrap()
    ).expect("Can't connect to podping socket.");

    loop {
        #[allow(deprecated)]
        let msg = socket.read_message().expect("Error reading message");
        //println!(" Podping Received: {:#?}", msg.to_text().unwrap());
        match serde_json::from_str(msg.to_text().unwrap()) {
            Ok(data) => {
                let socket_payload: SocketPayload = data;
                // println!("PODPING: [{:#?}]", socket_payload);
                for podping in socket_payload.p {
                    if podping.p.reason == "live" {
                        println!("*****LIVE PODPING: [{:#?}]", podping);
                        let first_iri = podping.p.iris.get(0);
                        if first_iri.is_none() {
                            continue;
                        }
                        //##: Sleep to let the index catch up
                        thread::sleep(Duration::from_millis(LOOP_TIMER_MILLISECONDS));
                        match api_block_get_live_items(
                            &api_key,
                            &api_secret,
                            first_iri.unwrap()
                        ) {
                            Ok(api_response) => {
                                match serde_json::from_str(api_response.as_str()) {
                                    Ok(response_data) => {
                                        let live_item_data: PILiveItems = response_data;
                                        for live_item in live_item_data.liveItems {
                                            if live_item.status == "live" {
                                                println!("*****PODPING LIVE - {} {}",
                                                         live_item.feedId,
                                                         live_item.status
                                                );
                                                match dbif::get_followers_from_db(&AP_DATABASE_FILE.to_string(), live_item.feedId) {
                                                    Ok(followers) => {
                                                        let mut shared_inboxes_called = Vec::new();
                                                        for follower in followers {
                                                            if !shared_inboxes_called.contains(&follower.shared_inbox) {
                                                                let _ = ap_block_send_live_note(
                                                                    live_item.feedId,
                                                                    &live_item,
                                                                    follower.shared_inbox.clone(),
                                                                );
                                                                shared_inboxes_called.push(follower.shared_inbox.clone());
                                                            }
                                                        }
                                                    }
                                                    Err(_e) => {
                                                        //##: TODO - refactor this deep nesting
                                                    }
                                                }
                                                break;
                                            }
                                        }
                                    }
                                    Err(_e) => {
                                        //##: TODO - refactor this deep nesting
                                    }
                                }
                            }
                            Err(_e) => {
                                //##: TODO - refactor this deep nesting
                            }
                        }

                    }
                }
            }
            Err(e) => {
                eprintln!("PODPING PARSE ERR: [{:#?}]", e);
            }
        }
    }
}