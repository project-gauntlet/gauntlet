mod apps;
mod sys;
mod window;

use accessibility_sys::AXIsProcessTrustedWithOptions;
use accessibility_sys::kAXTrustedCheckOptionPrompt;
pub use apps::MacosWindowTracker;
pub use apps::setup_macos_window_tracker;
use core_foundation::base::TCFType;
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::CFString;

// see https://github.com/jkelleyrtp/kauresel/tree/master for a massive collection of related links

pub fn request_macos_accessibility_permissions() -> bool {
    let options = CFDictionary::from_CFType_pairs(&[(
        unsafe { CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt) },
        CFBoolean::from(true),
    )]);

    unsafe { AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()) }
}

// todo support for windows on separate spaces
// todo sometimes the window state seems to be lost, clearing the list of windows
// todo when starting the gauntlet, tabbed windows only show single one
// todo implement this https://github.com/glide-wm/glide/issues/10
// todo how to handle system apps and settings wrt window tracking?
