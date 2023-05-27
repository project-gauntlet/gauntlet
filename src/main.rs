use std::any::type_name;
use std::ffi::c_void;
use std::future::Future;
use std::path::Path;
use std::rc::Rc;

use deno_core::{op, OpState, serde_v8, v8};
use deno_runtime::deno_core::error::AnyError;
use deno_runtime::deno_core::FsModuleLoader;
use deno_runtime::deno_core::ModuleSpecifier;
use deno_runtime::permissions::PermissionsContainer;
use deno_runtime::worker::MainWorker;
use deno_runtime::worker::WorkerOptions;
use gtk::glib;
use gtk::glib::ffi::GType;
use gtk::glib::translate::{FromGlib, FromGlibPtrFull, FromGlibPtrNone, IntoGlib, Ptr, ToGlibPtr};
use gtk::prelude::*;

const APP_ID: &str = "org.gtk_rs.HelloWorld2";

fn main() -> glib::ExitCode {
    let app = gtk::Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);

    app.run()
}

#[op(v8)]
pub fn op_gtk_get_container<'scope>(
    scope: &mut v8::HandleScope<'scope>,
    state: &mut OpState,
) -> serde_v8::Value<'scope> {
    let root_container = state.borrow::<gtk::Box>();
    let root_container = root_container.clone().upcast::<gtk::Widget>();

    from_gtk_widget_to_js(scope, root_container.clone())
}

#[op(v8)]
pub fn op_gtk_append_child<'scope>(
    scope: &mut v8::HandleScope<'scope>,
    parent: serde_v8::Value<'scope>,
    child: serde_v8::Value<'scope>,
) {
    let parent = from_js_to_gtk_widget(scope, parent)
        .downcast::<gtk::Box>()
        .unwrap();
    let child = from_js_to_gtk_widget(scope, child)
        .downcast::<gtk::Widget>()
        .unwrap();

    parent.append(&child)
}

#[op(v8)]
pub fn op_gtk_remove_child<'scope>(
    scope: &mut v8::HandleScope<'scope>,
    parent: serde_v8::Value<'scope>,
    child: serde_v8::Value<'scope>,
) {
    let parent = from_js_to_gtk_widget(scope, parent)
        .downcast::<gtk::Box>()
        .unwrap();
    let child = from_js_to_gtk_widget(scope, child)
        .downcast::<gtk::Widget>()
        .unwrap();

    // TODO somehow make sure we do not have dangling pointers

    parent.remove(&child)
}

#[op(v8)]
pub fn op_gtk_insert_before<'scope>(
    scope: &mut v8::HandleScope<'scope>,
    parent: serde_v8::Value<'scope>,
    child: serde_v8::Value<'scope>,
    before_child: serde_v8::Value<'scope>,
) {
    let parent = from_js_to_gtk_widget(scope, parent)
        .downcast::<gtk::Box>()
        .unwrap();
    let child = from_js_to_gtk_widget(scope, child)
        .downcast::<gtk::Widget>()
        .unwrap();
    let before_child = from_js_to_gtk_widget(scope, before_child)
        .downcast::<gtk::Widget>()
        .unwrap();

    child.insert_before(&parent, Some(&before_child))
}

#[op(v8)]
pub fn op_gtk_create_instance<'scope>(
    scope: &mut v8::HandleScope<'scope>,
    jsx_type: serde_v8::Value<'scope>,
    props: serde_v8::Value<'scope>,
) -> serde_v8::Value<'scope> {
    let jsx_type: v8::Local<v8::Value> = jsx_type.into();
    let jsx_type: v8::Local<v8::String> = jsx_type.try_into().unwrap();
    let jsx_type: String = jsx_type.to_rust_string_lossy(scope);

    let props: v8::Local<v8::Value> = props.into();
    let props: v8::Local<v8::Object> = props.try_into().unwrap();

    let widget: gtk::Widget = match jsx_type.as_str() {
        "box" => gtk::Box::new(gtk::Orientation::Horizontal, 6).into(),
        "button1" => gtk::Box::new(gtk::Orientation::Horizontal, 6).into(),
        // "button1" => {
        //     let children_name = v8::String::new(scope, "children").unwrap();
        //     let on_click_name = v8::String::new(scope, "onClick").unwrap();
        //
        //     let children = props.get(scope, children_name.into()).unwrap();
        //     let children: v8::Local<v8::String> = children.try_into().unwrap();
        //     let children: String = children.to_rust_string_lossy(scope);
        //
        //     let on_click = props.get(scope, on_click_name.into()).unwrap();
        //     let on_click: v8::Local<v8::Function> = on_click.try_into().unwrap();
        //
        //     // let nested_scope = v8::HandleScope::new(scope);
        //
        //     // TODO: not sure if lifetime of children is ok here
        //     let button = gtk::Button::with_label(&children);
        //     button.connect_clicked(|button| {
        //         // let nested_scope = &mut nested_scope;
        //
        //         let null: v8::Local<v8::Value> = v8::null(scope).into();
        //
        //         on_click.call(scope, null, &[]);
        //     });
        //
        //     button.into()
        // }
        _ => panic!("jsx_type {} not supported", jsx_type)
    };

    from_gtk_widget_to_js(scope, widget.upcast::<gtk::Widget>())
}

