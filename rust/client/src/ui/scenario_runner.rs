#![allow(unused)]

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use anyhow::anyhow;
use gauntlet_common::model::EntrypointId;
use gauntlet_common::model::PluginId;
use gauntlet_common::model::UiTheme;
use gauntlet_server::plugins::ApplicationManager;
use iced::Task;
use iced::advanced::graphics::futures::MaybeSend;
use iced::window;
use iced::window::Screenshot;
use serde::Deserialize;
use serde::Serialize;
use tokio::time::Duration;
use tokio::time::sleep;

use crate::ui::AppMsg;
use crate::ui::windows::WindowActionMsg;

#[derive(Clone, Debug)]
pub struct ScenarioRunnerData {
    pub scenarios_dir: String,
    pub plugins_dir: String,
    pub screenshots_dir: String,
    pub only_plugin: Option<String>,
    pub only_entrypoint: Option<String>,
}

#[derive(Clone, Debug)]
pub enum ScenarioRunnerMsg {
    AddScenarioPlugin { plugin_path: String },
    Screenshot { save_path: PathBuf },
    ScreenshotDone { save_path: PathBuf, screenshot: Screenshot },
    RemoveScenarioPlugin { plugin_id: PluginId },
    Shutdown,
}

pub fn handle_scenario_runner_msg(
    msg: ScenarioRunnerMsg,
    application_manager: Arc<ApplicationManager>,
    main_window_id: window::Id,
) -> Task<ScenarioRunnerMsg> {
    match msg {
        ScenarioRunnerMsg::AddScenarioPlugin { plugin_path } => {
            application_manager
                .save_local_plugin(&plugin_path)
                .expect("Unable to save scenario plugin");

            Task::none()
        }
        ScenarioRunnerMsg::Screenshot { save_path } => {
            window::screenshot(main_window_id).map(move |screenshot| {
                ScenarioRunnerMsg::ScreenshotDone {
                    save_path: save_path.clone(),
                    screenshot,
                }
            })
        }
        ScenarioRunnerMsg::ScreenshotDone { save_path, screenshot } => {
            println!("Saving screenshot at: {:?}", save_path);

            fs::create_dir_all(Path::new(&save_path).parent().unwrap())
                .expect("unable to create scenario out directories");

            image::save_buffer_with_format(
                &save_path,
                &screenshot.bytes,
                screenshot.size.width,
                screenshot.size.height,
                image::ColorType::Rgba8,
                image::ImageFormat::Png,
            )
            .expect("Unable to save screenshot");

            Task::none()
        }
        ScenarioRunnerMsg::RemoveScenarioPlugin { plugin_id } => {
            application_manager
                .remove_plugin(plugin_id)
                .expect("Unable to remove plugin");

            Task::none()
        }
        ScenarioRunnerMsg::Shutdown => {
            tracing::info!("Shutting down scenario");

            iced::exit()
        }
    }
}

pub fn run_scenario(data: ScenarioRunnerData, theme: UiTheme) -> Task<AppMsg> {
    tracing::info!("scenario inputs: {:?}", &data);

    let scenario_data = collect_scenario_data(
        PathBuf::from(data.scenarios_dir),
        PathBuf::from(data.plugins_dir),
        PathBuf::from(data.screenshots_dir),
        data.only_plugin,
        data.only_entrypoint,
    )
    .expect("Unable to collect data");

    let mut chain = Task::none().chain(Task::done(AppMsg::SetTheme { theme }));

    let shutdown = AppMsg::HandleScenario(ScenarioRunnerMsg::Shutdown);

    for work_item in scenario_data {
        let plugin_path = work_item.plugin_path.clone();
        let plugin_id = PluginId::from_string(format!("file://{}", &plugin_path.clone()));

        let add_plugin = AppMsg::HandleScenario(ScenarioRunnerMsg::AddScenarioPlugin {
            plugin_path: plugin_path.clone(),
        });
        let remove_plugin = AppMsg::HandleScenario(ScenarioRunnerMsg::RemoveScenarioPlugin {
            plugin_id: plugin_id.clone(),
        });

        chain = chain.chain(Task::done(add_plugin)).chain(wait_for(2500));

        for work_sub_item in work_item.entrypoints {
            let entrypoint_id = EntrypointId::from_string(work_sub_item.entrypoint_id);
            let save_path = work_sub_item.screenshot_out_path;

            let show_window = AppMsg::WindowAction(WindowActionMsg::ShowWindow);
            let close_window = AppMsg::WindowAction(WindowActionMsg::HideWindow);
            let run_entrypoint = AppMsg::RunEntrypoint {
                plugin_id: plugin_id.clone(),
                entrypoint_id: entrypoint_id.clone(),
            };
            let do_screenshot = AppMsg::HandleScenario(ScenarioRunnerMsg::Screenshot { save_path });
            let open_action_panel = AppMsg::ToggleActionPanel { keyboard: false };

            let log = Task::future(async move {
                println!("Running scenario for entrypoint: {}", entrypoint_id);
            });

            chain = chain
                .chain(Task::done(show_window))
                .chain(log.discard())
                .chain(wait_for(100));

            match work_sub_item.item_type {
                ScenarioWorkSubType::InlineView { text } => {
                    chain = chain.chain(Task::done(AppMsg::PromptChanged(text)))
                }
                ScenarioWorkSubType::View => {
                    chain = chain.chain(Task::done(run_entrypoint));
                }
                ScenarioWorkSubType::ViewWithActionPanel => {
                    chain = chain
                        .chain(Task::done(run_entrypoint))
                        .chain(wait_for(500))
                        .chain(Task::done(open_action_panel));
                }
            };

            chain = chain
                .chain(wait_for(1500))
                .chain(Task::done(do_screenshot))
                .chain(Task::done(close_window))
                .chain(wait_for(500))
        }

        chain = chain.chain(Task::done(remove_plugin)).chain(wait_for(1000));
    }

    chain = chain.chain(Task::done(shutdown));

    chain
}

