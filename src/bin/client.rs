#![allow(unused, dead_code)]
use std::str::FromStr;
use std::thread;
use std::time::Duration;

use bytes::{Buf, BufMut};
use futures::stream;
use http::uri::PathAndQuery;
use tonic::client::Grpc;
use tonic::transport::{Channel, Uri};
use tonic::{Code, IntoRequest, Request, Status};

use protobuf::descriptor::FileDescriptorProto;
use protobuf::reflect::FileDescriptor;
use protobuf::reflect::ReflectValueBox;
use protobuf::MessageDyn;
use tonic::codec::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder};
use tonic::metadata::MetadataValue;

type DynRequest = Box<dyn MessageDyn>;
type DynResponse = Box<dyn MessageDyn>;

fn make_unary_request() -> DynRequest {
    let mut file_descriptor_protos = protobuf_parse::Parser::new()
        .protoc()
        .includes(&["proto"])
        .input("proto/helloworld.proto")
        .parse_and_typecheck()
        .unwrap()
        .file_descriptors;
    assert_eq!(1, file_descriptor_protos.len());

    let file_descriptor_proto: FileDescriptorProto = file_descriptor_protos.pop().unwrap();
    let file_descriptor: FileDescriptor =
        FileDescriptor::new_dynamic(file_descriptor_proto, &[]).unwrap();
    let mmm_descriptor = file_descriptor
        .message_by_package_relative_name("HelloRequest")
        .unwrap();
    let mut mmm = mmm_descriptor.new_instance();
    let age_field = mmm_descriptor.field_by_name("name").unwrap();
    age_field.set_singular_field(&mut *mmm, ReflectValueBox::String("World".into()));
    let json = protobuf_json_mapping::print_to_string(&*mmm).unwrap();
    println!("request: {}", json);
    mmm
}

fn make_stream_request(num: i32) -> DynRequest {
    let mut file_descriptor_protos = protobuf_parse::Parser::new()
        .protoc()
        .includes(&["proto"])
        .input("proto/helloworld.proto")
        .parse_and_typecheck()
        .unwrap()
        .file_descriptors;
    assert_eq!(1, file_descriptor_protos.len());

    let file_descriptor_proto: FileDescriptorProto = file_descriptor_protos.pop().unwrap();
    let file_descriptor: FileDescriptor =
        FileDescriptor::new_dynamic(file_descriptor_proto, &[]).unwrap();
    let mmm_descriptor = file_descriptor
        .message_by_package_relative_name("Number")
        .unwrap();
    let mut mmm = mmm_descriptor.new_instance();
    let age_field = mmm_descriptor.field_by_name("data").unwrap();
    age_field.set_singular_field(&mut *mmm, ReflectValueBox::I32(num));
    let json = protobuf_json_mapping::print_to_string(&*mmm).unwrap();
    println!("push request: {}", json);
    mmm
}

fn get_ones_request() -> DynRequest {
    let mut file_descriptor_protos = protobuf_parse::Parser::new()
        .protoc()
        .includes(&["proto"])
        .input("proto/helloworld.proto")
        .parse_and_typecheck()
        .unwrap()
        .file_descriptors;
    assert_eq!(1, file_descriptor_protos.len());

    let file_descriptor_proto: FileDescriptorProto = file_descriptor_protos.pop().unwrap();
    let file_descriptor: FileDescriptor =
        FileDescriptor::new_dynamic(file_descriptor_proto, &[]).unwrap();
    let mmm_descriptor = file_descriptor
        .message_by_package_relative_name("Number")
        .unwrap();
    let mut mmm = mmm_descriptor.new_instance();
    let age_field = mmm_descriptor.field_by_name("data").unwrap();
    age_field.set_singular_field(&mut *mmm, ReflectValueBox::I32(7));
    let json = protobuf_json_mapping::print_to_string(&*mmm).unwrap();
    println!("request: {}", json);
    mmm
}

fn ones_reply(buffer: &[u8]) -> DynResponse {
    let mut file_descriptor_protos = protobuf_parse::Parser::new()
        .protoc()
        .includes(&["proto"])
        .input("proto/helloworld.proto")
        .parse_and_typecheck()
        .unwrap()
        .file_descriptors;
    assert_eq!(1, file_descriptor_protos.len());

    let file_descriptor_proto: FileDescriptorProto = file_descriptor_protos.pop().unwrap();
    let file_descriptor: FileDescriptor =
        FileDescriptor::new_dynamic(file_descriptor_proto, &[]).unwrap();
    let mmm_descriptor = file_descriptor
        .message_by_package_relative_name("Number")
        .unwrap();
    let mut mmm = mmm_descriptor.new_instance();
    mmm.merge_from_bytes_dyn(buffer).unwrap();
    let json = protobuf_json_mapping::print_to_string(&*mmm).unwrap();
    // println!("reply: {}", json);
    mmm
}

