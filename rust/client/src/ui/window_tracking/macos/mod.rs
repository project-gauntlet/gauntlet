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

// only active space(s) and fullscreen apps (only single item per app will be shown) are supported
// tabs are supported only on active "desktop" space, not supported in fullscreen window
// show warning when:
//  there are non-active spaces
//    except fullscreen windows
//  there multiple fullscreen windows for specific app

// warning should say:
// gauntlet doesn't show windows on non-active spaces except fullscreen applications
// gauntlet doesn't support fullscreen applications on multiple spaces
// gauntlet doesn't support native tabs for fullscreen applications

// refresh window list when space switches
// if current space is fullscreen do not scan for tabs? show warning?

// CGSSpaceGetType to get type of given space
// CGSGetWindowWorkspace to get list of spaces for specific window
// ? to get space for given
// ? to get current space

//  todo what if there are 2 monitors. is it same space or multiple? what does "separate spaces" setting do?
//  todo what if gauntlet started on fullscreen space?

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
//   https://github.com/glide-wm/glide/issues/10


