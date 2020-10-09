use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use trust_dns_proto::rr;

// The ResultsCache is a struct that the resulting records will be written to before being serialized
// into a json file. They key is the `IpAddr` for A or AAAA records, and Name if record type is CNAME.
#[derive(Debug)]
pub(crate) struct ResultsCache {
    pub inner: Mutex<HashMap<String, ResolveResponse>>,
}

impl ResultsCache {
    //TODO: Should be `Default` trait not method
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(HashMap::new()),
        })
    }

    // Returns the number of results cached
    pub(crate) async fn num_results(&self) -> usize {
        let lock = self.inner.lock().await;
        lock.keys().len()
    }

    // Drains records from the queue and inerts them into the ResultsCache. This method will be
    // called every time the queue reaches capacity, thereby avoding taking the lock too often.
    pub(crate) async fn insert(&self, records: &mut VecDeque<ResolveResponse>) {
        // Acquire the lock
        let mut map = self.inner.lock().await;
        // Drain the queue of all records
        map.extend(records.drain(..).map(|r| (r.key(), r)));
        drop(map);
    }

    // Serializes the contents of the `ResultsCache` into a json string.
    pub(crate) async fn json(&self) -> String {
        let lock = self.inner.lock().await;
        let vals: Vec<&ResolveResponse> = lock.values().collect();
        serde_json::to_string_pretty(&vals).unwrap()
    }
}

// Represents the different kind of reponses we will get when making a DNS query.
#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub(crate) enum ResolveResponse {
    Record {
        name: String,
        #[serde(rename(serialize = "type"))]
        kind: String,
        ttl: u32,
        is_wildcard: bool,
    },
    IpRecord {
        name: String,
        #[serde(rename(serialize = "ip"))]
        value: Option<IpAddr>,
        #[serde(rename(serialize = "type"))]
        kind: String,
        ttl: u32,
        is_wildcard: bool,
    },
    Error {
        query: String,
        response_code: String,
    },
}

impl ResolveResponse {
    // Returns the fields that we use for keys inside the ResultsCache. This is a clone for now, but
    // in the future we could return an `Arc<String>` to avoid the clone.
    pub(crate) fn key(&self) -> String {
        match self {
            ResolveResponse::IpRecord { value, .. } => value.unwrap().to_string(),
            ResolveResponse::Record { name, .. } => name.clone(),
            ResolveResponse::Error { query, .. } => query.clone(),
        }
    }
}

// Handles conversion from a `resource::Record` to a `ResolveResponse`. Since we only care about
impl From<&rr::resource::Record> for ResolveResponse {
    fn from(record: &rr::resource::Record) -> Self {
        use rr::record_type::RecordType;
        let name = record.name().to_utf8();
        let kind = record.record_type();
        let ttl = record.ttl();
        let is_wildcard = record.name().is_wildcard();

        match kind {
            RecordType::A | RecordType::AAAA => Self::IpRecord {
                name,
                value: record.rdata().to_ip_addr(),
                kind: kind.to_string(),
                ttl,
                is_wildcard,
            },
            RecordType::CNAME => Self::Record {
                name: record.rdata().as_cname().unwrap().to_utf8(),
                kind: kind.to_string(),
                ttl,
                is_wildcard,
            },
            _ => Self::Record {
                name,
                kind: kind.to_string(),
                ttl,
                is_wildcard,
            },
        }
    }
}
