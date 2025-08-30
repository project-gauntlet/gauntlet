use std::cell::RefCell;
use std::ffi::c_void;
use std::mem::MaybeUninit;
use std::rc::Rc;
use std::sync::Arc;

use accessibility::AXUIElement;
use accessibility::AXUIElementAttributes;
use accessibility::Error;
use accessibility_sys::AXObserverAddNotification;
use accessibility_sys::AXObserverCreate;
use accessibility_sys::AXObserverGetRunLoopSource;
use accessibility_sys::AXObserverRef;
use accessibility_sys::AXUIElementRef;
use accessibility_sys::kAXDialogSubrole;
use accessibility_sys::kAXErrorSuccess;
use accessibility_sys::kAXStandardWindowSubrole;
use accessibility_sys::kAXTitleChangedNotification;
use accessibility_sys::kAXUIElementDestroyedNotification;
use accessibility_sys::kAXWindowCreatedNotification;
use accessibility_sys::kAXWindowRole;
use accessibility_sys::pid_t;
use anyhow::Context;
use core_foundation::base::TCFType;
use core_foundation::runloop::CFRunLoop;
use core_foundation::runloop::kCFRunLoopDefaultMode;
use core_foundation::string::CFString;
use core_foundation::string::CFStringRef;
use gauntlet_common::model::MacosWindowTrackingEvent;
use gauntlet_server::plugins::ApplicationManager;
use objc2_app_kit::NSRunningApplication;
use uuid::Uuid;

use super::sys::AXObserver;
use super::sys::bruteforce_windows_for_app;

pub struct WindowNotificationDelegate {
    app_element: AXUIElement,
    observer: AXObserver,
    inner: Rc<WindowNotificationDelegateInner>,
}

struct WindowNotificationDelegateInner {
    app_pid: pid_t,
    windows: Rc<RefCell<Vec<(String, pid_t, AXUIElement)>>>,
    application_manager: Arc<ApplicationManager>,
}

const WINDOW_EVENTS: [&str; 3] = [
    kAXWindowCreatedNotification,
    kAXUIElementDestroyedNotification,
    kAXTitleChangedNotification,
];

const MESSAGING_TIMEOUT_SEC: f32 = 1.0;

impl WindowNotificationDelegate {
    pub fn new(
        pid: pid_t,
        application_manager: Arc<ApplicationManager>,
        windows: Rc<RefCell<Vec<(String, pid_t, AXUIElement)>>>,
    ) -> anyhow::Result<Self> {
        let observer = unsafe {
            let mut result = MaybeUninit::uninit();

            let err = AXObserverCreate(pid, Self::ax_observer_callback, result.as_mut_ptr());

            if err != kAXErrorSuccess {
                return Err(Error::Ax(err)).context("Failed to create AXObserver");
            }

            AXObserver::wrap_under_create_rule(result.assume_init())
        };

        let app_element = {
            let element = AXUIElement::application(pid);

            element
                .set_messaging_timeout(MESSAGING_TIMEOUT_SEC)
                .context("Failed to set messaging timeout")?;

            element
        };

        Ok(Self {
            app_element,
            observer,
            inner: Rc::new(WindowNotificationDelegateInner {
                app_pid: pid,
                windows,
                application_manager,
            }),
        })
    }

    pub fn start(&self) -> anyhow::Result<()> {
        for event in WINDOW_EVENTS {
            unsafe {
                // SAFETY: the user_data must not be movable,
                // if it is moved, various os errors happen
                // when we try to dereference the data inside the observer callback
                // also callback must be called on the same thread
                let user_data = Box::into_raw(Box::new(self.inner.clone()));

                let err = AXObserverAddNotification(
                    self.observer.as_concrete_TypeRef(),
                    self.app_element.as_concrete_TypeRef(),
                    CFString::from_static_string(event).as_concrete_TypeRef(),
                    user_data as _,
                );

                if err != kAXErrorSuccess {
                    return Err(Error::Ax(err)).context(format!("Failed to add notification to AXObserver: {}", event));
                }
            }
        }

        let run_loop = CFRunLoop::get_current();
        unsafe {
            let source = TCFType::wrap_under_get_rule(AXObserverGetRunLoopSource(self.observer.as_concrete_TypeRef()));

            run_loop.add_source(&source, kCFRunLoopDefaultMode);
        }

        if let Err(err) = self.inner.refresh_windows() {
            tracing::warn!("Failed to init macos windows: {}", err);
        }

        Ok(())
    }

    pub fn stop(&self) {
        // no need to remove notifications here because
        // we call this after the application has already terminated

        let run_loop = CFRunLoop::get_current();
        unsafe {
            let source = TCFType::wrap_under_get_rule(AXObserverGetRunLoopSource(self.observer.as_concrete_TypeRef()));

            run_loop.remove_source(&source, kCFRunLoopDefaultMode);
        }

        let windows = self.inner.windows.borrow();

        for (window_id, _, _) in windows.iter() {
            let event = MacosWindowTrackingEvent::WindowClosed {
                window_id: window_id.clone(),
            };

            self.inner.application_manager.send_macos_window_tracking_event(event);
        }
    }

