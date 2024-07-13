use common::rpc::backend_api::BackendApi;

pub struct ManagementAppGeneralState {
    backend_api: Option<BackendApi>,
}

impl ManagementAppGeneralState {
    pub fn new(backend_api: Option<BackendApi>) -> Self {
        Self {
            backend_api
        }
    }
}

#[derive(Debug, Clone)]
pub enum ManagementAppGeneralMsg {

}