fn hello_reply(buffer: &[u8]) -> DynResponse {
    let mut file_descriptor_protos = protobuf_parse::Parser::new()
        .protoc()
        .includes(&["proto"])
        .input("proto/helloworld.proto")
        .parse_and_typecheck()
        .unwrap()
        .file_descriptors;
    assert_eq!(1, file_descriptor_protos.len());

    let file_descriptor_proto: FileDescriptorProto = file_descriptor_protos.pop().unwrap();
    // let file_proto_json = protobuf_json_mapping::print_to_string(file_descriptor_proto.descriptor_dyn().to_string());

    let file_descriptor: FileDescriptor =
        FileDescriptor::new_dynamic(file_descriptor_proto, &[]).unwrap();
    let mmm_descriptor = file_descriptor
        .message_by_package_relative_name("HelloReply")
        .unwrap();
    let mut mmm = mmm_descriptor.new_instance();
    mmm.merge_from_bytes_dyn(buffer).unwrap();
    let json = protobuf_json_mapping::print_to_string(&*mmm).unwrap();
    println!("reply: {}", json);
    println!(
        "{:#?}",
        mmm.descriptor_dyn()
            .field_by_name("message")
            .unwrap()
            .proto()
    );
    mmm
}

fn number_reply(buffer: &[u8]) -> DynResponse {
    let mut file_descriptor_protos = protobuf_parse::Parser::new()
        .protoc()
        .includes(&["proto"])
        .input("proto/helloworld.proto")
        .parse_and_typecheck()
        .unwrap()
        .file_descriptors;
    assert_eq!(1, file_descriptor_protos.len());

    let file_descriptor_proto: FileDescriptorProto = file_descriptor_protos.pop().unwrap();
    let file_descriptor: FileDescriptor =
        FileDescriptor::new_dynamic(file_descriptor_proto, &[]).unwrap();
    let mmm_descriptor = file_descriptor
        .message_by_package_relative_name("Number")
        .unwrap();
    let mut mmm = mmm_descriptor.new_instance();
    mmm.merge_from_bytes_dyn(buffer).unwrap();
    // let json = protobuf_json_mapping::print_to_string(&*mmm).unwrap();
    mmm
}

#[derive(Clone)]
pub struct DynCodec;

impl Codec for DynCodec {
    type Encode = DynRequest;
    type Decode = DynResponse;

    type Encoder = DynCodec;
    type Decoder = DynCodec;

    fn encoder(&mut self) -> Self::Encoder {
        self.clone()
    }

    fn decoder(&mut self) -> Self::Decoder {
        self.clone()
    }
}

impl Encoder for DynCodec {
    type Item = DynRequest;
    type Error = Status;
    fn encode(&mut self, item: Self::Item, dst: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
        let buf = item.write_to_bytes_dyn().unwrap();
        dst.put(buf.as_slice());
        Ok(())
    }
}

impl Decoder for DynCodec {
    type Item = DynResponse;
    type Error = Status;

    fn decode(&mut self, src: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        let buf = src.chunk();
        let length = buf.len();
        let resp = hello_reply(buf);
        src.advance(length);
        Ok(Some(resp))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let uri = Uri::from_str("http://127.0.0.1:8080").map_err(|e| e.to_string())?;

    let chan = Channel::builder(uri)
        .connect()
        .await
        .map_err(|e| e.to_string())?;
    let mut client = Grpc::new(chan);
    client.ready().await.unwrap();

    // *** unary ***
    let mut req = make_unary_request().into_request();
    req.metadata_mut()
        .insert("p.s.", MetadataValue::try_from("hello again")?);
    let path = PathAndQuery::from_static("/helloworld.Greeter/SayHello");
    let resp = client.unary(req, path, DynCodec).await?;
    println!("{:#?}", resp.metadata().get("resp"));
    // *** Server Streaming ***
    // let req = get_ones_request().into_request();
    // let path = PathAndQuery::from_static("/helloworld.Greeter/GetOnes");
    // let mut resp = client
    //     .server_streaming(req, path, DynCodec)
    //     .await?
    //     .into_inner();
    //
    // while let Some(num) = resp.message().await? {
    //     let json = protobuf_json_mapping::print_to_string(&*num).unwrap();
    //     println!("reply: {}", json);
    // }

    // *** Client Streaming ***
    // let mut reqs = vec![];
    // for i in 1..=10 {
    //     reqs.push(make_stream_request(i));
    // }
    // let path = PathAndQuery::from_static("/helloworld.Greeter/CalcSum");
    // let resp = client
    //     .client_streaming(Request::new(stream::iter(reqs)), path, DynCodec)
    //     .await;

    // *** Bidirectional Streaming ***
    // let outbound = async_stream::stream! {
    //     let mut i = 1;
    //     let mut interval = tokio::time::interval(Duration::from_secs(1));
    //     loop {
    //         let time = interval.tick().await;
    //         yield make_stream_request(i);
    //         i += 1;
    //     }
    // };
    //
    // let path = PathAndQuery::from_static("/helloworld.Greeter/GetDoubles");
    // let resp = client
    //     .streaming(Request::new(outbound), path, DynCodec)
    //     .await
    //     .unwrap();
    // let mut inbound = resp.into_inner();
    // while let Some(num) = inbound.message().await? {
    //     let json = protobuf_json_mapping::print_to_string(&*num).unwrap();
    //     println!("reply: {}", json);
    // }

    Ok(())
}
