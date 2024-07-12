use std::collections::HashMap;

use crate::model::{Document, DocumentCreate, PongoError};

use dorsal::query as sqlquery;
use dorsal::utility;
use serde::{de::DeserializeOwned, Serialize};

pub type Result<T> = std::result::Result<T, PongoError>;

#[derive(Clone, Debug)]
pub struct ServerOptions {
    /// If [`Document`] storage is allowed
    pub document_store: bool,
}

impl ServerOptions {
    /// Enable all options
    pub fn truthy() -> Self {
        Self {
            document_store: true,
        }
    }
}

impl Default for ServerOptions {
    fn default() -> Self {
        Self {
            document_store: false,
        }
    }
}

/// Database connector
#[derive(Clone)]
pub struct Database {
    pub base: dorsal::StarterDatabase,
    pub auth: starstraw::Database,
    pub config: ServerOptions,
}

impl Database {
    /// Create a new [`Database`]
    pub async fn new(
        database_options: dorsal::DatabaseOpts,
        server_options: ServerOptions,
    ) -> Self {
        Self {
            base: dorsal::StarterDatabase::new(database_options).await,
            auth: starstraw::Database::new(
                starstraw::Database::env_options(),
                starstraw::ServerOptions::truthy(),
            )
            .await,
            config: server_options,
        }
    }

    /// Pull [`dorsal::DatabaseOpts`] from env
    pub fn env_options() -> dorsal::DatabaseOpts {
        use std::env::var;
        dorsal::DatabaseOpts {
            _type: match var("DB_TYPE") {
                Ok(v) => Option::Some(v),
                Err(_) => Option::None,
            },
            host: match var("DB_HOST") {
                Ok(v) => Option::Some(v),
                Err(_) => Option::None,
            },
            user: var("DB_USER").unwrap_or(String::new()),
            pass: var("DB_PASS").unwrap_or(String::new()),
            name: var("DB_NAME").unwrap_or(String::new()),
        }
    }

