fn main() {
    embed_resource::compile("assets/ryocs.rc", embed_resource::NONE).manifest_optional().unwrap();
}
