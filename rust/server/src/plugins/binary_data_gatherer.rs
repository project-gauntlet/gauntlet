use std::collections::HashMap;
use std::io::Read;

use futures::StreamExt;
use gauntlet_common::model::DataSource;
use gauntlet_common::model::DataSourceAsset;
use gauntlet_common::model::DataSourceUrl;
use gauntlet_common::model::ImageLike;
use gauntlet_common::model::RootWidget;
use gauntlet_common::model::SvgWidget;
use gauntlet_common::model::UiWidgetId;
use gauntlet_common::model::WidgetVisitor;
use gauntlet_plugin_runtime::BackendForPluginRuntimeApi;

use crate::plugins::js::BackendForPluginRuntimeApiImpl;

pub struct BinaryDataGatherer<'a> {
    api: &'a BackendForPluginRuntimeApiImpl,
    data: HashMap<UiWidgetId, anyhow::Result<Vec<u8>>>,
}

impl<'a> WidgetVisitor for BinaryDataGatherer<'a> {
    async fn image(&mut self, widget_id: UiWidgetId, widget: &ImageLike) {
        if let ImageLike::DataSource(image_source) = &widget {
            self.data.insert(widget_id, get_data(&self.api, image_source).await);
        }
    }

    async fn svg_widget(&mut self, widget: &SvgWidget) {
        self.data
            .insert(widget.__id__, get_data(&self.api, &widget.source).await);
    }
}

impl<'a> BinaryDataGatherer<'a> {
    pub async fn run_gatherer(
        api: &'a BackendForPluginRuntimeApiImpl,
        root_widget: &RootWidget,
    ) -> anyhow::Result<HashMap<UiWidgetId, Vec<u8>>> {
        let mut gatherer = Self {
            api,
            data: HashMap::new(),
        };

        gatherer.root_widget(root_widget).await;

        gatherer
            .data
            .into_iter()
            .map(|(widget_id, image)| image.map(|image| (widget_id, image)))
            .collect::<anyhow::Result<_>>()
    }
}

async fn get_data(api: &BackendForPluginRuntimeApiImpl, source: &DataSource) -> anyhow::Result<Vec<u8>> {
    match source {
        DataSource::DataSourceAsset(DataSourceAsset { asset }) => {
            let bytes = api.get_asset_data(&asset).await?;

            Ok(bytes)
        }
        DataSource::DataSourceUrl(DataSourceUrl { url }) => {
            // FIXME implement error handling so it doesn't error whole view
            // TODO implement caching

            let bytes = ureq::get(&url)
                .call()?
                .into_reader()
                .bytes()
                .collect::<std::io::Result<Vec<u8>>>()?
                .into();

            Ok(bytes)
        }
    }
}