    unsafe extern "C" fn ax_observer_callback(
        _observer: AXObserverRef,
        element: AXUIElementRef,
        notification: CFStringRef,
        user_data: *mut c_void,
    ) {
        let delegate = unsafe { &*(user_data as *const Rc<WindowNotificationDelegateInner>) };

        let notification = unsafe { CFString::wrap_under_get_rule(notification) }.to_string();
        let element = unsafe { AXUIElement::wrap_under_get_rule(element) };

        tracing::debug!("Macos window accessibility notification: {}", notification);

        #[allow(non_upper_case_globals)]
        match notification.as_str() {
            kAXUIElementDestroyedNotification | kAXWindowCreatedNotification => {
                if let Err(err) = delegate.refresh_windows() {
                    tracing::warn!("Failed to refresh windows: {}", err);
                }
            }
            kAXTitleChangedNotification => {
                delegate.title_changed(element);
            }
            _ => {}
        }
    }
}

fn get_bundle_path(pid: pid_t) -> Option<String> {
    let app = unsafe { NSRunningApplication::runningApplicationWithProcessIdentifier(pid) };
    let Some(app) = app else {
        return None;
    };

    let bundle_path = unsafe { app.bundleURL() };

    let Some(bundle_path) = bundle_path else {
        return None;
    };

    let bundle_path = unsafe { bundle_path.path() };

    let Some(bundle_path) = bundle_path else {
        return None;
    };

    let bundle_path = bundle_path.to_string();

    Some(bundle_path)
}

impl WindowNotificationDelegateInner {
    fn refresh_windows(&self) -> anyhow::Result<()> {
        tracing::debug!("Refreshing windows for app: {}", self.app_pid);
        let mut retrieved_windows: Vec<_> = AXUIElement::application(self.app_pid)
            .windows()?
            .into_iter()
            .map(|item| item.clone())
            .collect();

        for window in bruteforce_windows_for_app(self.app_pid) {
            if !retrieved_windows.contains(&window) {
                retrieved_windows.push(window);
            };
        }

        tracing::debug!("Retrieved {} windows", retrieved_windows.len());

        let stored_windows = self
            .windows
            .borrow()
            .iter()
            .map(|(_, _, window)| window.clone())
            .collect::<Vec<_>>();

        tracing::debug!("Stored {} windows", stored_windows.len());

        for window in stored_windows.into_iter() {
            let Some(index) = retrieved_windows.iter().position(|el| el == &window) else {
                // doesn't exist anymore, destroy it
                self.window_destroyed(window);
                continue;
            };

            retrieved_windows.swap_remove(index);
        }

        tracing::debug!("left retrieved {} windows", retrieved_windows.len());

        for window in retrieved_windows.into_iter() {
            self.window_opened(window);
        }

        Ok(())
    }

    fn window_opened(&self, window: AXUIElement) {
        let mut windows = self.windows.borrow_mut();

        let role = window.role().ok().map(|role| role.to_string());
        let subrole = window.subrole().ok().map(|subrole| subrole.to_string());

        // ignore non-regular windows
        if role.as_deref() != Some(kAXWindowRole) {
            return;
        }

        // standard but minimized windows can have dialog subrole instead of standard window subrole
        if subrole.as_deref() != Some(kAXStandardWindowSubrole) && subrole.as_deref() != Some(kAXDialogSubrole) {
            return;
        }

        let window_id = Uuid::new_v4().to_string();
        windows.push((window_id.clone(), self.app_pid, window.clone()));

        let title = window.title().map(|title| title.to_string()).ok();
        let bundle_path = get_bundle_path(self.app_pid);

        let event = MacosWindowTrackingEvent::WindowOpened {
            window_id,
            bundle_path,
            title,
        };

        self.application_manager.send_macos_window_tracking_event(event);
    }

    fn window_destroyed(&self, element: AXUIElement) {
        let mut windows = self.windows.borrow_mut();

        let Some(index) = windows.iter().position(|(_, _, el)| el == &element) else {
            return;
        };

        let (window_id, _, _) = windows.swap_remove(index);

        let event = MacosWindowTrackingEvent::WindowClosed { window_id };

        self.application_manager.send_macos_window_tracking_event(event);
    }

    fn title_changed(&self, element: AXUIElement) {
        let windows = self.windows.borrow();

        let Some((window_id, _, _)) = windows.iter().find(|(_, _, el)| el == &element) else {
            return;
        };

        let title = element.title().map(|title| title.to_string()).ok();

        let event = MacosWindowTrackingEvent::WindowTitleChanged {
            window_id: window_id.clone(),
            title,
        };

        self.application_manager.send_macos_window_tracking_event(event);
    }
}
