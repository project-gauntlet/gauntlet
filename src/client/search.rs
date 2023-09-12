use crate::client::model::{NativeUiSearchRequest, NativeUiSearchResult};
use crate::utils::channel::RequestSender;

pub struct SearchClient {
    search: RequestSender<NativeUiSearchRequest, Vec<NativeUiSearchResult>>
}

impl SearchClient {
    pub fn new(search: RequestSender<NativeUiSearchRequest, Vec<NativeUiSearchResult>>) -> SearchClient {
        Self {
            search
        }
    }

    pub async fn search(&self, prompt: &str) -> Vec<NativeUiSearchResult> {
        self.search.send_receive(NativeUiSearchRequest { prompt: prompt.to_owned() })
            .await
            .unwrap()
    }
}
