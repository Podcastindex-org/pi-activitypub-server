use rusqlite::{params, Connection};
use std::error::Error;
use std::fmt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::os::unix::fs::PermissionsExt;


#[derive(Serialize, Deserialize, Debug)]
pub struct BoostRecord {
    pub index: u64,
    pub time: i64,
    pub value_msat: i64,
    pub value_msat_total: i64,
    pub action: u8,
    pub sender: String,
    pub app: String,
    pub message: String,
    pub podcast: String,
    pub episode: String,
    pub tlv: String,
    pub remote_podcast: Option<String>,
    pub remote_episode: Option<String>,
}

impl BoostRecord {
    //Removes unsafe html interpretable characters from displayable strings
    pub fn escape_for_html(field: String) -> String {
        return field.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;");
    }

    //Removes unsafe html interpretable characters from displayable strings
    pub fn escape_for_csv(field: String) -> String {
        return field.replace("\"", "\"\"").replace("\n", " ");
    }

    //Parses the TLV record into a Value
    pub fn parse_tlv(&self) -> Result<Value, Box<dyn Error>> {
        return Ok(serde_json::from_str(self.tlv.as_str())?);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActorRecord {
    pub pcid: u64,
    pub guid: String,
    pub pem_private_key: String,
    pub pem_public_key: String,
    pub last_episode_guid: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FollowerRecord {
    pub pcid: u64,
    pub actor: String,
    pub instance: String,
    pub inbox: String,
    pub shared_inbox: String,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReplyRecord {
    pub pcid: u64,
    pub statusid: String,
    pub objectid: String,
    pub objecttype: String,
    pub attributedto: String,
    pub content: String,
    pub sensitive: u64,
    pub published: String,
    pub received: u64,
    pub conversation: String,
}

#[derive(Debug)]
struct HydraError(String);

impl fmt::Display for HydraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fatal error: {}", self.0)
    }
}

impl Error for HydraError {}


//Connect to the database at the given file location
fn connect_to_database(init: bool, filepath: &String) -> Result<Connection, Box<dyn Error>> {
    if let Ok(conn) = Connection::open(filepath.as_str()) {
        if init {
            match set_database_file_permissions(filepath.as_str()) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{:#?}", e);
                }
            }
            println!("Using database file: [{}]", filepath.as_str());
        }
        Ok(conn)
    } else {
        return Err(Box::new(HydraError(format!("Could not open a database file at: [{}].", filepath).into())));
    }
}


//Set permissions on the database files
fn set_database_file_permissions(filepath: &str) -> Result<bool, Box<dyn Error>> {
    match std::fs::File::open(filepath) {
        Ok(fh) => {
            match fh.metadata() {
                Ok(metadata) => {
                    let mut perms = metadata.permissions();
                    perms.set_mode(0o666);
                    println!("Set file permission to: [666] on database file: [{}]", filepath);
                    Ok(true)
                }
                Err(e) => {
                    return Err(Box::new(HydraError(format!("Error getting metadata from database file handle: [{}].  Error: {:#?}.", filepath, e).into())));
                }
            }
        }
        Err(e) => {
            return Err(Box::new(HydraError(format!("Error opening database file handle: [{}] for permissions setting.  Error: {:#?}.", filepath, e).into())));
        }
    }
}


//Create or update a new database file if needed
pub fn create_database(filepath: &String) -> Result<bool, Box<dyn Error>> {
    let conn = connect_to_database(true, filepath)?;

    //Create the actors table
    match conn.execute(
        "CREATE TABLE IF NOT EXISTS actors (
             pcid integer primary key,
             guid text,
             pem_private_key text,
             pem_public_key text,
             last_episode_guid text
         )",
        [],
    ) {
        Ok(_) => {
            println!("Actors table is ready.");
        }
        Err(e) => {
            eprintln!("{}", e);
            return Err(Box::new(HydraError(format!("Failed to create database actors table: [{}].", filepath).into())));
        }
    }