#[op(v8)]
pub fn op_gtk_create_text_instance<'scope>(
    scope: &mut v8::HandleScope<'scope>,
    text: String,
) -> serde_v8::Value<'scope> {
    let label = gtk::Label::new(Some(&text));

    from_gtk_widget_to_js(scope, label.upcast::<gtk::Widget>())
}

deno_core::extension!(
    gtk_ext,
    ops = [
        op_gtk_get_container,
        op_gtk_create_instance,
        op_gtk_create_text_instance,
        op_gtk_append_child,
        op_gtk_insert_before,
    ],
    options = {
        root_container: gtk::Box,
    },
    state = |state, options| {
        state.put(options.root_container);
    },
    customizer = |ext: &mut deno_core::ExtensionBuilder| {
        ext.force_op_registration();
    },
);

const POINTER_FIELD: &str = "__ptr__";
const GTYPE_FIELD: &str = "__gtype__";

fn from_js_to_gtk_widget<'scope>(
    scope: &mut v8::HandleScope<'scope>,
    widget: serde_v8::Value<'scope>,
) -> glib::Object {
    let widget: v8::Local<v8::Value> = widget.into();
    let widget: v8::Local<v8::Object> = widget.try_into().unwrap();

    // let gtype = {
    //     let gtype_name = v8::String::new(scope, GTYPE_FIELD).unwrap();
    //
    //     let prototype = widget.get_prototype(scope).unwrap();
    //     let prototype: v8::Local<v8::Object> = prototype.try_into().unwrap();
    //
    //
    //     let gtype_value = prototype.get(scope, gtype_name.into()).unwrap();
    //     let gtype_value: v8::Local<v8::BigInt> = gtype_value.try_into().unwrap();
    //     let (gtype_value, wrapped) = gtype_value.u64_value();
    //     assert!(!wrapped);
    //
    //     // no idea what i am doing here
    //     let gtype = gtype_value as GType;
    //     let gtype = unsafe { glib::Type::from_glib(gtype) };
    //
    //     gtype
    // };

    let ptr_value = {
        let ptr_name = v8::String::new(scope, POINTER_FIELD).unwrap();

        let ptr_value = widget.get(scope, ptr_name.into()).unwrap();
        let ptr_value: v8::Local<v8::External> = ptr_value.try_into().unwrap();
        let ptr_value = ptr_value.value();

        ptr_value
    };

    let widget = {
        // no idea what i am doing here
        let widget = ptr_value as *mut glib::gobject_ffi::GObject;
        let widget = unsafe { glib::Object::from_glib_none(widget) };

        widget
    };

    widget
}

fn from_gtk_widget_to_js<'scope>(
    scope: &mut v8::HandleScope<'scope>,
    widget: gtk::Widget,
) -> serde_v8::Value<'scope> {
    let result = v8::Object::new(scope);

    // {
    //     let gtype_name = v8::String::new(scope, GTYPE_FIELD).unwrap();
    //
    //     let gtype_value = widget.type_();
    //     let gtype_value = gtype_value.into_glib();
    //     debug_assert!(std::mem::size_of::<GType>() <= std::mem::size_of::<u64>());
    //     let gtype_value = v8::BigInt::new_from_u64(scope, gtype_value as u64);
    //
    //     let prototype_object = v8::Object::new(scope);
    //     prototype_object.set(scope, gtype_name.into(), gtype_value.into());
    //     assert!(result
    //         .set_prototype(scope, prototype_object.into())
    //         .unwrap());
    // }

    {
        let widget_ptr: *const gtk::ffi::GtkWidget = widget.to_glib_full();

        let ptr_name = v8::String::new(scope, POINTER_FIELD).unwrap();
        let ptr_value = v8::External::new(scope, widget_ptr as *mut c_void);

        let is_set = result.set(scope, ptr_name.into(), ptr_value.into());
        assert!(is_set.unwrap());
    }

    let result: v8::Local<v8::Value> = result.into();

    result.into()
}

