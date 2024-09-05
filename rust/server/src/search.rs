use std::cmp::Ordering;
use std::sync::{Arc, Mutex};
use tantivy::{doc, Index, IndexReader, ReloadPolicy, Searcher};
use tantivy::collector::TopDocs;
use tantivy::query::{AllQuery, BooleanQuery, FuzzyTermQuery, Query, RegexQuery, TermQuery};
use tantivy::schema::*;
use tantivy::tokenizer::TokenizerManager;
use common::model::{EntrypointId, PluginId, SearchResult, SearchResultEntrypointType};
use common::rpc::frontend_api::FrontendApi;

#[derive(Clone)]
pub struct SearchIndex {
    frontend_api: FrontendApi,
    index: Index,
    index_reader: IndexReader,
    index_writer_mutex: Arc<Mutex<()>>,

    entrypoint_type: Field,
    entrypoint_name: Field,
    entrypoint_id: Field,
    entrypoint_icon_path: Field,
    entrypoint_frecency: Field,
    plugin_name: Field,
    plugin_id: Field,
}

impl SearchIndex {
    pub fn create_index(frontend_api: FrontendApi) -> tantivy::Result<Self> {
        let schema = {
            let mut schema_builder = Schema::builder();

            schema_builder.add_text_field("entrypoint_type", STORED);
            schema_builder.add_text_field("entrypoint_name", TEXT | STORED);
            schema_builder.add_text_field("entrypoint_id", STRING | STORED);
            schema_builder.add_text_field("entrypoint_icon_path", STORED);
            schema_builder.add_text_field("entrypoint_frecency", STORED);
            schema_builder.add_text_field("plugin_name", TEXT | STORED);
            schema_builder.add_text_field("plugin_id", STRING | STORED);

            schema_builder.build()
        };

        let entrypoint_type = schema.get_field("entrypoint_type").expect("entrypoint_type field should exist");
        let entrypoint_name = schema.get_field("entrypoint_name").expect("entrypoint_name field should exist");
        let entrypoint_id = schema.get_field("entrypoint_id").expect("entrypoint_id field should exist");
        let entrypoint_icon_path = schema.get_field("entrypoint_icon_path").expect("entrypoint_icon_path field should exist");
        let entrypoint_frecency = schema.get_field("entrypoint_frecency").expect("entrypoint_frecency field should exist");
        let plugin_name = schema.get_field("plugin_name").expect("plugin_name field should exist");
        let plugin_id = schema.get_field("plugin_id").expect("plugin_id field should exist");

        let index = Index::create_in_ram(schema.clone());

        let index_reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()?;

        Ok(Self {
            frontend_api,
            index,
            index_reader,
            index_writer_mutex: Arc::new(Mutex::new(())),
            entrypoint_type,
            entrypoint_name,
            entrypoint_id,
            entrypoint_icon_path,
            entrypoint_frecency,
            plugin_name,
            plugin_id,
        })
    }

    pub fn remove_for_plugin(&self, plugin_id: PluginId) -> tantivy::Result<()> {
        // writer panics if another writer exists
        let _guard = self.index_writer_mutex.lock().expect("lock is poisoned");

        let mut index_writer = self.index.writer(5_000_000)?;

        index_writer.delete_query(Box::new(
            TermQuery::new(Term::from_field_text(self.plugin_id, &plugin_id.to_string()), IndexRecordOption::Basic)
        ))?;
        index_writer.commit()?;

        Ok(())
    }

