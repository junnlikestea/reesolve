use crate::OutputFormat;
use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use trust_dns_proto::error::ProtoErrorKind;
use trust_dns_proto::rr;
use trust_dns_resolver::error::{ResolveError, ResolveErrorKind};

/// The ResultsCache is a struct that the resulting records will be written to before being serialized
/// into a json or csv file. They key is the `IpAddr` for A or AAAA records, and Name if record type is CNAME.
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

    /// Returns the number of results cached
    pub(crate) async fn num_results(&self) -> usize {
        let lock = self.inner.lock().await;
        lock.keys().len()
    }

    /// Drains records from the queue and inerts them into the `ResultsCache`. This method will be
    /// called every time the queue reaches capacity, thereby avoding taking the lock too often and
    /// reducing contention.
    pub(crate) async fn insert(&self, records: &mut VecDeque<ResolveResponse>) {
        // Acquire the lock
        let mut map = self.inner.lock().await;
        // Drain the queue of all records
        map.extend(records.drain(..).map(|r| (r.key(), r)));
    }

    pub async fn records(&self) -> HashMap<String, ResolveResponse> {
        let map = self.inner.lock().await;
        map.clone()
    }

    pub(crate) async fn set_wildcard(&self, key: &str) {
        let mut lock = self.inner.lock().await;

        if let Some(record) = lock.get_mut(key) {
            if let ResolveResponse::IpRecord { is_wildcard, .. }
            | ResolveResponse::Record { is_wildcard, .. } = record
            {
                *is_wildcard = true;
            }
        }
    }

    pub(crate) async fn results(&self, format: &OutputFormat) -> Result<Vec<u8>> {
        match format {
            OutputFormat::Csv => self.csv().await,
            OutputFormat::Json => self.json().await,
        }
    }

    /// Serializes the contents of the `ResultsCache` into json.
    async fn json(&self) -> Result<Vec<u8>> {
        let lock = self.inner.lock().await;
        let vals: Vec<&ResolveResponse> = lock.values().collect();
        Ok(serde_json::to_vec_pretty(&vals).unwrap())
    }

    /// Serializes the contents of the `ResultsCache` into a csv.
    async fn csv(&self) -> Result<Vec<u8>> {
        let mut wtr = csv::Writer::from_writer(vec![]);
        let lock = self.inner.lock().await;
        lock.values().map(|v| wtr.serialize(v)).for_each(drop);
        Ok(wtr.into_inner()?)
    }
}

// Represents the different kind of reponses we will get when making a DNS query.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub(crate) enum ResolveResponse {
    Record {
        query: String,
        name: String,
        #[serde(rename(serialize = "type"))]
        kind: String,
        ttl: u32,
        is_wildcard: bool,
    },
    IpRecord {
        query: String,
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
    /// A wrapper around the `From` trait, but adds the query if the record is a CNAME.
    pub(crate) fn new(record: &rr::resource::Record, q: Arc<String>) -> ResolveResponse {
        let mut record = ResolveResponse::from(record);

        match &mut record {
            ResolveResponse::Record { query, .. } | ResolveResponse::IpRecord { query, .. } => {
                *query = q.to_string();
                record
            }
            _ => record,
        }
    }

    /// Returns the fields that we use for keys inside the ResultsCache. This is a clone for now, but
    /// in the future we could return an `Arc<String>` to avoid the clone.
    pub(crate) fn key(&self) -> String {
        match self {
            ResolveResponse::IpRecord { value, .. } => value.unwrap().to_string(),
            ResolveResponse::Record { name, .. } => name.clone(),
            ResolveResponse::Error { query, .. } => query.clone(),
        }
    }

    /// Extracts the errors we want from the `ResolveError`
    pub(crate) fn from_error(error: ResolveError) -> Option<ResolveResponse> {
        //TODO: How can we get the query that triggered the error, if it doesn't actually contain
        // the field?

        match error.kind() {
            // Message & Msg cannot be in the same match arm, because of the different inner types
            // String/&str
            ResolveErrorKind::Message(m) => Some(ResolveResponse::Error {
                query: String::default(),
                response_code: m.to_string(),
            }),
            ResolveErrorKind::Msg(m) => Some(ResolveResponse::Error {
                query: String::default(),
                response_code: m.to_string(),
            }),
            ResolveErrorKind::NoRecordsFound {
                query,
                response_code,
                ..
            } => Some(ResolveResponse::Error {
                query: query.name().to_string(),
                response_code: response_code.to_string(),
            }),

            // SERVFAIL are returned as a `ProtoErrorKind::Msg` or `ProtoErrorKind::Message` ?
            ResolveErrorKind::Proto(e) => match e.kind() {
                ProtoErrorKind::Msg(s) => Some(ResolveResponse::Error {
                    query: String::default(),
                    response_code: s.to_string(),
                }),
                ProtoErrorKind::Message(s) => Some(ResolveResponse::Error {
                    query: String::default(),
                    response_code: s.to_string(),
                }),
                _ => None,
            },
            ResolveErrorKind::Io(..) => None,
            ResolveErrorKind::Timeout => None,
        }
    }
}

// Handles conversion from a `resource::Record` to a `ResolveResponse`. Since we only care about a
// few of the record types this is not exhaustive.
impl From<&rr::resource::Record> for ResolveResponse {
    fn from(record: &rr::resource::Record) -> Self {
        use rr::record_type::RecordType;
        let name = record.name().to_utf8();
        let kind = record.record_type();
        let ttl = record.ttl();
        let is_wildcard = false;

        match kind {
            RecordType::A | RecordType::AAAA => Self::IpRecord {
                query: String::default(),
                name,
                value: record.rdata().to_ip_addr(),
                kind: kind.to_string(),
                ttl,
                is_wildcard,
            },
            RecordType::CNAME => Self::Record {
                query: String::default(),
                name: record.rdata().as_cname().unwrap().to_utf8(),
                kind: kind.to_string(),
                ttl,
                is_wildcard,
            },
            _ => Self::Record {
                query: String::default(),
                name,
                kind: kind.to_string(),
                ttl,
                is_wildcard,
            },
        }
    }
}