fn run_js(root_container: gtk::Box) -> impl Future<Output=Result<(), AnyError>> {
    async {
        let js_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("react_renderer/dist/main.js");
        let main_module = ModuleSpecifier::from_file_path(js_path).unwrap();
        let mut worker = MainWorker::bootstrap_from_options(
            main_module.clone(),
            PermissionsContainer::allow_all(),
            WorkerOptions {
                module_loader: Rc::new(FsModuleLoader),
                extensions: vec![gtk_ext::init_ops(root_container)],
                ..Default::default()
            },
        );
        worker.execute_main_module(&main_module).await
    }
}

fn build_ui(app: &gtk::Application) {
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .deletable(false)
        .resizable(false)
        .decorated(false)
        .default_height(400)
        .default_width(650)
        .title("My GTK App")
        .build();

    let spacing = 12;
    let entry = gtk::Entry::builder()
        .margin_top(spacing)
        .margin_bottom(spacing)
        .margin_start(spacing)
        .margin_end(spacing)
        .build();

    let separator = gtk::Separator::new(gtk::Orientation::Horizontal);

    let string_list = gtk::StringList::new(&[
        "test1", "test2", "test3", "test4", "test5", "test5", "test5", "test5", "test5", "test5",
        "test5", "test5", "test5", "test5", "test5", "test5", "test5", "test5", "test5", "test5",
        "test5", "test5", "test5", "test5", "test5", "test5", "test5",
    ]);

    let selection = gtk::SingleSelection::new(Some(string_list));

    let factory = gtk::SignalListItemFactory::new();
    {
        factory.connect_setup(move |_, list_item| {
            let image = gtk::Image::from_file(Path::new("extension_icon.png"));
            let label = gtk::Label::builder().margin_start(6).build();

            let gtk_box = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .margin_top(6)
                .margin_bottom(6)
                .margin_start(6)
                .margin_end(6)
                .build();

            gtk_box.append(&image);
            gtk_box.append(&label);

            let list_item = list_item
                .downcast_ref::<gtk::ListItem>()
                .expect("Needs to be ListItem");
            list_item.set_child(Some(&gtk_box));

            list_item
                .property_expression("item")
                .chain_property::<gtk::StringObject>("string")
                .bind(&label, "label", gtk::Widget::NONE);
        });
    }

    let list_view = gtk::ListView::builder()
        .model(&selection)
        .factory(&factory)
        .show_separators(true)
        .build();

    let window_in_list_view_callback = window.clone();
    list_view.connect_activate(move |list_view, position| {
        // let model = list_view.model().expect("The model has to exist.");
        // let string_object = model
        //     .item(position)
        //     .and_downcast::<gtk::StringObject>()
        //     .expect("The item has to be an `StringObject`.");

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let gtk_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        window_in_list_view_callback.set_child(Some(&gtk_box));

        rt.block_on(run_js(gtk_box.clone())).unwrap();

        println!("test timeout")

        // println!("test {}", string_object.string());

        // let label = gtk::Label::builder()
        //     .margin_start(6)
        //     .label(string_object.string())
        //     .build();
        //
        // window_in_list_view_callback.set_child(Some(&label));
    });

    let scrolled_window = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&list_view)
        .vexpand(true)
        .margin_top(spacing)
        .margin_bottom(spacing)
        .margin_start(spacing)
        .margin_end(spacing)
        .build();

    let gtk_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    gtk_box.append(&entry);
    gtk_box.append(&separator);
    gtk_box.append(&scrolled_window);

    window.set_child(Some(&gtk_box));

    // // Before the window is first realized, set it up to be a layer surface
    // gtk_layer_shell::init_for_window(&window);
    //
    // // Order below normal windows
    // gtk_layer_shell::set_layer(&window, gtk_layer_shell::Layer::Overlay);
    //
    // // Push other windows out of the way
    // gtk_layer_shell::auto_exclusive_zone_enable(&window);
    //
    // // The margins are the gaps around the window's edges
    // // Margins and anchors can be set like this...
    // gtk_layer_shell::set_margin(&window, gtk_layer_shell::Edge::Left, 40);
    // gtk_layer_shell::set_margin(&window, gtk_layer_shell::Edge::Right, 40);
    // gtk_layer_shell::set_margin(&window, gtk_layer_shell::Edge::Top, 20);
    //
    // // ... or like this
    // // Anchors are if the window is pinned to each edge of the output
    // let anchors = [
    //     (gtk_layer_shell::Edge::Left, true),
    //     (gtk_layer_shell::Edge::Right, true),
    //     (gtk_layer_shell::Edge::Top, false),
    //     (gtk_layer_shell::Edge::Bottom, true),
    // ];
    //
    // for (anchor, state) in anchors {
    //     gtk_layer_shell::set_anchor(&window, anchor, state);
    // }

    window.present();
}
