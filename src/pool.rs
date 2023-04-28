
use std::{sync::{atomic::AtomicU32, Arc}, collections::hash_map::RandomState, vec, convert::Infallible, rc::Rc, task::Poll, ops::Deref};

use deadpool::managed::{Manager, Pool};
use std::sync::Mutex;
use tonic::{transport::{Channel, Endpoint, ClientTlsConfig}, codegen::BoxFuture};
use tower::{Service, load::{PendingRequests, CompleteOnResponse}, ServiceExt};

use crate::{generated::{ydb::{discovery::{ListEndpointsResult, EndpointInfo, WhoAmIRequest}, table::v1::table_service_client::TableServiceClient}, DiscoveryServiceClient}, client::{Credentials, YdbService, AsciiValue}};

struct YdbEndpoint {
    inner: Endpoint,
    load_factor: f32,
    connections: AtomicU32,
}

pub struct Endpoints {
    endpoints: Vec<Arc<YdbEndpoint>>,
    
}

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


impl Endpoints {
    pub fn next_endpoint(&self) -> Endpoint {
        let mut rng = rand::thread_rng();
        let rnd: usize = rng.gen();
        use rand::Rng;
        let endpoints = &self.endpoints;

        let size = endpoints.iter().fold(0usize, |s, e|(e.load_factor.abs() * 10.0) as usize + 1);
        let e = endpoints.iter().map(|e|{
            let count = (e.load_factor.abs() * 10.0) as usize + 1;
            [e].into_iter().cycle().take(count)
        }).flatten().cycle().nth(rnd % 1000).unwrap();
        e.inner.clone()
    }
}


impl<C: Credentials> Service<C> for Endpoints {
    type Response = PendingRequests<YdbService<C>>;

    type Error = Infallible;

    type Future = std::future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: C) -> Self::Future {
        let endpoint = self.next_endpoint();
        let channel = endpoint.connect_lazy();
        let service = YdbService::new(channel, "bgg".try_into().unwrap(), req);
        let wrapped = PendingRequests::new(service, Default::default());
        std::future::ready(Ok(wrapped))
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
    endpoints: Endpoints,
}

#[async_trait::async_trait]
impl <C: Credentials + Sync> Manager for ConnectionManager<C> {
    type Type = YdbService<C>;

    type Error = tonic::transport::Error;

    async fn create(&self) ->  Result<Self::Type, Self::Error> {
        let endpoint = self.endpoints.next_endpoint();
        let channel = endpoint.connect().await?;
        let db_name = self.db_name.clone();
        let creds = self.creds.clone();
        Ok(YdbService::new(channel, db_name, creds))
    }

    async fn recycle(&self, obj: &mut Self::Type) ->  deadpool::managed::RecycleResult<Self::Error> {
        obj.ready().await?;
        Ok(())
    }
}

fn make_pool() -> deadpool::managed::Pool<ConnectionManager<String>> { 
    let man = ConnectionManager {
        creds: "bgg".to_owned(),
        db_name: "xx".try_into().unwrap(),
        endpoints: Endpoints { endpoints: Default::default() },
    };
    Pool::builder(man).build().unwrap()
}

async fn use_pool() {
    let pool = make_pool();
    let mut x = pool.get().await.unwrap();
    let client = x.discovery();
    let x = Arc::new(client);

}