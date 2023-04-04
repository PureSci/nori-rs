use mongodb::{
    bson::{doc, spec::ElementType, Bson, Document},
    options::FindOneOptions,
    Database,
};

pub fn get_value_from_all(doc: &Document, key: &str, val: &str) -> String {
    let a = doc
        .get(key)
        .unwrap()
        .as_document()
        .unwrap()
        .get(val)
        .unwrap();
    get_value_sub(a)
}
#[allow(dead_code)]
pub fn get_value(doc: &Document, val: &str) -> String {
    let a = doc.get(val).unwrap();
    get_value_sub(a)
}

fn get_value_sub(a: &Bson) -> String {
    match a.element_type() {
        ElementType::String => a.as_str().unwrap().to_string(),
        ElementType::Boolean => a.as_bool().unwrap().to_string(),
        ElementType::Int32 => a.as_i32().unwrap().to_string(),
        ElementType::Int64 => a.as_i64().unwrap().to_string(),
        _ => "".to_string(),
    }
}

/// Gets the data of config thru MongoDb
/// ### Examples:
/// ```rust
/// // Get one
/// let showgen = get_config_data(db, "analysis", "user_id", "guild_id", "showgen").await;
/// assert_eq!(showgen, doc! {
///     "server_default": bool,
///     "value": Value
/// });
/// // Get all
/// let analysis = get_config_data(db, "analysis", "user_id", "guild_id", "_all").await;
/// assert_eq!(analysis, doc! {
///     "showgen": {
///         "server_default": bool,
///         "value": Value
///     },
///     ...
/// });
/// ```
pub async fn get_config_data(
    db: &Database,
    typ: &str,
    user_id: Option<String>,
    guild_id: String,
    data: &str,
) -> Document {
    if user_id.is_none() {
        return get_guild_defaults(db, typ, guild_id, data).await;
    }
    let raw_result = db
        .collection::<Document>(format!("user_config_{typ}").as_str())
        .find_one(
            doc! {
                "_id": &user_id
            },
            if data == "_all" {
                None
            } else {
                Some(FindOneOptions::builder().projection(doc! {data:1}).build())
            },
        )
        .await
        .unwrap();
    let end_result: Document;
    if raw_result != None && data != "_all" && raw_result.clone().unwrap().get(data) == None {
        end_result = get_guild_defaults(db, typ, guild_id, data).await;
    } else if raw_result != None && data != "_all" {
        end_result = doc! {
            "server_default": false,
            "value": raw_result.unwrap().get(data).unwrap()
        }
    } else if raw_result != None && data == "_all" {
        let guild_result = get_guild_defaults(db, typ, guild_id, data).await;
        let mut arr = doc! {};
        let unwrapped = raw_result.unwrap();
        for key in guild_result.keys() {
            arr.insert(
                key,
                match unwrapped.get(key) {
                    None => guild_result
                        .get(key)
                        .unwrap()
                        .as_document()
                        .unwrap()
                        .to_owned(),
                    Some(d) => doc! {
                        "server_default": false,
                        "value": d
                    },
                },
            );
        }
        end_result = arr;
    } else {
        end_result = get_guild_defaults(db, typ, guild_id, data).await;
    }
    return end_result;
}

async fn get_guild_defaults(db: &Database, typ: &str, guild_id: String, data: &str) -> Document {
    let defaults = doc! {
        "reminders": {
            "drop":true,
            "grab":false, // false
            "raid":true
        },
        "analysis": {
            "enabled": true,
            "expire-delete": false,
            "show-gen":true,
            "ping": true,
            "show-time":true,
            "type":1
        },
        "utils": {
            "delete-message": false,
        }
    };
    let guild_result = db
        .collection::<Document>(format!("server_config_{typ}").as_str())
        .find_one(
            doc! {
                "_id": guild_id
            },
            if data == "_all" {
                None
            } else {
                Some(FindOneOptions::builder().projection(doc! {data:1}).build())
            },
        )
        .await
        .unwrap();
    let end_guild_result: Document;
    if data == "_all" && guild_result == None {
        let default = defaults.get(typ).unwrap().as_document().unwrap();
        let mut arr = doc! {};
        for key in default.keys() {
            arr.insert(
                key,
                doc! {
                    "server_default": true,
                    "value": default.get(key).unwrap().to_owned()
                },
            );
        }
        end_guild_result = arr;
        //end_guild_result = defaults.get(typ).unwrap().as_document().unwrap().to_owned();
    } else if data == "_all" && guild_result != None {
        let default = defaults.get(typ).unwrap().as_document().unwrap();
        let mut arr = doc! {};
        for key in default.keys() {
            arr.insert(
                key,
                match guild_result.as_ref().unwrap().get(key) {
                    None => doc! {
                        "server_default": true,
                        "value":default.get(key).unwrap().to_owned()
                    },
                    Some(d) => doc! {
                        "server_default":true,
                        "value": d.to_owned()
                    },
                },
            );
        }
        end_guild_result = arr;
    } else if data != "_all" && guild_result == None {
        end_guild_result = doc! {
            "server_default": true,
            "value": defaults
            .get(typ)
            .unwrap()
            .as_document()
            .unwrap()
            .get(data)
            .unwrap()
        };
    } else if data != "_all" && guild_result.as_ref().unwrap().get(data) == None {
        end_guild_result = doc! {
            "server_default": true,
            "value": defaults.get(typ).unwrap().as_document().unwrap().get(data)
        }
    } else {
        end_guild_result = doc! {
            "server_default": true,
            "value": guild_result.as_ref().unwrap().get(data).unwrap()
        }
    }
    return end_guild_result;
}
