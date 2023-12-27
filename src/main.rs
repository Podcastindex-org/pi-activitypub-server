use hyper::{body::to_bytes, service::{make_service_fn, service_fn}, Body, Request, Server, StatusCode};
use route_recognizer::Params;
use router::Router;
use std::sync::Arc;
use hyper::server::conn::AddrStream;
//use std::thread;
//use std::time;
//use tokio::task;
use std::env;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::handler::{api_block_get_episodes, ap_block_send_note, API_KEY, API_SECRET, PIEpisodes};
//use drop_root::set_user_group;

//Globals ----------------------------------------------------------------------------------------------------
mod handler;
mod router;
mod http_signature;
mod crypto_rsa;
mod base64;

const LOOP_TIMER_MILLISECONDS: u64 = 9000;
const AP_DATABASE_FILE: &str = "database.db";

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
    body_bytes: Option<hyper::body::Bytes>,
}

//Functions --------------------------------------------------------------------------------------------------
#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let arg_port = &args[1];
    //let arg_chatid = &args[2];

    //TODO: these must handle errors better
    //Make sure we have a good database
    if dbif::create_database(&AP_DATABASE_FILE.to_string()).is_err() {
        eprintln!("Error initializing the database file.");
    }

    //Start the LND polling thread.  This thread will poll LND every few seconds to
    //get the latest invoices and store them in the database.
    thread::spawn(move || {
        episode_tracker()
    });

    let some_state = "state".to_string();

    let mut router: Router = Router::new();
    router.get("/profiles", Box::new(handler::profiles)); //User profile html page
    router.get("/podcasts", Box::new(handler::podcasts)); //Actor profile page OUT
    router.post("/podcasts", Box::new(handler::podcasts)); //Actor profile page IN
    router.get("/inbox", Box::new(handler::inbox)); //User inbox OUT
    router.post("/inbox", Box::new(handler::inbox)); //User inbox IN
    router.get("/outbox", Box::new(handler::outbox)); //User outbox OUT
    router.post("/outbox", Box::new(handler::outbox)); //User outbox IN
    router.get("/featured", Box::new(handler::featured)); //Featured posts
    router.get("/episodes", Box::new(handler::episodes)); //Statuses
    router.get("/contexts", Box::new(handler::contexts)); //Contexts
    router.get("/followers", Box::new(handler::followers)); //Followers
    router.get("/.well-known/webfinger", Box::new(handler::webfinger)); //Webfinger

    let shared_router = Arc::new(router);
    let new_service = make_service_fn(move |conn: &AddrStream| {
        let app_state = AppState {
            state_thing: some_state.clone(),
            remote_ip: conn.remote_addr().to_string().clone(),
        };

        let router_capture = shared_router.clone();
        async {
            Ok::<_, Error>(service_fn(move |req| {
                route(router_capture.clone(), req, app_state.clone())
            }))
        }
    });

    let binding = format!("0.0.0.0:{}", arg_port);
    let addr = binding.parse().expect("address creation works");
    let server = Server::bind(&addr).serve(new_service);
    println!("Listening on http://{}", addr);

    //If a "run as" user is set in the "PODPING_RUN_AS" environment variable, then switch to that user
    //and drop root privileges after we've bound to the low range socket
    // match env::var("PODPING_RUNAS_USER") {
    //     Ok(runas_user) => {
    //         match set_user_group(runas_user.as_str(), "nogroup") {
    //             Ok(_) => {
    //                 println!("RunAs: {}", runas_user.as_str());
    //             }
    //             Err(e) => {
    //                 eprintln!("RunAs Error: {} - Check that your PODPING_RUNAS_USER env var is set correctly.", e);
    //             }
    //         }
    //     }
    //     Err(_) => {
    //         eprintln!("ALERT: Use the PODPING_RUNAS_USER env var to avoid running as root.");
    //     }
    // }

    let _ = server.await;
}

async fn route(
    router: Arc<Router>,
    req: Request<hyper::Body>,
    app_state: AppState,
) -> Result<Response, Error> {
    let found_handler = router.route(req.uri().path(), req.method());
    let resp = found_handler
        .handler
        .invoke(Context::new(app_state, req, found_handler.params))
        .await;
    Ok(resp)
}

impl Context {
    pub fn new(state: AppState, req: Request<Body>, params: Params) -> Context {
        Context {
            state,
            req,
            params,
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

fn episode_tracker() {
    //TODO some sort of polling here against the PI API to detect when new episodes arrive for followed podcasts
    //and then send them out to followers of those podcasts
    thread::sleep(Duration::from_millis(LOOP_TIMER_MILLISECONDS));
    println!("TRACKER: Polling podcast data.");

    loop {
        //##: Lookup API of podcast
        println!("  Podcast - 920666");
        match api_block_get_episodes(API_KEY, API_SECRET, "920666") {
            Ok(response_body) => {
                //eprintln!("{:#?}", response_body);
                match serde_json::from_str(response_body.as_str()) {
                    Ok(data) => {
                        let podcast_data: PIEpisodes = data;
                        //TODO Get this code out of this deep level of nesting
                        let latest_episode = podcast_data.items.get(0);
                        if latest_episode.is_some() {
                            ap_block_send_note(
                                920666,
                                latest_episode.unwrap(),
                                "https://podcastindex.social/inbox".to_string(),
                            );
                            ap_block_send_note(
                                920666,
                                latest_episode.unwrap(),
                                "https://noagendasocial.com/inbox".to_string(),
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!("  API response prep error: [{:#?}].\n", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("  API call error: [{:#?}].\n", e);
            }
        }

        break;
    }
}