    //Create indexes on the actors table
    match conn.execute(
        "CREATE INDEX IF NOT EXISTS guid_idx ON actors (guid)",
        [],
    ) {
        Ok(_) => {
            println!("Actors indexes created.");
        }
        Err(e) => {
            eprintln!("{}", e);
            return Err(Box::new(HydraError(format!("Failed to create database actors indexes: [{}].", filepath).into())));
        }
    }

    //Create the followers table
    match conn.execute(
        "CREATE TABLE IF NOT EXISTS followers (
             pcid integer,
             actor text,
             instance text,
             inbox text,
             shared_inbox text,
             status text
         )",
        [],
    ) {
        Ok(_) => {
            println!("Followers table is ready.");
        }
        Err(e) => {
            eprintln!("{}", e);
            return Err(Box::new(HydraError(format!("Failed to create database followers table: [{}].", filepath).into())));
        }
    }

    //Create indexes on the followers table
    match conn.execute(
        "CREATE INDEX IF NOT EXISTS shared_inbox_idx ON followers (shared_inbox)",
        [],
    ) {
        Ok(_) => {
            println!("Followers index created.");
        }
        Err(e) => {
            eprintln!("{}", e);
            return Err(Box::new(HydraError(format!("Failed to create database followers index: [{}].", filepath).into())));
        }
    }

    match conn.execute(
        "CREATE INDEX IF NOT EXISTS pcid_idx ON followers (pcid)",
        [],
    ) {
        Ok(_) => {
            println!("Followers index created.");
        }
        Err(e) => {
            eprintln!("{}", e);
            return Err(Box::new(HydraError(format!("Failed to create database followers index: [{}].", filepath).into())));
        }
    }

    match conn.execute(
        "CREATE INDEX IF NOT EXISTS actor_idx ON followers (actor)",
        [],
    ) {
        Ok(_) => {
            println!("Followers index created.");
        }
        Err(e) => {
            eprintln!("{}", e);
            return Err(Box::new(HydraError(format!("Failed to create database followers index: [{}].", filepath).into())));
        }
    }

    match conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS unique_follow_idx ON followers (pcid,actor)",
        [],
    ) {
        Ok(_) => {
            println!("Followers index created.");
        }
        Err(e) => {
            eprintln!("{}", e);
            return Err(Box::new(HydraError(format!("Failed to create database followers index: [{}].", filepath).into())));
        }
    }

    //Create the replies table
    match conn.execute(
        "CREATE TABLE IF NOT EXISTS replies (
             pcid integer,
             statusid text,
             objectid text,
             objecttype text,
             attributedto text,
             content text,
             sensitive integer,
             published text,
             received integer
         )",
        [],
    ) {
        Ok(_) => {
            println!("Replies table is ready.");
        }
        Err(e) => {
            eprintln!("{}", e);
            return Err(Box::new(HydraError(format!("Failed to create database replies table: [{}].", filepath).into())));
        }
    }

    match conn.execute(
        "CREATE INDEX IF NOT EXISTS pcid_idx ON replies (pcid)",
        [],
    ) {
        Ok(_) => {
            println!("Replies index created.");
        }
        Err(e) => {
            eprintln!("{}", e);
            return Err(Box::new(HydraError(format!("Failed to create database replies index: [{}].", filepath).into())));
        }
    }

    match conn.execute(
        "CREATE INDEX IF NOT EXISTS statusid_idx ON replies (statusid)",
        [],
    ) {
        Ok(_) => {
            println!("Replies index created.");
        }
        Err(e) => {
            eprintln!("{}", e);
            return Err(Box::new(HydraError(format!("Failed to create database replies index: [{}].", filepath).into())));
        }
    }

    match conn.execute(
        "CREATE INDEX IF NOT EXISTS received_idx ON replies (received)",
        [],
    ) {
        Ok(_) => {
            println!("Replies index created.");
        }
        Err(e) => {
            eprintln!("{}", e);
            return Err(Box::new(HydraError(format!("Failed to create database replies index: [{}].", filepath).into())));
        }
    }

    match conn.execute(
        "ALTER TABLE replies ADD conversation text",
        [],
    ) {
        Ok(_) => {
            println!("Conversation column created in replies table.");
        }
        Err(e) => {
            eprintln!("{}", e);
            //return Err(Box::new(HydraError(format!("Failed to add column to replies table: [{}].", filepath).into())));
        }
    }

    match conn.execute(
        "CREATE INDEX IF NOT EXISTS conversation_idx ON replies (conversation)",
        [],
    ) {
        Ok(_) => {
            println!("Replies index created.");
        }
        Err(e) => {
            eprintln!("{}", e);
            return Err(Box::new(HydraError(format!("Failed to create database replies index: [{}].", filepath).into())));
        }
    }

    match conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS objectid_idx ON replies (objectid)",
        [],
    ) {
        Ok(_) => {
            println!("Replies index created.");
        }
        Err(e) => {
            eprintln!("{}", e);
            return Err(Box::new(HydraError(format!("Failed to create database replies index: [{}].", filepath).into())));
        }
    }

    Ok(true)
}


