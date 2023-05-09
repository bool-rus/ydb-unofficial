
use std::{vec, time::Duration};

use deadpool::managed::{Manager, Pool, PoolBuilder, PoolConfig, Hook};

use tonic::transport::{Endpoint, Uri};
use tower::ServiceExt;

use crate::payload::YdbResponseWithResult;
use crate::generated::ydb::discovery::{EndpointInfo, ListEndpointsRequest};
use crate::client::{Credentials, YdbService, AsciiValue};
use crate::YdbError;


type YdbEndpoints = std::sync::RwLock<Vec<EndpointInfo>>;


fn make_endpoint(info: &EndpointInfo) -> Endpoint {
    let uri: tonic::transport::Uri = format!("{}://{}:{}", info.scheme(), info.address, info.port).try_into().unwrap();
    let mut e = Endpoint::from(uri).tcp_keepalive(Some(std::time::Duration::from_secs(15)));
    if info.ssl {
        e = e.tls_config(Default::default()).unwrap()
    }
    e
}

impl EndpointInfo {
    pub fn scheme(&self) -> &'static str {
        if self.ssl { "grpcs" } else { "grpc" }
    }
}


/* 
pub fn create_pool<C: Credentials>(creds: C) -> tower::balance::pool::Pool<YdbPool, C, tonic::codegen::http::Request<tonic::body::BoxBody>> {
    let mut pool = tower::balance::pool::Pool::new(YdbPool{endpoints: vec![]}, creds.clone());
    let mut client = DiscoveryServiceClient::new(&mut pool);
    pool
}


*/

pub struct ConnectionManager<C> {
    creds: C,
    db_name: AsciiValue,
    endpoints: YdbEndpoints,
}

impl<C: Credentials> ConnectionManager<C> {
    pub fn next_endpoint(&self) -> Endpoint {
        let mut rng = rand::thread_rng();
        let rnd: usize = rng.gen();
        use rand::Rng;
        let endpoints = self.endpoints.read().unwrap().clone();

        let size = endpoints.iter().fold(0usize, |_s, e|(e.load_factor.abs() * 10.0) as usize + 1);
        let e = endpoints.iter().map(|e|{
            let count = (e.load_factor.abs() * 10.0) as usize + 1;
            [e].into_iter().cycle().take(count)
        }).flatten().cycle().nth(rnd % size).unwrap();
        make_endpoint(e)
    }
}

#[async_trait::async_trait]
impl <C: Credentials + Sync> Manager for ConnectionManager<C> {
    type Type = YdbService<C>;

    type Error = tonic::transport::Error;

    async fn create(&self) ->  Result<Self::Type, Self::Error> {
        let endpoint = self.next_endpoint();
        let channel = endpoint.connect().await?;
        let db_name = self.db_name.clone();
        let creds = self.creds.clone();
        Ok(YdbService::new(channel, db_name, creds))
    }

    async fn recycle(&self, obj: &mut Self::Type) ->  deadpool::managed::RecycleResult<Self::Error> {
        //TODO: еще здесь нужно проверить, что сессия не протухла
        obj.ready().await?;
        Ok(())
    }
}

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

impl<C: Credentials + Send + Sync> YdbPoolBuilder<C> {
    pub fn new(creds: C, db_name: AsciiValue, endpoint: EndpointInfo) -> Self {
        let endpoints =  std::sync::RwLock::new(vec![endpoint]);
        let inner = Pool::builder(ConnectionManager {creds, db_name, endpoints});
        let update_interval = Duration::from_secs(1);
        Self {inner, update_interval}
    }
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
    let endpoints = response.into_inner().result()?.endpoints;
    log::debug!("Pool endpoints updated ({} endpoints)", endpoints.len());
    *pool.manager().endpoints.write().unwrap() = endpoints;
    Ok(())
}

impl TryFrom<Uri> for EndpointInfo {
    type Error = String;

    fn try_from(value: Uri) -> Result<Self, Self::Error> {
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
}