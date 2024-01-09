# pi-activitypub-server
This is an ActivityPub bridge that allows ActivityPub users (like Mastodon accounts) to follow podcasts 
as if they were ActivityPub users (actors).  It gets the podcast information from the Podcast Index API.

## Running
You can run a bridge by downloading the source and compiling it with:

```bash
export PI_API_KEY="[YOUR KEY]"
export PI_API_SECRET="[YOUR SECRET]"
cargo build --release && ./target/release/pi-activitypub-server 80 1
```

The bridge requires a Podcast Index API key set to be present in the environment, as noted above.