//GetSet an actor in the database
pub fn add_actor_to_db(filepath: &String, actor: ActorRecord) -> Result<bool, Box<dyn Error>> {
    let conn = connect_to_database(false, filepath)?;

    match conn.execute("INSERT INTO actors (\
                                      pcid, \
                                      guid, \
                                      pem_private_key, \
                                      pem_public_key, \
                                      last_episode_guid \
                                    ) \
                        VALUES (?1, ?2, ?3, ?4, ?5)",
                       params![
                           actor.pcid,
                           actor.guid,
                           actor.pem_private_key,
                           actor.pem_public_key,
                           actor.last_episode_guid
                       ],
    ) {
        Ok(_) => {
            Ok(true)
        }
        Err(e) => {
            eprintln!("{}", e);
            return Err(Box::new(HydraError(format!("Failed to add actor: [{}].", actor.pcid).into())));
        }
    }
}

pub fn update_actor_last_episode_guid_in_db(filepath: &String, pcid: u64, episode_guid: String) -> Result<bool, Box<dyn Error>> {
    let conn = connect_to_database(false, filepath)?;

    match conn.execute("UPDATE actors \
                            SET last_episode_guid = ?1 \
                            WHERE pcid = ?2",
                       params![
                           episode_guid,
                           pcid,
                       ],
    ) {
        Ok(_) => {
            Ok(true)
        }
        Err(e) => {
            eprintln!("{}", e);
            return Err(Box::new(HydraError(format!("Failed to update guid: [{}] for pcid: [{}].", episode_guid, pcid).into())));
        }
    }
}

pub fn get_actor_from_db(filepath: &String, pcid: u64) -> Result<ActorRecord, Box<dyn Error>> {
    let conn = connect_to_database(false, filepath)?;
    let mut actors: Vec<ActorRecord> = Vec::new();
    let max = 1;

    //Prepare and execute the query
    let mut stmt = conn.prepare("SELECT \
                                    pcid, \
                                    guid,\
                                    pem_private_key, \
                                    pem_public_key, \
                                    last_episode_guid \
                                 FROM actors \
                                 WHERE pcid = :pcid \
                                 ORDER BY pcid DESC \
                                 LIMIT :max")?;
    let rows = stmt.query_map(&[(":max", max.to_string().as_str()), (":pcid", pcid.to_string().as_str())], |row| {
        Ok(ActorRecord {
            pcid: row.get(0)?,
            guid: row.get(1)?,
            pem_private_key: row.get(2)?,
            pem_public_key: row.get(3)?,
            last_episode_guid: row.get(4)?,
        })
    }).unwrap();

    //Parse the results
    for row in rows {
        let actor: ActorRecord = row.unwrap();
        actors.push(actor);
    }

    if actors.len() > 0 {
        return Ok(actors[0].clone());
    }


    Err(Box::new(HydraError(format!("Failed to get actor: [{}].", pcid).into())))
}

