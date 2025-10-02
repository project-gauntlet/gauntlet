use accessibility::AXUIElement;
use accessibility::AXUIElementAttributes;
use accessibility_sys::AXError;
use accessibility_sys::AXObserverGetTypeID;
use accessibility_sys::AXObserverRef;
use accessibility_sys::AXUIElementRef;
use accessibility_sys::kAXErrorSuccess;
use accessibility_sys::kAXWindowRole;
use accessibility_sys::pid_t;
use anyhow::Context;
use anyhow::anyhow;
use core_foundation::base::CFIndexConvertible;
use core_foundation::base::OSStatus;
use core_foundation::base::TCFType;
use core_foundation::data::CFData;
use core_foundation::data::CFDataCreate;
use core_foundation::data::CFDataRef;
use core_foundation::declare_TCFType;
use core_foundation::impl_CFTypeDescription;
use core_foundation::impl_TCFType;
use core_graphics::base::kCGErrorSuccess;
use core_graphics::display::CGError;
use core_graphics::window::CGWindowID;

declare_TCFType!(AXObserver, AXObserverRef);
impl_TCFType!(AXObserver, AXObserverRef, AXObserverGetTypeID);
impl_CFTypeDescription!(AXObserver);

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn _AXUIElementGetWindow(element: AXUIElementRef, out: *mut CGWindowID) -> AXError;
    fn GetProcessForPID(pid: pid_t, psn: *mut ProcessSerialNumber) -> OSStatus;
}

#[link(name = "SkyLight", kind = "framework")] // PrivateFrameworks included in build.rs
unsafe extern "C" {
    fn _SLPSSetFrontProcessWithOptions(psn: *const ProcessSerialNumber, wid: CGWindowID, mode: u32) -> CGError;
    fn SLPSPostEventRecordTo(psn: *const ProcessSerialNumber, bytes: *const u8) -> CGError;
    pub fn _AXUIElementCreateWithRemoteToken(data: CFDataRef) -> AXUIElementRef;
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

pub fn make_key_window(pid: pid_t, window_id: CGWindowID) -> anyhow::Result<()> {
    // See https://github.com/Hammerspoon/hammerspoon/issues/370#issuecomment-545545468.
    // god bless all the wizards in that thread that worked on it, thank you

    println!("window id: {}", window_id);

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
        if err == kCGErrorSuccess {
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

#[allow(unused)]
pub fn bruteforce_windows_for_app(app_pid: pid_t) -> Vec<AXUIElement> {
    // this whole thing can take more than a second, do not run on the main thread
    unsafe {
        let mut result = vec![];
        let mut data = [0; 0x14];

        let app_pid = u32::to_ne_bytes(app_pid as u32);
        data[0..0x4].copy_from_slice(&app_pid);

        let magic = u32::to_ne_bytes(0x636f636f);
        data[0x8..0xC].copy_from_slice(&magic);

        for element_id in 0..0x7fffu64 {
            let mut data = data.clone();

            let element_id = element_id.to_ne_bytes();
            data[0xC..0x14].copy_from_slice(&element_id);

            let data_ref = CFDataCreate(std::ptr::null(), data.as_ptr(), data.len().to_CFIndex());
            let data_ref = CFData::wrap_under_create_rule(data_ref);
            let data_ref = data_ref.as_concrete_TypeRef();

            let window = AXUIElement::wrap_under_create_rule(_AXUIElementCreateWithRemoteToken(data_ref));

            let role = window.role().ok().map(|role| role.to_string());
            if role.as_deref() != Some(kAXWindowRole) {
                continue;
            }

            result.push(window);
        }

        result
    }
}

pub fn ax_window_id(window: &AXUIElement) -> anyhow::Result<CGWindowID> {
    let mut window_id = 0;
    let res = unsafe { _AXUIElementGetWindow(window.as_concrete_TypeRef(), &mut window_id) };
    if res != kAXErrorSuccess {
        return Err(accessibility::Error::Ax(res)).context(format!("Failed to get window id for element {:?}", window));
    }
    Ok(window_id)
}
