use crate::utils::channel::RequestSender;

pub struct SearchClient {
    search: RequestSender<UiSearchRequest, Vec<UiSearchResult>>
}

impl SearchClient {
    pub fn new(search: RequestSender<UiSearchRequest, Vec<UiSearchResult>>) -> SearchClient {
        Self {
            search
        }
    }

    pub async fn search(&self, prompt: &str) -> Vec<UiSearchResult> {
        self.search.send_receive(UiSearchRequest { prompt: prompt.to_owned() })
            .await
            .unwrap()
    }
}

#[derive(Debug)]
pub struct UiSearchRequest {
    pub prompt: String
}

#[derive(Debug)]
pub struct UiSearchResult {
    pub plugin_uuid: String,
    pub plugin_name: String,
    pub entrypoint_id: String,
    pub entrypoint_name: String,
}