pub fn get_actors_from_db(filepath: &String) -> Result<Vec<ActorRecord>, Box<dyn Error>> {
    let conn = connect_to_database(false, filepath)?;
    let mut actors: Vec<ActorRecord> = Vec::new();
    let max = 1000; //TODO: debug - hard limit for now

    //Prepare and execute the query
    let mut stmt = conn.prepare("SELECT \
                                    pcid, \
                                    guid,\
                                    pem_private_key, \
                                    pem_public_key, \
                                    last_episode_guid \
                                 FROM actors \
                                 ORDER BY pcid ASC \
                                 LIMIT :max")?;
    let rows = stmt.query_map(&[(":max", max.to_string().as_str())], |row| {
        Ok(ActorRecord {
            pcid: row.get(0)?,
            guid: row.get(1)?,
            pem_private_key: row.get(2)?,
            pem_public_key: row.get(3)?,
            last_episode_guid: row.get(4)?,
        })
    }).unwrap();

    //Parse the results
    for row in rows {
        let actor: ActorRecord = row.unwrap();
        actors.push(actor);
    }

    if actors.len() > 0 {
        return Ok(actors.clone());
    }


    Err(Box::new(HydraError(format!("Failed to get any actors.").into())))
}


//GetSet a follower in the database
pub fn add_follower_to_db(filepath: &String, follower: FollowerRecord) -> Result<bool, Box<dyn Error>> {
    let conn = connect_to_database(false, filepath)?;

    match conn.execute("INSERT INTO followers (\
                                      pcid, \
                                      actor, \
                                      instance, \
                                      inbox, \
                                      shared_inbox, \
                                      status \
                                    ) \
                        VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                       params![
                           follower.pcid,
                           follower.actor,
                           follower.instance,
                           follower.inbox,
                           follower.shared_inbox,
                           follower.status
                       ],
    ) {
        Ok(_) => {
            Ok(true)
        }
        Err(e) => {
            eprintln!("{}", e);
            return Err(Box::new(HydraError(format!("Failed to add follower: [{}].", follower.actor).into())));
        }
    }
}

pub fn remove_follower_from_db(filepath: &String, follower: FollowerRecord) -> Result<bool, Box<dyn Error>> {
    let conn = connect_to_database(false, filepath)?;

    match conn.execute("DELETE FROM followers WHERE pcid=?1 AND actor=?2",
                       params![
                           follower.pcid,
                           follower.actor,
                       ],
    ) {
        Ok(_) => {
            Ok(true)
        }
        Err(e) => {
            eprintln!("{}", e);
            return Err(Box::new(HydraError(format!("Failed to remove follower: [{}].", follower.actor).into())));
        }
    }
}

pub fn get_followers_from_db(filepath: &String, pcid: u64) -> Result<Vec<FollowerRecord>, Box<dyn Error>> {
    let conn = connect_to_database(false, filepath)?;
    let mut followers: Vec<FollowerRecord> = Vec::new();
    let max = 1000; //TODO: debug - hard limit for now

    //Prepare and execute the query
    let mut stmt = conn.prepare("SELECT \
                                    pcid, \
                                    actor,\
                                    instance, \
                                    inbox, \
                                    shared_inbox, \
                                    status \
                                 FROM followers \
                                 WHERE pcid = :pcid \
                                 ORDER BY instance DESC \
                                 LIMIT :max")?;
    let rows = stmt.query_map(
        &[
            (":max", max.to_string().as_str()),
            (":pcid", pcid.to_string().as_str())
        ],
        |row| {
            Ok(FollowerRecord {
                pcid: row.get(0)?,
                actor: row.get(1)?,
                instance: row.get(2)?,
                inbox: row.get(3)?,
                shared_inbox: row.get(4)?,
                status: row.get(5)?,
            })
        }).unwrap();

    //Parse the results
    for row in rows {
        let follower: FollowerRecord = row.unwrap();
        followers.push(follower);
    }

    return Ok(followers.clone());


    //Err(Box::new(HydraError(format!("Failed to get followers for: [{}].", pcid).into())))
}


