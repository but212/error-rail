use crate::common::configure_criterion;
use criterion::{criterion_group, Criterion};

#[cfg(feature = "tower")]
use crate::common::{DomainError, UserData};
#[cfg(feature = "tower")]
use core::future::{ready, Ready};
#[cfg(feature = "tower")]
use error_rail::tower::ErrorRailLayer;
#[cfg(feature = "tower")]
use std::hint::black_box;
#[cfg(feature = "tower")]
use tokio::runtime::Runtime;
#[cfg(feature = "tower")]
use tower::{Service, ServiceBuilder, ServiceExt};

#[cfg(feature = "tower")]
#[derive(Clone)]
struct EchoService;

#[cfg(feature = "tower")]
impl Service<UserData> for EchoService {
    type Response = UserData;
    type Error = DomainError;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: UserData) -> Self::Future {
        ready(Ok(req))
    }
}

#[cfg(feature = "tower")]
pub fn bench_tower_layer_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("tower/layer");

    group.bench_function("baseline_raw_service", |b| {
        let mut svc = EchoService;
        b.iter(|| {
            rt.block_on(async {
                let _ = black_box(svc.call(UserData::new(1)).await);
            })
        })
    });

    group.bench_function("error_rail_layer_success", |b| {
        let mut svc = ServiceBuilder::new()
            .layer(ErrorRailLayer::new("service-context"))
            .service(EchoService);

        b.iter(|| {
            rt.block_on(async {
                let _ = black_box(svc.ready().await.unwrap().call(UserData::new(1)).await);
            })
        })
    });

    group.finish();
}

#[cfg(feature = "tower")]
criterion_group! {
    name = tower_benches;
    config = configure_criterion();
    targets = bench_tower_layer_overhead,
}

#[cfg(not(feature = "tower"))]
criterion_group! {
    name = tower_benches;
    config = configure_criterion();
    targets = dummy
}

#[cfg(not(feature = "tower"))]
fn dummy(_c: &mut Criterion) {}
