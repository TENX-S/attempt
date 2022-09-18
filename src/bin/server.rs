#![feature(generators)]

extern crate core;

// fn main() {}

use futures::{Stream, StreamExt};
use std::pin::Pin;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status, Streaming};

use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::Number;
use hello_world::{HelloReply, HelloRequest};

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[derive(Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request from {:?}", request.remote_addr());
        let reply = hello_world::HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };
        Ok(Response::new(reply))
    }

    async fn calc_sum(
        &self,
        request: Request<Streaming<Number>>,
    ) -> Result<Response<Number>, Status> {
        println!("Got a request from {:?}", request.remote_addr());
        let mut stream = request.into_inner();
        let mut number = Number::default();
        while let Some(Ok(num)) = stream.next().await {
            number.data += num.data
        }
        Ok(Response::new(number))
    }

    type GetOnesStream = ReceiverStream<Result<Number, Status>>;

    async fn get_ones(
        &self,
        request: Request<Number>,
    ) -> Result<Response<Self::GetOnesStream>, Status> {
        println!("Got a request from {:?}", request.remote_addr());
        let num = request.into_inner().data as usize;
        let (tx, rx) = mpsc::channel(num);

        tokio::spawn(async move {
            for _ in 1..=num {
                let number = Number { data: 1 };
                tx.send(Ok(number)).await.unwrap();
            }
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    type GetDoublesStream = Pin<Box<dyn Stream<Item = Result<Number, Status>> + Send + 'static>>;

    async fn get_doubles(
        &self,
        request: Request<Streaming<Number>>,
    ) -> Result<Response<Self::GetDoublesStream>, Status> {
        println!("Got a request from {:?}", request.remote_addr());
        let stream = request.into_inner();
        // let (tx, rx) = mpsc::unbounded_channel();
        let output = async_stream::try_stream! {
            while let Some(num) = stream.next().await {
                tokio::spawn(async move {
                    let num = num.unwrap();
                    for _ in 1..=num.data {
                        yield Number { data: 2}
                    }
                });
            }
        };
        Ok(Response::new(Box::pin(output) as Self::GetDoublesStream))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080".parse().unwrap();
    let greeter = MyGreeter::default();

    println!("GreeterServer listening on {}", addr);

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
