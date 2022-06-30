use bytes::{Buf, BufMut};
use tonic::client::Grpc;
use tonic::codegen::http::uri::PathAndQuery;
use tonic::transport::{Channel, Uri};
use tonic::{IntoRequest, Status};

use protobuf::descriptor::FileDescriptorProto;
use protobuf::reflect::FileDescriptor;
use protobuf::reflect::ReflectValueBox;
use protobuf::MessageDyn;
use tonic::codec::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder};

type DynRequest = Box<dyn MessageDyn>;
type DynResponse = Box<dyn MessageDyn>;

fn make_request() -> DynRequest {
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

fn get_response(buffer: &[u8]) -> DynResponse {
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
        .message_by_package_relative_name("HelloReply")
        .unwrap();
    let mut mmm = mmm_descriptor.new_instance();
    mmm.merge_from_bytes_dyn(buffer).unwrap();
    let json = protobuf_json_mapping::print_to_string(&*mmm).unwrap();
    println!("reply: {}", json);
    // println!("{:#?}", mmm.descriptor_dyn().field_by_name("message").unwrap().proto());
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
        let resp = get_response(src.chunk());
        Ok(Some(resp))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let uri = Uri::builder()
        .scheme("http")
        .authority("127.0.0.1:8080")
        .path_and_query("")
        .build()
        .unwrap();
    let chan = Channel::builder(uri).connect().await.unwrap();
    let mut client = Grpc::new(chan);
    let req = make_request().into_request();
    let path = PathAndQuery::from_static("/helloworld.Greeter/SayHello");
    client.ready().await.unwrap();
    let resp = client.unary(req, path, DynCodec).await;
    println!("{:#?}", resp);
    Ok(())
}
