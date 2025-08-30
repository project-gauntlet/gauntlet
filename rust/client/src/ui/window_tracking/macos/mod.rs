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


pub fn request_macos_accessibility_permissions() -> bool {
    let options = CFDictionary::from_CFType_pairs(&[(
        unsafe { CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt) },
        CFBoolean::from(true),
    )]);

    unsafe { AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()) }
}

// todo
//  on each ax notification + non-ax destroy event
//  get app pid using AXUIElementGetPid
//  run bruteforce search (+ regular) for axuielement using _AXUIElementCreateWithRemoteToken
//  from each found window axuielement
//    filter based on window role/subrole
//    get window using _AXUIElementGetWindow
//    get title
//  to focus use window id and private api


// do not support hidden windows
// do not support multiple "desktop" spaces unless they are all visible
// what about multiple fullscreen windows of the same app?
// what about tabs in visible apps in non-visible spaces?
// i.e., only non-hidden windows in visible spaces and maybe(?) fullscreen apps
// support minimized windows but only on visible spaces

// support fullscreen applications?
// support tabs on the visible windows in visible spaces only

// I think ignoring existence of spaces is fine???

// is the private function for focusing a window needed?

// todo support for windows on separate spaces
// todo multiple desktop spaces ("Desktop" vs. "Desktop 1" and "Desktop 2")
// todo sometimes the window state seems to be lost, clearing the list of windows
// todo when starting the gauntlet, tabbed windows only show single one
// todo implement this https://github.com/glide-wm/glide/issues/10
// todo how to handle system apps and settings wrt window tracking?
// todo add all github issue links and appreciations to the commit message
//   https://github.com/Hammerspoon/hammerspoon/issues/370#issuecomment-545545468
//   https://github.com/lwouis/alt-tab-macos/issues/1540#issuecomment-1138579049
//   https://github.com/lwouis/alt-tab-macos/issues/3589#issuecomment-2422002210
//   https://github.com/lwouis/alt-tab-macos/issues/258#issuecomment-624753940
//   https://github.com/jkelleyrtp/kauresel/tree/master
//   https://github.com/koekeishiya/yabai/issues/68
//   https://github.com/koekeishiya/yabai/commit/6f9006dd957100ec13096d187a8865e85a164a9b#r148091577
//   https://github.com/koekeishiya/yabai/issues/68
//   https://github.com/koekeishiya/yabai/issues/199#issuecomment-519152388
//   https://github.com/lwouis/alt-tab-macos/issues/1324#issuecomment-2631035482


