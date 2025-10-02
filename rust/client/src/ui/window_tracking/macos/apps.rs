use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use accessibility::AXUIElement;
use accessibility_sys::kAXRaiseAction;
use accessibility_sys::pid_t;
use anyhow::Context;
use core_foundation::base::CFType;
use core_foundation::base::FromVoid;
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::CFString;
use core_graphics::display::CGDisplay;
use core_graphics::window::kCGWindowListExcludeDesktopElements;
use gauntlet_server::plugins::ApplicationManager;
use objc2::AnyThread;
use objc2::DefinedClass;
use objc2::define_class;
use objc2::msg_send;
use objc2::rc::Retained;
use objc2::runtime::NSObject;
use objc2::sel;
use objc2_app_kit::NSApplicationActivationPolicy;
use objc2_app_kit::NSRunningApplication;
use objc2_app_kit::NSWorkspace;
use objc2_app_kit::NSWorkspaceApplicationKey;
use objc2_app_kit::NSWorkspaceDidLaunchApplicationNotification;
use objc2_app_kit::NSWorkspaceDidTerminateApplicationNotification;
use objc2_foundation::NSNotification;

use super::window::{WindowNotificationDelegate, WindowType};
use crate::ui::window_tracking::macos::sys::ax_window_id;
use crate::ui::window_tracking::macos::sys::make_key_window;

pub struct MacosWindowTracker {
    delegate: Retained<ApplicationNotificationDelegate>,
}

impl MacosWindowTracker {
    pub fn focus_window(&self, window_uuid: String) {
        if let Err(err) = self.delegate.focus_window(window_uuid) {
            tracing::warn!("Unable to focus window: {:#}", err);
        }
    }
}

pub fn setup_macos_window_tracker(application_manager: Arc<ApplicationManager>) -> MacosWindowTracker {
    let delegate = ApplicationNotificationDelegate::new(application_manager);

    let shared_workspace = unsafe { NSWorkspace::sharedWorkspace() };
    let notification_center = unsafe { shared_workspace.notificationCenter() };

    unsafe {
        notification_center.addObserver_selector_name_object(
            &delegate,
            sel!(applicationDidLaunch:),
            Some(NSWorkspaceDidLaunchApplicationNotification),
            None,
        )
    }
    unsafe {
        notification_center.addObserver_selector_name_object(
            &delegate,
            sel!(applicationDidTerminate:),
            Some(NSWorkspaceDidTerminateApplicationNotification),
            None,
        )
    }

    let running_applications = unsafe { NSWorkspace::sharedWorkspace().runningApplications() };

    let running_applications: Vec<_> = running_applications
        .into_iter()
        .filter(|application| unsafe { application.activationPolicy() } == NSApplicationActivationPolicy::Regular)
        .map(|app| unsafe { app.processIdentifier() })
        .collect();

    for pid in running_applications {
        if let Err(err) = delegate.create_window_notification_delegate(pid) {
            tracing::warn!("Error creating window notification delegate: {}", err);
        }
    }

    MacosWindowTracker { delegate }
}

impl ApplicationNotificationDelegate {
    fn new(application_manager: Arc<ApplicationManager>) -> Retained<Self> {
        let state = ApplicationNotificationDelegateState {
            application_manager,
            applications: RefCell::new(HashMap::new()),
        };

        let delegate = ApplicationNotificationDelegate::alloc().set_ivars(state);
        unsafe { msg_send![super(delegate), init] }
    }

    fn application_pid(&self, notification: &NSNotification) -> Option<pid_t> {
        let user_info = unsafe { notification.userInfo() };
        let Some(user_info) = user_info else {
            return None;
        };

        let application = unsafe { user_info.valueForKey(NSWorkspaceApplicationKey) };
        let Some(application) = application else {
            return None;
        };

        let Some(application) = application.downcast_ref::<NSRunningApplication>() else {
            return None;
        };

        if unsafe { application.activationPolicy() } != NSApplicationActivationPolicy::Regular {
            return None;
        }

        let pid = unsafe { application.processIdentifier() };

        Some(pid)
    }

    fn create_window_notification_delegate(&self, pid: pid_t) -> anyhow::Result<()> {
        let application_manager = self.ivars().application_manager.clone();

        let delegate = WindowNotificationDelegate::new(pid, application_manager)
            .context("Error creating window notification delegate")?;

        delegate
            .start()
            .context("Error starting window notification delegate")?;

        let mut applications = self.ivars().applications.borrow_mut();

        applications.insert(pid, delegate);

        Ok(())
    }

    fn destroy_window_notification_delegate(&self, pid: pid_t) {
        let mut applications = self.ivars().applications.borrow_mut();

        let Some(delegate) = applications.remove(&pid) else {
            tracing::debug!("No delegate for pid: {}", pid);
            return;
        };

        delegate.stop();
    }

    fn focus_window(&self, window_uuid: String) -> anyhow::Result<()> {
        let windows = self.ivars().windows.borrow();

        let Some((_, pid, window)) = windows.iter().find(|(uuid, _, _)| uuid == &window_uuid) else {
            return Err(anyhow::anyhow!("No window with uuid: {}", window_uuid));
        };

        println!("Focusing window: {}, {:?}, {}", window_uuid, window, pid);

        if let Some(windows) = CGDisplay::window_list_info(kCGWindowListExcludeDesktopElements, None) {
            for item in windows.into_iter() {
                let item: CFDictionary<CFString, CFType> = unsafe { CFDictionary::from_void(item.clone()) }.clone();
                println!("CFDictionary: {:?}", item);
            }
        };

        let window_id = ax_window_id(window).context("Failed to get window id")?;

        make_key_window(*pid, window_id).context("Failed to make window key")?;

        // some apps seem to also require additional raise action
        window
            .perform_action(&CFString::new(kAXRaiseAction))
            .context("Failed to raise window")?;

        Ok(())
    }
}

struct ApplicationNotificationDelegateState {
    application_manager: Arc<ApplicationManager>,
    applications: RefCell<HashMap<pid_t, WindowNotificationDelegate>>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[ivars = ApplicationNotificationDelegateState]
    struct ApplicationNotificationDelegate;

    impl ApplicationNotificationDelegate {
        #[unsafe(method(applicationDidLaunch:))]
        fn application_did_launch(&self, notification: &NSNotification) {
            tracing::debug!("Application did launch");

            let Some(pid) = self.application_pid(notification) else {
                return;
            };

            if let Err(err) = self.create_window_notification_delegate(pid) {
                tracing::warn!("Error creating window notification delegate: {}", err);
            }
        }

        #[unsafe(method(applicationDidTerminate:))]
        fn application_did_terminate(&self, notification: &NSNotification) {
            tracing::debug!("Application did terminate");

            let Some(pid) = self.application_pid(notification) else {
                return;
            };

            self.destroy_window_notification_delegate(pid)
        }
    }
);
