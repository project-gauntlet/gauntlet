use std::sync::{Arc, Mutex};
use tantivy::{doc, Index, IndexReader, ReloadPolicy, Searcher};
use tantivy::collector::TopDocs;
use tantivy::query::{AllQuery, BooleanQuery, FuzzyTermQuery, Query, TermQuery};
use tantivy::schema::*;
use tantivy::tokenizer::TokenizerManager;
use common::model::PluginId;

#[derive(Clone)]
pub struct SearchIndex {
    index: Index,
    index_reader: IndexReader,
    index_writer_mutex: Arc<Mutex<()>>,

    entrypoint_type: Field,
    entrypoint_name: Field,
    entrypoint_id: Field,
    plugin_name: Field,
    plugin_id: Field,
}

impl SearchIndex {
    pub fn create_index() -> tantivy::Result<Self> {
        let schema = {
            let mut schema_builder = Schema::builder();

            schema_builder.add_text_field("entrypoint_type", STORED);
            schema_builder.add_text_field("entrypoint_name", TEXT | STORED);
            schema_builder.add_text_field("entrypoint_id", STORED);
            schema_builder.add_text_field("plugin_name", TEXT | STORED);
            schema_builder.add_text_field("plugin_id", STORED);

            schema_builder.build()
        };

        let entrypoint_type = schema.get_field("entrypoint_type").expect("entrypoint_type field should exist");
        let entrypoint_name = schema.get_field("entrypoint_name").expect("entrypoint_name field should exist");
        let entrypoint_id = schema.get_field("entrypoint_id").expect("entrypoint_id field should exist");
        let plugin_name = schema.get_field("plugin_name").expect("plugin_name field should exist");
        let plugin_id = schema.get_field("plugin_id").expect("plugin_id field should exist");

        let index = Index::create_in_ram(schema.clone());

        let index_reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()?;

        Ok(Self {
            index,
            index_reader,
            index_writer_mutex: Arc::new(Mutex::new(())),
            entrypoint_type,
            entrypoint_name,
            entrypoint_id,
            plugin_name,
            plugin_id,
        })
    }

    pub fn remove_for_plugin(&self, plugin_id: PluginId) -> tantivy::Result<()> {
        let mut index_writer = self.index.writer(5_000_000)?;

        index_writer.delete_term(Term::from_field_text(self.plugin_id, &plugin_id.to_string()));
        index_writer.commit()?;

        Ok(())
    }