struct ScenarioWorkItem {
    plugin_path: String,
    entrypoints: Vec<ScenarioWorkSubItem>,
}

struct ScenarioWorkSubItem {
    entrypoint_id: String,
    screenshot_out_path: PathBuf,
    item_type: ScenarioWorkSubType,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ScenarioWorkSubType {
    InlineView { text: String },
    View,
    ViewWithActionPanel,
}

fn collect_scenario_data(
    scenarios_dir: PathBuf,
    plugins_dir: PathBuf,
    screenshot_out: PathBuf,
    only_plugin: Option<String>,
    only_entrypoint: Option<String>,
) -> anyhow::Result<Vec<ScenarioWorkItem>> {
    let mut results = vec![];

    for scenario_dir in fs::read_dir(&scenarios_dir)? {
        let scenario_dir = scenario_dir?;

        let scenario_name = scenario_dir.file_name().to_str().context("invalid UTF-8")?.to_string();

        if let Some(only_plugin) = &only_plugin {
            if only_plugin != &scenario_name {
                continue;
            }
        }

        tracing::info!("scenario: {:?}", &scenario_name);

        let plugin_path = plugins_dir.join(&scenario_name);
        let screenshot_out_dir = screenshot_out.join(&scenario_name);

        let mut entrypoints = vec![];

        for entrypoint in fs::read_dir(&scenario_dir.path())? {
            let entrypoint = entrypoint?;
            if !entrypoint.metadata()?.is_dir() {
                return Err(anyhow!("unexpected file {:?}", &entrypoint));
            }

            let entrypoint_id = entrypoint.file_name().to_str().context("invalid UTF-8")?.to_string();

            if let Some(only_entrypoint) = &only_entrypoint {
                if only_entrypoint != &entrypoint_id {
                    continue;
                }
            }

            tracing::info!("entrypoint: {}", &entrypoint_id);

            let entrypoint_dir = entrypoint.path();
            let screenshot_out_dir = screenshot_out_dir.join(&entrypoint_id);

            for scenario_item in fs::read_dir(&entrypoint_dir)? {
                let scenario_item = scenario_item?;
                if !scenario_item.metadata()?.is_file() {
                    return Err(anyhow!("unexpected file {:?}", &scenario_item));
                }

                let scenario_item_name = scenario_item
                    .path()
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .context("invalid UTF-8")?
                    .to_string();

                let scenario_item_path = scenario_item.path();

                if &scenario_item_name != "default" {
                    todo!();
                }

                tracing::info!("scenario item: {:?}", &scenario_item_name);

                let screenshot_out_path = screenshot_out_dir.join(&format!("{}.png", scenario_item_name));

                let scenario_data = fs::read(&scenario_item_path)?;

                let item_type = serde_json::from_slice(&scenario_data)?;

                entrypoints.push(ScenarioWorkSubItem {
                    entrypoint_id: entrypoint_id.clone(),
                    screenshot_out_path,
                    item_type,
                })
            }
        }

        results.push(ScenarioWorkItem {
            plugin_path: plugin_path.to_str().context("invalid UTF-8")?.to_string(),
            entrypoints,
        })
    }

    Ok(results)
}

fn wait_for<O>(millis: u64) -> Task<O>
where
    O: MaybeSend + 'static,
{
    Task::future(async move { sleep(Duration::from_millis(millis)).await }).discard()
}
