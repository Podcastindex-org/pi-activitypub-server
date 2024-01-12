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

## Operation

Followers of podcasts are recorded and when new episodes are posted a Note is sent.  A Note is also sent when 
podcasts go live, by watching the LiveWire Podping websocket.

## Database

The bridge uses a SQLite file for it's DB and will auto-create the file if one is not present.

## To-do

- Watch for replies and return them in the episode status
- Note content cleanup
- AS2 object for media references

## Contributing

Outside contributions are very welcome.

## Version history

v0.0.5 - Initial stages of reply handling.