    pub fn save_for_plugin(&self, plugin_id: PluginId, plugin_name: String, search_items: Vec<SearchIndexItem>) -> tantivy::Result<()> {
        tracing::debug!("Reloading search index for plugin {:?} {:?} using following data: {:?}", plugin_id, plugin_name, search_items);

        // writer panics if another writer exists
        let _guard = self.index_writer_mutex.lock().expect("lock is poisoned");

        let mut index_writer = self.index.writer(3_000_000)?;

        index_writer.delete_term(Term::from_field_text(self.plugin_id, &plugin_id.to_string()));

        for search_item in search_items {
            index_writer.add_document(doc!(
                self.entrypoint_name => search_item.entrypoint_name,
                self.entrypoint_type => search_index_entrypoint_to_str(search_item.entrypoint_type),
                self.entrypoint_id => search_item.entrypoint_id,
                self.plugin_name => plugin_name.clone(),
                self.plugin_id => plugin_id.to_string(),
            ))?;
        }

        index_writer.commit()?;

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
            plugin_name: self.plugin_name,
            plugin_id: self.plugin_id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SearchIndexItem {
    pub entrypoint_type: SearchIndexPluginEntrypointType,
    pub entrypoint_name: String,
    pub entrypoint_id: String,
}

#[derive(Clone, Debug)]
pub struct SearchResultItem {
    pub entrypoint_type: SearchIndexPluginEntrypointType,
    pub entrypoint_name: String,
    pub entrypoint_id: String,
    pub plugin_name: String,
    pub plugin_id: String,
}

pub struct SearchHandle {
    searcher: Searcher,
    query_parser: QueryParser,

    entrypoint_name: Field,
    entrypoint_type: Field,
    entrypoint_id: Field,
    plugin_name: Field,
    plugin_id: Field,
}

impl SearchHandle {
    pub(crate) fn search(&self, query: &str) -> anyhow::Result<Vec<SearchResultItem>> {
        let query = self.query_parser.create_query(query);

        let mut index = 0;

        let fetch = std::iter::from_fn(|| -> Option<anyhow::Result<Vec<SearchResultItem>>> {
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

        Ok(result.into_iter().flatten().collect::<Vec<_>>())
    }

    fn fetch(&self, query: &dyn Query, collector: TopDocs) -> anyhow::Result<Vec<SearchResultItem>> {
        let get_str_field = |retrieved_doc: &Document, field: Field| -> String {
            retrieved_doc.get_first(field)
                .unwrap_or_else(|| panic!("there should be a field with name {:?}", self.searcher.schema().get_field_name(field)))
                .as_text()
                .unwrap_or_else(|| panic!("field with name {:?} should contain string", self.searcher.schema().get_field_name(field)))
                .to_owned()
        };

        let result = self.searcher.search(query, &collector)?
            .into_iter()
            .map(|(_score, doc_address)| {
                let retrieved_doc = self.searcher.doc(doc_address)
                    .expect("index should contain just searched results");

                SearchResultItem {
                    entrypoint_type: search_index_entrypoint_from_str(&get_str_field(&retrieved_doc, self.entrypoint_type)),
                    entrypoint_name: get_str_field(&retrieved_doc, self.entrypoint_name),
                    entrypoint_id: get_str_field(&retrieved_doc, self.entrypoint_id),
                    plugin_name: get_str_field(&retrieved_doc, self.plugin_name),
                    plugin_id: get_str_field(&retrieved_doc, self.plugin_id),
                }
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

        // TODO https://github.com/quickwit-oss/tantivy/issues/563
        //  fuzzy search scoring doesn't account for levenshtein distance
        //  which means results don't make sense
        //  if distance is > 0
        //  FuzzyTermQuery is used because it supports prefix matching

        let fuzzy_terms_fn = |field: Field| -> Box<dyn Query> {
            let res = self.tokenize(field, query)
                .into_iter()
                .map(|term| -> Box<dyn Query> {
                    Box::new(
                        FuzzyTermQuery::new_prefix(term, 0, false)
                    )
                })
                .collect::<Vec<_>>();

            Box::new(BooleanQuery::union(res))
        };

        let terms_fn = |field: Field| -> Box<dyn Query> {
            Box::new(
                fuzzy_terms_fn(field)
            )
        };

        let entrypoint_name_terms = terms_fn(self.entrypoint_name);
        let plugin_name_terms = terms_fn(self.plugin_name);

        return Box::new(
            BooleanQuery::union(vec![
                Box::new(entrypoint_name_terms),
                Box::new(plugin_name_terms),
            ]),
        );
    }

    fn tokenize(&self, field: Field, query: &str) -> Vec<Term> {
        let mut text_analyzer = self
            .tokenizer_manager
            .get("default")
            .expect("default tokenizer should exist");

        let mut terms: Vec<Term> = Vec::new();
        let mut token_stream = text_analyzer.token_stream(query);
        token_stream.process(&mut |token| {
            terms.push(Term::from_field_text(field, &token.text));
        });

        terms
    }
}


#[derive(Debug, Clone)]
pub enum SearchIndexPluginEntrypointType {
    Command,
    View,
    GeneratedCommand,
}

fn search_index_entrypoint_to_str(value: SearchIndexPluginEntrypointType) -> &'static str {
    match value {
        SearchIndexPluginEntrypointType::Command => "command",
        SearchIndexPluginEntrypointType::View => "view",
        SearchIndexPluginEntrypointType::GeneratedCommand => "generated-command",
    }
}

fn search_index_entrypoint_from_str(value: &str) -> SearchIndexPluginEntrypointType {
    match value {
        "command" => SearchIndexPluginEntrypointType::Command,
        "view" => SearchIndexPluginEntrypointType::View,
        "generated-command" => SearchIndexPluginEntrypointType::GeneratedCommand,
        _ => panic!("index contains illegal entrypoint_type: {}", value)
    }
}