    pub fn save_for_plugin(&mut self, plugin_id: PluginId, plugin_name: String, search_items: Vec<SearchIndexItem>, refresh_search_list: bool) -> tantivy::Result<()> {
        tracing::debug!("Reloading search index for plugin {:?} {:?} using following data: {:?}", plugin_id, plugin_name, search_items);

        // writer panics if another writer exists
        let _guard = self.index_writer_mutex.lock().expect("lock is poisoned");

        let mut index_writer = self.index.writer(3_000_000)?;

        index_writer.delete_query(Box::new(
            TermQuery::new(Term::from_field_text(self.plugin_id, &plugin_id.to_string()), IndexRecordOption::Basic)
        ))?;

        for search_item in search_items {
            index_writer.add_document(doc!(
                self.entrypoint_name => search_item.entrypoint_name,
                self.entrypoint_type => search_index_entrypoint_to_str(search_item.entrypoint_type),
                self.entrypoint_id => search_item.entrypoint_id,
                self.entrypoint_icon_path => search_item.entrypoint_icon_path.unwrap_or_default(),
                self.entrypoint_frecency => search_item.entrypoint_frecency,
                self.plugin_name => plugin_name.clone(),
                self.plugin_id => plugin_id.to_string(),
            ))?;
        }

        index_writer.commit()?;

        if refresh_search_list {
            let mut frontend_api = self.frontend_api.clone();
            tokio::spawn(async move {
                tracing::info!("requesting search results update because search index update for plugin: {:?}", plugin_id);

                let result = frontend_api.request_search_results_update()
                    .await;

                if let Err(err) = &result {
                    tracing::warn!("error occurred when requesting search results update {:?}", err)
                }
            });
        }

        Ok(())
    }

