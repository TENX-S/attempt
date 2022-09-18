mod project;

use protobuf::descriptor::{FileDescriptorProto, MethodDescriptorProto};
use protobuf::reflect::{FileDescriptor, MethodDescriptor};
use protobuf::MessageDyn;

use bytes::{Buf, BufMut};
use diesel::insertable::ColumnInsertValue::Default;
use futures::stream;
use tonic::client::Grpc;
use tonic::codec::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder};
use tonic::transport::{Channel, Uri};
use tonic::{IntoRequest, Request, Status};

type DynRequest = Box<dyn MessageDyn>;
type DynResponse = Box<dyn MessageDyn>;

fn json2request(json: String, name: String) -> anyhow::Result<DynRequest> {
    let mut file_descriptor_protos = protobuf_parse::Parser::new()
        .protoc()
        .includes(&["proto"])
        .input("proto/helloworld.proto")
        .parse_and_typecheck()
        .unwrap()
        .file_descriptors;

    let file_descriptor_proto: FileDescriptorProto = file_descriptor_protos.pop().unwrap();
    let file_descriptor = FileDescriptor::new_dynamic(file_descriptor_proto, &[])?;

    let msg_descriptor = file_descriptor
        .message_by_package_relative_name(&name)
        .unwrap();
    let mut msg = msg_descriptor.new_instance();
    protobuf_json_mapping::merge_from_str(&mut *msg, &json)?;

    Ok(msg)
}

fn buf2response(buffer: &[u8], name: String) -> anyhow::Result<DynResponse> {
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
    let msg_descriptor = file_descriptor
        .message_by_package_relative_name(&name)
        .unwrap();
    let mut msg = msg_descriptor.new_instance();
    msg.merge_from_bytes_dyn(buffer)?;
    // let json = protobuf_json_mapping::print_to_string(&*msg)?;

    Ok(msg)
}

#[derive(Clone)]
pub struct DynCodec(MethodDescriptorProto);

impl DynCodec {
    fn new(mdp: MethodDescriptorProto) -> Self {
        DynCodec(mdp)
    }
}

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
        item.write_to_bytes_dyn()
            .map(|buf| dst.put(buf.as_slice()))
            .map_err(|err| Status::internal(format!("{:?}", err)))
    }
}

impl Decoder for DynCodec {
    type Item = DynResponse;
    type Error = Status;
    fn decode(&mut self, src: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        let buf = src.chunk();
        let length = buf.len();
        let ret = buf2response(buf, self.0.clone().name.unwrap())
            .map(|resp| Some(resp))
            .map_err(|err| Status::internal(format!("{:?}", err)));
        src.advance(length);
        ret
    }
}

type JsonResponse = String;
type JsonRequest = String;
type StatusStr = String;
type MethodPath = String;
type MessageName = String;

fn make_grpc_call(
    message: MessageName,
    method: MethodPath,
    request: JsonRequest,
) -> (JsonResponse, StatusStr) {
    let mut ret = (String::default(), "OK".into());
    ret
}
