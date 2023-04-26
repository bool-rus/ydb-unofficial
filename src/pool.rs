
use std::{sync::atomic::AtomicU32, collections::hash_map::RandomState, vec, convert::Infallible, rc::Rc};

use tonic::{transport::{Channel, Endpoint, ClientTlsConfig}, codegen::BoxFuture};
use tower::{Service, balance::pool::Pool};

use crate::{generated::ydb::discovery::{ListEndpointsResult, EndpointInfo}, client::{Credentials, YdbService}};

struct YdbEndpoint {
    inner: Endpoint,
    load_factor: f32,
    connections: AtomicU32,
}

#[derive(Clone)]
pub struct YdbPool {
    endpoints: Vec<Rc<YdbEndpoint>>,
    
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


impl YdbPool {
    pub fn next_endpoint(&self) -> Endpoint {
        let mut rng = rand::thread_rng();
        let rnd: usize = rng.gen();
        use rand::Rng;
        let size = self.endpoints.iter().fold(0usize, |s, e|(e.load_factor.abs() * 10.0) as usize + 1);
        let e = self.endpoints.iter().map(|e|{
            let count = (e.load_factor.abs() * 10.0) as usize + 1;
            [e].into_iter().cycle().take(count)
        }).flatten().cycle().nth(rnd % 1000).unwrap();
        e.inner.clone()
    }
}


impl<C: Credentials> Service<C> for YdbPool {
    type Response = YdbService<C>;

    type Error = Infallible;

    type Future = BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        todo!()
    }

    fn call(&mut self, req: C) -> Self::Future {
        todo!()
    }
}
//* 
fn create_pool<C: Credentials>() -> Pool<YdbPool, C, tonic::codegen::http::Request<tonic::body::BoxBody>> {
    todo!()
}
//*/