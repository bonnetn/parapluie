
pub mod parapluie {
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("service_descriptor");
    tonic::include_proto!("parapluie");
}
