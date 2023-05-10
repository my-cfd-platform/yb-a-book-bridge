fn main() {
    tonic_build::compile_protos("proto/ABookBridge.proto").unwrap();
}
