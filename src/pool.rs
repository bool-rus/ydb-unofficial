
use std::{sync::{atomic::AtomicU32, Arc}, collections::hash_map::RandomState, vec, convert::Infallible, rc::Rc, task::Poll, ops::Deref, time::Duration};

use deadpool::managed::{Manager, Pool, PoolBuilder};
use std::sync::Mutex;
use tonic::{transport::{Channel, Endpoint, ClientTlsConfig}, codegen::BoxFuture};
use tower::{Service, load::{PendingRequests, CompleteOnResponse}, ServiceExt};

use crate::{generated::{ydb::{discovery::{ListEndpointsResult, EndpointInfo, WhoAmIRequest, self, ListEndpointsRequest}, table::v1::table_service_client::TableServiceClient, scheme::ListDirectoryRequest}, DiscoveryServiceClient}, client::{Credentials, YdbService, AsciiValue, YdbError}, exper::YdbResponse};

struct YdbEndpoint {
    inner: Endpoint,
    load_factor: f32,
    connections: AtomicU32,
}

type YdbEndpoints = std::sync::RwLock<Vec<EndpointInfo>>;

impl Into<Endpoint> for YdbEndpoint {
    fn into(self) -> Endpoint {
        self.inner
    }
}

impl From<&EndpointInfo> for YdbEndpoint {
    fn from(info: &EndpointInfo) -> Self {
        let uri: tonic::transport::Uri = format!("{}:{}", info.address, info.port).try_into().unwrap();
        let mut inner = Endpoint::from(uri).tcp_keepalive(Some(std::time::Duration::from_secs(15)));
        if info.ssl {
            inner = inner.tls_config(Default::default()).unwrap()
        }
        Self {
            inner,
            load_factor: info.load_factor,
            connections: Default::default(),
        }
    }
}

fn make_endpoint(info: &EndpointInfo) -> Endpoint {
    let uri: tonic::transport::Uri = format!("{}:{}", info.address, info.port).try_into().unwrap();
    let mut e = Endpoint::from(uri).tcp_keepalive(Some(std::time::Duration::from_secs(15)));
    if info.ssl {
        e = e.tls_config(Default::default()).unwrap()
    }
    e
}


/* 
pub fn create_pool<C: Credentials>(creds: C) -> tower::balance::pool::Pool<YdbPool, C, tonic::codegen::http::Request<tonic::body::BoxBody>> {
    let mut pool = tower::balance::pool::Pool::new(YdbPool{endpoints: vec![]}, creds.clone());
    let mut client = DiscoveryServiceClient::new(&mut pool);
    pool
}


*/

struct ConnectionManager<C> {
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

        let size = endpoints.iter().fold(0usize, |s, e|(e.load_factor.abs() * 10.0) as usize + 1);
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

fn make_pool() -> deadpool::managed::Pool<ConnectionManager<String>> { 
    let man = ConnectionManager {
        creds: "bgg".to_owned(),
        db_name: "xx".try_into().unwrap(),
        endpoints: Default::default(),
    };
    let pool = Pool::builder(man).build().unwrap();
    pool.status();
    pool
}

async fn use_pool() {
    let pool = make_pool();
    let mut x = pool.get().await.unwrap();
    let client = x.discovery();
    let x = Arc::new(client);

}

pub struct YdbPoolBuilder<C: Credentials + Send + Sync> {
    delegate: PoolBuilder<ConnectionManager<C>>,
    update_interval: Duration,
}

impl<C: Credentials + Send + Sync> YdbPoolBuilder<C> {
    pub fn new(creds: C, db_name: AsciiValue, endpoint: EndpointInfo) -> Self {
        let endpoints =  std::sync::RwLock::new(vec![endpoint]);
        let delegate = Pool::builder(ConnectionManager {creds, db_name, endpoints});
        let update_interval = Duration::from_secs(60);
        Self {delegate, update_interval}
    }
    pub fn build(self) -> Result<Pool<ConnectionManager<C>>, deadpool::managed::BuildError<tonic::transport::Error>> {
        let pool = self.delegate.build()?;
        let result = pool.clone();
        let db_name = pool.manager().db_name.to_str().unwrap().to_owned();
        tokio::spawn(async move {
            loop {
                if pool.is_closed() {
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

async fn update_endpoints<C: Credentials + Send + Sync>(pool: &Pool<ConnectionManager<C>>, database: String) -> Result<(), YdbError> {
    let mut service = pool.get().await?;
    let mut discovery = service.discovery();
    let response = discovery.list_endpoints(ListEndpointsRequest{database, ..Default::default()}).await?; 
    let endpoints = response.into_inner().payload()?.endpoints;
    log::debug!("Pool endpoints updated ({} endpoints)", endpoints.len());
    *pool.manager().endpoints.write().unwrap() = endpoints;
    Ok(())
}