//GetSet a reply in the database
pub fn add_reply_to_db(filepath: &String, reply: ReplyRecord) -> Result<bool, Box<dyn Error>> {
    let conn = connect_to_database(false, filepath)?;

    match conn.execute("INSERT INTO replies (\
                                         pcid, \
                                         statusid, \
                                         objectid, \
                                         objecttype, \
                                         attributedto, \
                                         content, \
                                         sensitive, \
                                         published, \
                                         received,\
                                         conversation \
                                    ) \
                        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                       params![
                           reply.pcid,
                           reply.statusid,
                           reply.objectid,
                           reply.objecttype,
                           reply.attributedto,
                           reply.content,
                           reply.sensitive,
                           reply.published,
                           reply.received,
                           reply.conversation
                       ],
    ) {
        Ok(_) => {
            Ok(true)
        }
        Err(e) => {
            eprintln!("{}", e);
            return Err(Box::new(HydraError(format!("Failed to add reply: [{}].", reply.objectid).into())));
        }
    }
}
//TODO
pub fn remove_reply_from_db(filepath: &String, reply: ReplyRecord) -> Result<bool, Box<dyn Error>> {
    let conn = connect_to_database(false, filepath)?;

    match conn.execute("DELETE FROM replies WHERE objectid=?",
                       params![
                           reply.objectid,
                       ],
    ) {
        Ok(_) => {
            Ok(true)
        }
        Err(e) => {
            eprintln!("{}", e);
            return Err(Box::new(HydraError(format!("Failed to remove reply: [{}].", reply.objectid).into())));
        }
    }
}

pub fn get_replies_from_db_by_episode(filepath: &String, pcid: u64, statusid: String) -> Result<Vec<ReplyRecord>, Box<dyn Error>> {
    let conn = connect_to_database(false, filepath)?;
    let mut replies: Vec<ReplyRecord> = Vec::new();
    let max = 1000; //TODO: debug - hard limit for now

    //Prepare and execute the query
    let mut stmt = conn.prepare("SELECT \
                                     pcid, \
                                     statusid, \
                                     objectid, \
                                     objecttype, \
                                     attributedto, \
                                     content, \
                                     sensitive, \
                                     published, \
                                     received \
                                 FROM replies \
                                 WHERE pcid = :pcid \
                                   AND statusid = :statusid \
                                 ORDER BY received DESC \
                                 LIMIT :max")?;
    let rows = stmt.query_map(
        &[
            (":max", max.to_string().as_str()),
            (":pcid", pcid.to_string().as_str()),
            (":statusid", statusid.as_str())
        ],
        |row| {
            Ok(ReplyRecord {
                pcid: row.get(0)?,
                statusid: row.get(1)?,
                objectid: row.get(2)?,
                objecttype: row.get(3)?,
                attributedto: row.get(4)?,
                content: row.get(5)?,
                sensitive: row.get(6)?,
                published: row.get(7)?,
                received: row.get(8)?,
                conversation: row.get(9)?,
            })
        }).unwrap();

    //Parse the results
    for row in rows {
        let reply: ReplyRecord = row.unwrap();
        replies.push(reply);
    }

    return Ok(replies.clone());


    //Err(Box::new(HydraError(format!("Failed to get followers for: [{}].", pcid).into())))
}

pub fn get_a_reply_by_conversation(filepath: &String, conversation: String) -> Result<Vec<ReplyRecord>, Box<dyn Error>> {
    let conn = connect_to_database(false, filepath)?;
    let mut replies: Vec<ReplyRecord> = Vec::new();

    //Prepare and execute the query
    let mut stmt = conn.prepare("SELECT \
                                     pcid, \
                                     statusid, \
                                     objectid, \
                                     objecttype, \
                                     attributedto, \
                                     content, \
                                     sensitive, \
                                     published, \
                                     received,\
                                     conversation \
                                 FROM replies \
                                 WHERE conversation = :conversation \
                                 ORDER BY received DESC")?;
    let rows = stmt.query_map(
        &[
            (":conversation", conversation.as_str())
        ],
        |row| {
            Ok(ReplyRecord {
                pcid: row.get(0)?,
                statusid: row.get(1)?,
                objectid: row.get(2)?,
                objecttype: row.get(3)?,
                attributedto: row.get(4)?,
                content: row.get(5)?,
                sensitive: row.get(6)?,
                published: row.get(7)?,
                received: row.get(8)?,
                conversation: row.get(9)?,
            })
        }).unwrap();

    //Parse the results
    for row in rows {
        let reply: ReplyRecord = row.unwrap();
        replies.push(reply);
    }

    return Ok(replies.clone());


    //Err(Box::new(HydraError(format!("Failed to get followers for: [{}].", pcid).into())))
}