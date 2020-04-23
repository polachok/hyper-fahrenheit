use fahrenheit;
use hyper::{Client, Uri};
use hyper_fahrenheit::{Connector, FahrenheitExecutor};

fn main() {
    fahrenheit::run(async move {
        let client: Client<Connector, hyper::Body> = Client::builder()
            .executor(FahrenheitExecutor)
            .build(Connector);
        let res = client
            .get(Uri::from_static("http://httpbin.org/ip"))
            .await
            .unwrap();
        println!("status: {}", res.status());
        let buf = hyper::body::to_bytes(res).await.unwrap();
        println!("body: {:?}", buf);
    });
}
