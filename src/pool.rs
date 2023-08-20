//! Implementation of pool of [`YdbConnection`].
//! Uses method `list_endpoints` of `DiscoveryServiceClient` to create pool on multiple endpoints
//! # Examples
//! ```rust
//! # #[tokio::main]
//! # async fn main() {
//! let db_name = std::env::var("DB_NAME").expect("DB_NAME not set");
//! let creds = std::env::var("DB_TOKEN").expect("DB_TOKEN not set");
//! let ep = ydb_unofficial::client::YdbEndpoint {ssl: true, host: "ydb.serverless.yandexcloud.net".to_owned(), port: 2135, load_factor: 0.0};
//! let pool = ydb_unofficial::pool::YdbPoolBuilder::new(creds, db_name.try_into().unwrap(), ep).build().unwrap();
//! let mut conn = pool.get().await.unwrap();
//! let mut table_client = conn.table();
//! //do something...
//! let mut conn2 = pool.get().await.unwrap();
//! //do another staff
//! pool.close();
//! # }
//! ```
use super::*;
use std::{vec, time::Duration};

use deadpool::managed::{Manager, Pool, PoolBuilder, PoolConfig, Hook};

use tonic::transport::{Endpoint, Uri};
use tower::ServiceExt;

use payload::YdbResponseWithResult;
use generated::ydb::discovery::{EndpointInfo, ListEndpointsRequest};
use auth::Credentials;
use crate::client::YdbEndpoint;


type YdbEndpoints = std::sync::RwLock<Vec<YdbEndpoint>>;
pub type YdbPool<C> = Pool<ConnectionManager<C>>;

impl From<EndpointInfo> for YdbEndpoint {
    fn from(value: EndpointInfo) -> Self {
        Self {
            ssl: value.ssl,
            host: value.address,
            port: value.port as u16,
            load_factor: value.load_factor,
        }
    }
}


fn make_endpoint(info: &YdbEndpoint) -> Endpoint {
    let uri: tonic::transport::Uri = format!("{}://{}:{}", info.scheme(), info.host, info.port).try_into().unwrap();
    let mut e = Endpoint::from(uri).tcp_keepalive(Some(std::time::Duration::from_secs(15)));
    if info.ssl {
        e = e.tls_config(Default::default()).unwrap()
    }
    e
}

pub trait GetScheme {
    fn get_scheme(&self) -> &'static str;
}

impl GetScheme for EndpointInfo {
    fn get_scheme(&self) -> &'static str {
        if self.ssl { "grpcs" } else { "grpc" }
    }
}

pub struct ConnectionManager<C> {
    creds: C,
    db_name: AsciiValue,
    endpoints: YdbEndpoints,
}

impl<C: Credentials> ConnectionManager<C> {
    pub fn next_endpoint(&self) -> Endpoint {
        let endpoints = self.endpoints.read().unwrap();
        if endpoints.len() == 1 {
            return endpoints.first().unwrap().make_endpoint();
        } else if endpoints.is_empty() {
            panic!("List of endpoints is empty");
        }
        let mut rng = rand::thread_rng();
        use rand::Rng;
        let e1 = rng.gen::<usize>() % endpoints.len();
        let mut e2 = e1;
        while e2 == e1 { //TODO: кажется, это что-то неоптимальное
            e2 = rng.gen::<usize>() % endpoints.len();
        }
        let e1 = &endpoints[e1];
        let e2 = &endpoints[e2];
        let endpoint = if e1.load_factor < e2.load_factor {e1} else { e2 };
        make_endpoint(&endpoint)
    }
}

#[async_trait::async_trait]
impl <C: Credentials + Sync> Manager for ConnectionManager<C> {
    type Type = YdbConnection<C>;

    type Error = tonic::transport::Error;

    async fn create(&self) ->  Result<Self::Type, Self::Error> {
        let endpoint = self.next_endpoint();
        let channel = endpoint.connect().await?;
        let db_name = self.db_name.clone();
        let creds = self.creds.clone();
        Ok(YdbConnection::new(channel, db_name, creds))
    }

    async fn recycle(&self, obj: &mut Self::Type) ->  deadpool::managed::RecycleResult<Self::Error> {
        obj.ready().await?;
        Ok(())
    }
}

/// Builder for pool of [`YdbConnection`]
pub struct YdbPoolBuilder<C: Credentials + Send + Sync> {
    inner: PoolBuilder<ConnectionManager<C>>,
    update_interval: Duration,
}

macro_rules! delegate {
    ($( $fun:ident($param:ty), )+) => { $(
        pub fn $fun(mut self, v: $param) -> Self {
            self.inner = self.inner.$fun(v);
            self
        }
    )+ };
}
/// Wrapper on [`PoolBuilder`] for YdbConnection.
impl<C: Credentials + Send + Sync> YdbPoolBuilder<C> {
    pub fn new(creds: C, db_name: AsciiValue, endpoint: YdbEndpoint) -> Self {
        let endpoints =  std::sync::RwLock::new(vec![endpoint]);
        let inner = Pool::builder(ConnectionManager {creds, db_name, endpoints});
        let update_interval = Duration::from_secs(77);
        Self {inner, update_interval}
    }
    /// Set period to update endpoints for pool. Default is 77 seconds.
    pub fn update_interval(mut self, interval: Duration) -> Self {
        self.update_interval = interval;
        self
    }
    delegate!{ 
        config(PoolConfig),
        create_timeout(Option<Duration>),
        max_size(usize),
        post_create(impl Into<Hook<ConnectionManager<C>>>),
        post_recycle(impl Into<Hook<ConnectionManager<C>>>),
        pre_recycle(impl Into<Hook<ConnectionManager<C>>>),
        recycle_timeout(Option<Duration>),
        runtime(deadpool::Runtime),
        timeouts(deadpool::managed::Timeouts),
        wait_timeout(Option<Duration>),
    }
    pub fn build(self) -> Result<Pool<ConnectionManager<C>>, deadpool::managed::BuildError<tonic::transport::Error>> {
        let pool = self.inner.build()?;
        let result = pool.clone();
        let db_name = pool.manager().db_name.to_str().unwrap().to_owned();
        tokio::spawn(async move {
            loop {
                if pool.is_closed() {
                    log::debug!("Connection pool closed");
                    break;
                }
                if let Err(e) = update_endpoints(&pool, db_name.clone()).await {
                    log::error!("Error on update endpoints for pool: {e:?}");
                }
                tokio::time::sleep(self.update_interval).await;
            }
        });
        Ok(result)
    }
}

async fn update_endpoints<C: Credentials + Send + Sync>(pool: &Pool<ConnectionManager<C>>, database: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut service = pool.get().await?;
    let mut discovery = service.discovery();
    let response = discovery.list_endpoints(ListEndpointsRequest{database, ..Default::default()}).await?; 
    let endpoints: Vec<_> = response.into_inner().result()?.endpoints.into_iter().map(From::from).collect();
    log::debug!("Pool endpoints updated ({} endpoints)", endpoints.len());
    *pool.manager().endpoints.write().unwrap() = endpoints;
    Ok(())
}
pub fn to_endpoint_info(value: Uri) -> Result<EndpointInfo, String> {
    let mut e = EndpointInfo::default();
    e.ssl = match value.scheme_str() {
        Some("grpc") => false,
        Some("grpcs") => true,
        _ => return Err("Unknown protocol".to_owned()),
    };
    e.address = value.host().ok_or("no host")?.to_owned();
    e.port = value.port_u16().ok_or("no port")? as u32;
    Ok(e)
}