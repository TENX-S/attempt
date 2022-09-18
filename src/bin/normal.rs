extern crate core;

use hello_world::greeter_client::GreeterClient;
use hello_world::Number;

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GreeterClient::connect("http://127.0.0.1:8080").await?;

    let request = tonic::Request::new(Number { data: 7 });

    let mut response = client.get_ones(request).await?.into_inner();
    while let Some(num) = response.message().await? {
        println!("{}", num.data);
    }

    Ok(())
}