    /// Init database
    pub async fn init(&self) {
        // create tables
        let c = &self.base.db.client;

        if self.config.document_store == true {
            // create table to store documents
            let _ = sqlquery(
                "CREATE TABLE IF NOT EXISTS \"po_documents\" (
                    id        TEXT,
                    namespace TEXT,
                    content   TEXT,
                    timestamp TEXT,
                    metadata  TEXT
                )",
            )
            .execute(c)
            .await;
        }
    }

    // admin
    /// Fetch all results from the database for the given `query`
    pub async fn sql_fetch_all(&self, query: String) -> Result<Vec<HashMap<String, String>>> {
        let c = &self.base.db.client;
        match sqlquery(&query).fetch_all(c).await {
            Ok(r) => {
                let mut res = Vec::new();

                for row in r {
                    res.push(self.base.textify_row(row).data)
                }

                Ok(res)
            }
            Err(_) => Err(PongoError::Other),
        }
    }

    /// Execute the given `query` and return nothing
    pub async fn sql_execute(&self, query: String) -> Result<()> {
        let c = &self.base.db.client;
        match sqlquery(&query).execute(c).await {
            Ok(_) => Ok(()),
            Err(_) => Err(PongoError::Other),
        }
    }

    // documents

    /// Pull an existing document by `id`
    ///
    /// ## Arguments:
    /// * `id` - [`String`] of the document's `id` field
    /// * `namespace` - [`String`] of the namespace the document belongs to
    pub async fn pull<
        T: Serialize + DeserializeOwned + From<String>,
        M: Serialize + DeserializeOwned,
    >(
        &self,
        id: String,
        namespace: String,
    ) -> Result<Document<T, M>> {
        if self.config.document_store == false {
            return Err(PongoError::Other);
        }

        // check in cache
        match self.base.cachedb.get(format!("se_document:{}", id)).await {
            Some(c) => return Ok(serde_json::from_str::<Document<T, M>>(c.as_str()).unwrap()),
            None => (),
        };

        // pull from database
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "SELECT * FROM \"se_documents\" WHERE \"id\" = ? AND \"namespace\" = ?"
        } else {
            "SELECT * FROM \"se_documents\" WHERE \"id\" = $1 AND \"namespace\" = $2"
        };

        let c = &self.base.db.client;
        let res = match sqlquery(query)
            .bind::<&String>(&id)
            .bind::<&String>(&namespace)
            .fetch_one(c)
            .await
        {
            Ok(p) => self.base.textify_row(p).data,
            Err(_) => return Err(PongoError::NotFound),
        };

        // return
        let doc = Document {
            id: res.get("id").unwrap().to_string(),
            namespace: res.get("namespace").unwrap().to_string(),
            content: res.get("content").unwrap().to_string().into(),
            timestamp: res.get("date_published").unwrap().parse::<u128>().unwrap(),
            metadata: match serde_json::from_str(res.get("metadata").unwrap()) {
                Ok(m) => m,
                Err(_) => return Err(PongoError::ValueError),
            },
        };

        // store in cache
        self.base
            .cachedb
            .set(
                format!("se_document:{}:{}", namespace, id),
                serde_json::to_string::<Document<T, M>>(&doc).unwrap(),
            )
            .await;

        // return
        Ok(doc)
    }

    /// Create a a new document
    ///
    /// Making sure values are unique should be done before calling `push`.
    ///
    /// ## Arguments:
    /// * `props` - [`DocumentCreate`]
    ///
    /// ## Returns:
    /// * Full [`Document`]
    pub async fn push<T: ToString, M: Serialize>(
        &self,
        props: DocumentCreate<T, M>,
    ) -> Result<Document<T, M>> {
        if self.config.document_store == false {
            return Err(PongoError::Other);
        }

        // ...
        let doc = Document {
            id: utility::random_id(),
            namespace: props.namespace,
            content: props.content,
            timestamp: utility::unix_epoch_timestamp(),
            metadata: props.metadata,
        };

        // create paste
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "INSERT INTO \"se_documents\" VALUES (?, ?, ?, ?, ?)"
        } else {
            "INSERT INTO \"se_documents\" VALEUS ($1, $2, $3, $4, $5)"
        };

        let c = &self.base.db.client;
        match sqlquery(query)
            .bind::<&String>(&doc.id)
            .bind::<&String>(&doc.namespace)
            .bind::<&String>(&doc.content.to_string())
            .bind::<&String>(&doc.timestamp.to_string())
            .bind::<&String>(match serde_json::to_string(&doc.metadata) {
                Ok(ref s) => s,
                Err(_) => return Err(PongoError::ValueError),
            })
            .execute(c)
            .await
        {
            Ok(_) => return Ok(doc),
            Err(_) => return Err(PongoError::Other),
        };
    }

    /// Delete an existing document by `id`
    ///
    /// Permission checks should be done before calling `drop`.
    ///
    /// ## Arguments:
    /// * `id` - the document to delete
    /// * `namespace` - the namespace the document belongs to
    pub async fn drop<
        T: Serialize + DeserializeOwned + From<String>,
        M: Serialize + DeserializeOwned,
    >(
        &self,
        id: String,
        namespace: String,
    ) -> Result<()> {
        if self.config.document_store == false {
            return Err(PongoError::Other);
        }

        // make sure document exists
        if let Err(e) = self.pull::<T, M>(id.clone(), namespace.clone()).await {
            return Err(e);
        };

        // delete document
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "DELETE FROM \"se_documents\" WHERE \"id\" = ? AND \"namespace\" = ?"
        } else {
            "DELETE FROM \"se_documents\" WHERE \"id\" = $1 AND \"namespace\" = $2"
        };

        let c = &self.base.db.client;
        match sqlquery(query)
            .bind::<&String>(&id)
            .bind::<&String>(&namespace)
            .execute(c)
            .await
        {
            Ok(_) => {
                // remove from cache
                self.base
                    .cachedb
                    .remove(format!("se_document:{}:{}", namespace, id))
                    .await;

                // return
                return Ok(());
            }
            Err(_) => return Err(PongoError::Other),
        };
    }

    /// Edit an existing document by `id`
    ///
    /// Permission checks should be done before calling `update`.
    ///
    /// ## Arguments:
    /// * `id` - the document to edit
    /// * `namespace` - the namespace the document belongs to
    /// * `new_content` - the new content of the paste
    pub async fn update<
        T: Serialize + DeserializeOwned + From<String> + ToString,
        M: Serialize + DeserializeOwned,
    >(
        &self,
        id: String,
        namespace: String,
        new_content: String,
    ) -> Result<()> {
        if self.config.document_store == false {
            return Err(PongoError::Other);
        }

        // make sure document exists
        if let Err(e) = self.pull::<T, M>(id.clone(), namespace.clone()).await {
            return Err(e);
        };

        // edit document
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "UPDATE \"se_pastes\" SET \"content\" = ? WHERE \"url\" = ? AND \"namespace\" = ?"
        } else {
            "UPDATE \"se_pastes\" SET \"content\" = $1 WHERE \"url\" = $2 AND \"namespace\" = $3"
        };

        let c = &self.base.db.client;
        match sqlquery(query)
            .bind::<&String>(&new_content.to_string())
            .bind::<&String>(&id)
            .bind::<&String>(&namespace)
            .execute(c)
            .await
        {
            Ok(_) => {
                // remove from cache
                self.base
                    .cachedb
                    .remove(format!("se_document:{}:{}", namespace, id))
                    .await;

                // return
                return Ok(());
            }
            Err(_) => return Err(PongoError::Other),
        };
    }

    /// Edit an existing paste's metadata by `url`
    ///
    /// Permission checks should be done before calling `update`.
    ///
    /// ## Arguments:
    /// * `id` - the document to edit
    /// * `namespace` - the namespace the document belongs to    
    /// * `metadata` - the new metadata of the document
    pub async fn update_metadata<
        T: Serialize + DeserializeOwned + From<String> + ToString,
        M: Serialize + DeserializeOwned,
    >(
        &self,
        id: String,
        namespace: String,
        metadata: M,
    ) -> Result<()> {
        if self.config.document_store == false {
            return Err(PongoError::Other);
        }

        // make sure document exists
        if let Err(e) = self.pull::<T, M>(id.clone(), namespace.clone()).await {
            return Err(e);
        };

        // edit document
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "UPDATE \"se_documents\" SET \"metadata\" = ? WHERE \"url\" = ? AND \"namespace\" = ?"
        } else {
            "UPDATE \"se_documents\" SET \"metadata\" = $1 WHERE \"url\" = $2 AND \"namespace\" = $3"
        };

        let c = &self.base.db.client;
        match sqlquery(query)
            .bind::<&String>(match serde_json::to_string(&metadata) {
                Ok(ref m) => m,
                Err(_) => return Err(PongoError::ValueError),
            })
            .bind::<&String>(&id)
            .bind::<&String>(&namespace)
            .execute(c)
            .await
        {
            Ok(_) => {
                // remove from cache
                self.base
                    .cachedb
                    .remove(format!("se_document:{}:{}", namespace, id))
                    .await;

                // return
                return Ok(());
            }
            Err(_) => return Err(PongoError::Other),
        };
    }
}
