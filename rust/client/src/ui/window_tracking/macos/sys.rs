use accessibility::AXUIElement;
use accessibility_sys::AXError;
use accessibility_sys::AXObserverGetTypeID;
use accessibility_sys::AXObserverRef;
use accessibility_sys::AXUIElementRef;
use accessibility_sys::kAXErrorSuccess;
use accessibility_sys::pid_t;
use anyhow::Context;
use anyhow::anyhow;
use core_foundation::base::OSStatus;
use core_foundation::base::TCFType;
use core_foundation::declare_TCFType;
use core_foundation::impl_CFTypeDescription;
use core_foundation::impl_TCFType;
use objc2_core_graphics::CGError;
use objc2_core_graphics::CGWindowID;

declare_TCFType!(AXObserver, AXObserverRef);
impl_TCFType!(AXObserver, AXObserverRef, AXObserverGetTypeID);
impl_CFTypeDescription!(AXObserver);

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn _AXUIElementGetWindow(element: AXUIElementRef, out: *mut CGWindowID) -> AXError;
    fn GetProcessForPID(pid: pid_t, psn: *mut ProcessSerialNumber) -> OSStatus;
}

#[repr(C)]
#[derive(Default)]
struct ProcessSerialNumber {
    high: u32,
    low: u32,
}

impl ProcessSerialNumber {
    fn for_pid(pid: pid_t) -> anyhow::Result<Self> {
        let mut psn = ProcessSerialNumber::default();
        if unsafe { GetProcessForPID(pid, &mut psn) } == 0 {
            Ok(psn)
        } else {
            Err(anyhow!("Failed to get process serial number for pid: {}", pid))
        }
    }
}

pub fn make_key_window(pid: pid_t, window: &AXUIElement) -> anyhow::Result<()> {
    // See https://github.com/Hammerspoon/hammerspoon/issues/370#issuecomment-545545468.
    // god bless all the wizards in that thread that worked on it, thank you

    let mut window_id = 0;
    let res = unsafe { _AXUIElementGetWindow(window.as_concrete_TypeRef(), &mut window_id) };
    if res != kAXErrorSuccess {
        return Err(accessibility::Error::Ax(res)).context(format!("Failed to get window id for element {:?}", window));
    }

    #[allow(non_upper_case_globals)]
    const kCPSUserGenerated: u32 = 0x200;

    let mut event1 = [0; 0x100];
    event1[0x04] = 0xf8;
    event1[0x08] = 0x01;
    event1[0x3a] = 0x10;
    event1[0x3c..0x3c + 4].copy_from_slice(&window_id.to_le_bytes());
    event1[0x20..(0x20 + 0x10)].fill(0xff);

    let mut event2 = event1.clone();
    event2[0x08] = 0x02;

    let psn = ProcessSerialNumber::for_pid(pid)?;

    let check = |err| {
        if err == CGError::Success {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Cursed api failed: {:?}", err))
        }
    };
    unsafe {
        check(_SLPSSetFrontProcessWithOptions(&psn, window_id, kCPSUserGenerated))?;
        check(SLPSPostEventRecordTo(&psn, event1.as_ptr()))?;
        check(SLPSPostEventRecordTo(&psn, event2.as_ptr()))?;
    }
    Ok(())
}

#[link(name = "SkyLight", kind = "framework")] // PrivateFrameworks included in build.rs
unsafe extern "C" {
    fn _SLPSSetFrontProcessWithOptions(psn: *const ProcessSerialNumber, wid: u32, mode: u32) -> CGError;
    fn SLPSPostEventRecordTo(psn: *const ProcessSerialNumber, bytes: *const u8) -> CGError;
}
