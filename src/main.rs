use hyper::{
    body::to_bytes,
    service::{make_service_fn, service_fn},
    Body, Request, Server,
};
use route_recognizer::Params;
use router::Router;
use std::sync::Arc;
use hyper::server::conn::AddrStream;
//use std::thread;
//use std::time;
//use tokio::task;
use std::env;
//use drop_root::set_user_group;

//Globals ----------------------------------------------------------------------------------------------------
//const ZMQ_SOCKET_ADDR: &str = "tcp://127.0.0.1:5555";

mod handler;
mod router;

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
    let arg_chatid = &args[2];

    //TODO: these must handle errors better
    //Make sure we have a good database
    if dbif::create_database(&"ap.db".to_string()).is_err() {
        eprintln!("Error initializing the database file.");
    }


    let some_state = "state".to_string();

    let mut router: Router = Router::new();
    router.get("/profiles", Box::new(handler::profiles)); //User profile html page
    router.get("/podcasts", Box::new(handler::podcasts)); //JSON activity page
    router.get("/inbox", Box::new(handler::inbox)); //User inbox
    router.get("/outbox", Box::new(handler::outbox)); //User outbox
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