    pub fn create_handle(&self) -> SearchHandle {
        let searcher = self.index_reader.searcher();

        let query_parser = QueryParser::new(
            self.index.tokenizers().clone(),
            self.entrypoint_name,
            self.plugin_name,
        );

        SearchHandle {
            searcher,
            query_parser,
            entrypoint_type: self.entrypoint_type,
            entrypoint_name: self.entrypoint_name,
            entrypoint_id: self.entrypoint_id,
            entrypoint_icon_path: self.entrypoint_icon_path,
            entrypoint_frecency: self.entrypoint_frecency,
            plugin_name: self.plugin_name,
            plugin_id: self.plugin_id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SearchIndexItem {
    pub entrypoint_type: SearchResultEntrypointType,
    pub entrypoint_name: String,
    pub entrypoint_id: String,
    pub entrypoint_icon_path: Option<String>,
    pub entrypoint_frecency: f64,
}

pub struct SearchHandle {
    searcher: Searcher,
    query_parser: QueryParser,

    entrypoint_name: Field,
    entrypoint_type: Field,
    entrypoint_id: Field,
    entrypoint_icon_path: Field,
    entrypoint_frecency: Field,
    plugin_name: Field,
    plugin_id: Field,
}

impl SearchHandle {
    pub(crate) fn search(&self, query: &str) -> anyhow::Result<Vec<SearchResult>> {
        let query = self.query_parser.create_query(query);

        let mut index = 0;

        let fetch = std::iter::from_fn(|| -> Option<anyhow::Result<Vec<(SearchResult, f64)>>> {
            let result = self.fetch(&query, TopDocs::with_limit(20).and_offset(index * 20));

            index += 1;

            match result {
                Ok(result) => {
                    if result.is_empty() {
                        None
                    } else {
                        Some(Ok(result))
                    }
                }
                Err(error) => {
                    Some(Err(error))
                }
            }
        });

        let result = fetch.collect::<Result<Vec<Vec<_>>, _>>()?;

        let mut result = result.into_iter()
            .flatten()
            .collect::<Vec<_>>();

        result.sort_by(|(_, score_a), (_, score_b)| score_b.partial_cmp(score_a).unwrap_or(Ordering::Less));

        let result = result.into_iter()
            .map(|(item, _)| item)
            .collect::<Vec<_>>();

        Ok(result)
    }

    fn fetch(&self, query: &dyn Query, collector: TopDocs) -> anyhow::Result<Vec<(SearchResult, f64)>> {
        let get_str_field = |retrieved_doc: &Document, field: Field| -> String {
            retrieved_doc.get_first(field)
                .unwrap_or_else(|| panic!("there should be a field with name {:?}", self.searcher.schema().get_field_name(field)))
                .as_text()
                .unwrap_or_else(|| panic!("field with name {:?} should contain string", self.searcher.schema().get_field_name(field)))
                .to_owned()
        };

        let get_f64_field = |retrieved_doc: &Document, field: Field| -> f64 {
            retrieved_doc.get_first(field)
                .unwrap_or_else(|| panic!("there should be a field with name {:?}", self.searcher.schema().get_field_name(field)))
                .as_f64()
                .unwrap_or_else(|| panic!("field with name {:?} should contain string", self.searcher.schema().get_field_name(field)))
        };

        let result = self.searcher.search(query, &collector)?
            .into_iter()
            .map(|(_score, doc_address)| {
                let retrieved_doc = self.searcher.doc(doc_address)
                    .expect("index should contain just searched results");

                let score = get_f64_field(&retrieved_doc, self.entrypoint_frecency);

                let result_item = SearchResult {
                    entrypoint_type: search_index_entrypoint_from_str(&get_str_field(&retrieved_doc, self.entrypoint_type)),
                    entrypoint_name: get_str_field(&retrieved_doc, self.entrypoint_name),
                    entrypoint_id: EntrypointId::from_string(get_str_field(&retrieved_doc, self.entrypoint_id)),
                    entrypoint_icon: Some(get_str_field(&retrieved_doc, self.entrypoint_icon_path)).filter(|value| value != ""),
                    plugin_name: get_str_field(&retrieved_doc, self.plugin_name),
                    plugin_id: PluginId::from_string(get_str_field(&retrieved_doc, self.plugin_id)),
                };

                (result_item, score)
            })
            .collect::<Vec<_>>();

        Ok(result)
    }
}

struct QueryParser {
    tokenizer_manager: TokenizerManager,
    entrypoint_name: Field,
    plugin_name: Field,
}

impl QueryParser {
    fn new(tokenizer_manager: TokenizerManager, entrypoint_name: Field, plugin_name: Field) -> Self {
        Self {
            tokenizer_manager,
            entrypoint_name,
            plugin_name,
        }
    }

    fn create_query(&self, query: &str) -> Box<dyn Query> {
        if query.is_empty() {
            return Box::new(AllQuery);
        }

        let contains_terms_fn = |field: Field| -> Box<dyn Query> {
            let res = self.tokenize(query)
                .into_iter()
                .map(|term| -> Box<dyn Query> {
                    Box::new(
                        // basically a "contains" query
                        RegexQuery::from_pattern(&format!(".*{}.*", regex::escape(&term)), field)
                            .expect("there should not exist a situation where that regex is invalid")
                    )
                })
                .collect::<Vec<_>>();

            Box::new(BooleanQuery::intersection(res))
        };

        let terms_fn = |field: Field| -> Box<dyn Query> {
            Box::new(
                contains_terms_fn(field)
            )
        };

        let entrypoint_name_terms = terms_fn(self.entrypoint_name);
        let plugin_name_terms = terms_fn(self.plugin_name);

        Box::new(
            BooleanQuery::union(vec![
                Box::new(entrypoint_name_terms),
                Box::new(plugin_name_terms),
            ]),
        )
    }

    fn tokenize(&self, query: &str) -> Vec<String> {
        let mut text_analyzer = self
            .tokenizer_manager
            .get("default")
            .expect("default tokenizer should exist");

        let mut terms: Vec<String> = Vec::new();
        let mut token_stream = text_analyzer.token_stream(query);
        token_stream.process(&mut |token| {
            terms.push(token.text.to_string());
        });

        terms
    }
}

fn search_index_entrypoint_to_str(value: SearchResultEntrypointType) -> &'static str {
    match value {
        SearchResultEntrypointType::Command => "command",
        SearchResultEntrypointType::View => "view",
        SearchResultEntrypointType::GeneratedCommand => "generated-command",
    }
}

fn search_index_entrypoint_from_str(value: &str) -> SearchResultEntrypointType {
    match value {
        "command" => SearchResultEntrypointType::Command,
        "view" => SearchResultEntrypointType::View,
        "generated-command" => SearchResultEntrypointType::GeneratedCommand,
        _ => panic!("index contains illegal entrypoint_type: {}", value)
    }
}