use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Response, Server};
use std::convert::Infallible;
use tower::Service;


#[tokio::main(flavor = "current_thread")]
async fn main() {
    let second_layer = false;

    let mut service1 = make_service_fn(move |_| async move {
        Ok::<_, Infallible>(service_fn(move |_| async move {
            Ok::<Response<_>, Infallible>(Response::new(Body::from(format!("Hello 1!"))))
        }))
    });
    let mut service2 = make_service_fn(move |_| async move {
        Ok::<_, Infallible>(service_fn(move |_| async move {
            Ok::<Response<_>, Infallible>(Response::new(Body::from(format!("Hello 2!"))))
        }))
    });
    let mut service = make_service_fn(move |inner| {
        let srv1 = service1.call(inner);
        let srv2 = service2.call(inner);
        async move {
            Ok::<_, Infallible>(if second_layer {
                tower::util::Either::A(srv1.await.unwrap())
            } else {
                tower::util::Either::B(srv2.await.unwrap())
            })
        }
    });
    let service = make_service_fn(move |inner| service.call(inner));

    let server = Server::bind(&([127, 0, 0, 1], 3000).into()).serve(service);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
