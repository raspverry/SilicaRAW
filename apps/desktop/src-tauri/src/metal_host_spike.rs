use std::cell::Cell;
use std::error::Error;
use std::io;
use std::time::{Duration, Instant};

use objc2::rc::{PartialInit, Retained};
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{define_class, msg_send, DefinedClass, MainThreadOnly};
use objc2_app_kit::{NSAutoresizingMaskOptions, NSEvent, NSWindow};
use objc2_foundation::{MainThreadMarker, NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize};
use objc2_metal::{
    MTLClearColor, MTLCommandBuffer, MTLCommandEncoder, MTLCommandQueue,
    MTLCreateSystemDefaultDevice, MTLDevice, MTLDrawable, MTLPixelFormat,
};
use objc2_metal_kit::{MTKView, MTKViewDelegate};
use tauri::{LogicalSize, Manager};

type SpikeResult<T> = Result<T, Box<dyn Error>>;

static DELEGATE_ASSOCIATION_KEY: u8 = 0;

#[derive(Debug, Default)]
struct SpikeMetalViewIvars;

define_class!(
    #[unsafe(super = MTKView)]
    #[thread_kind = MainThreadOnly]
    #[ivars = SpikeMetalViewIvars]
    struct SpikeMetalView;

    unsafe impl NSObjectProtocol for SpikeMetalView {}

    impl SpikeMetalView {
        #[unsafe(method(acceptsFirstResponder))]
        fn accepts_first_responder(&self) -> bool {
            true
        }

        #[unsafe(method(mouseDown:))]
        fn mouse_down(&self, event: &NSEvent) {
            log_pointer_event("mouseDown", event);
            unsafe {
                let _: () = msg_send![super(self), mouseDown: event];
            }
        }

        #[unsafe(method(mouseDragged:))]
        fn mouse_dragged(&self, event: &NSEvent) {
            log_pointer_event("mouseDragged", event);
            unsafe {
                let _: () = msg_send![super(self), mouseDragged: event];
            }
        }

        #[unsafe(method(scrollWheel:))]
        fn scroll_wheel(&self, event: &NSEvent) {
            let point = event.locationInWindow();
            eprintln!(
                "[SilicaRAW Spike 001] scrollWheel at {:.1},{:.1} delta {:.2},{:.2} phase {:?} momentum {:?}",
                point.x,
                point.y,
                event.scrollingDeltaX(),
                event.scrollingDeltaY(),
                event.phase(),
                event.momentumPhase()
            );
            unsafe {
                let _: () = msg_send![super(self), scrollWheel: event];
            }
        }

        #[unsafe(method(magnifyWithEvent:))]
        fn magnify_with_event(&self, event: &NSEvent) {
            let point = event.locationInWindow();
            eprintln!(
                "[SilicaRAW Spike 001] magnifyWithEvent at {:.1},{:.1} magnification {:.4} phase {:?}",
                point.x,
                point.y,
                event.magnification(),
                event.phase()
            );
            unsafe {
                let _: () = msg_send![super(self), magnifyWithEvent: event];
            }
        }

        #[unsafe(method(smartMagnifyWithEvent:))]
        fn smart_magnify_with_event(&self, event: &NSEvent) {
            log_pointer_event("smartMagnifyWithEvent", event);
            unsafe {
                let _: () = msg_send![super(self), smartMagnifyWithEvent: event];
            }
        }

        #[unsafe(method(setFrameSize:))]
        fn set_frame_size(&self, new_size: NSSize) {
            unsafe {
                let _: () = msg_send![super(self), setFrameSize: new_size];
            }
            eprintln!(
                "[SilicaRAW Spike 001] MTKView frame resized to {:.0}x{:.0}pt",
                new_size.width, new_size.height
            );
        }

        #[unsafe(method(viewDidChangeBackingProperties))]
        fn view_did_change_backing_properties(&self) {
            unsafe {
                let _: () = msg_send![super(self), viewDidChangeBackingProperties];
            }
            eprintln!("[SilicaRAW Spike 001] MTKView backing properties changed");
        }
    }
);

impl SpikeMetalView {
    fn new_with_device(
        mtm: MainThreadMarker,
        frame_rect: NSRect,
        device: Option<&ProtocolObject<dyn MTLDevice>>,
    ) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(SpikeMetalViewIvars);
        unsafe { Self::init_with_frame_device(this, frame_rect, device) }
    }

    unsafe fn init_with_frame_device(
        this: PartialInit<Self>,
        frame_rect: NSRect,
        device: Option<&ProtocolObject<dyn MTLDevice>>,
    ) -> Retained<Self> {
        unsafe { msg_send![super(this), initWithFrame: frame_rect, device: device] }
    }
}

#[derive(Debug)]
struct SpikeMetalDelegateIvars {
    command_queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
    frames_drawn: Cell<u64>,
    started_at: Instant,
}

define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = SpikeMetalDelegateIvars]
    struct SpikeMetalDelegate;

    unsafe impl NSObjectProtocol for SpikeMetalDelegate {}

    unsafe impl MTKViewDelegate for SpikeMetalDelegate {
        #[unsafe(method(mtkView:drawableSizeWillChange:))]
        fn drawable_size_will_change(&self, _view: &MTKView, size: NSSize) {
            eprintln!(
                "[SilicaRAW Spike 001] MTKView drawable resized to {:.0}x{:.0}px",
                size.width, size.height
            );
        }

        #[unsafe(method(drawInMTKView:))]
        fn draw_in_mtk_view(&self, view: &MTKView) {
            let Some(render_pass_descriptor) = view.currentRenderPassDescriptor() else {
                return;
            };
            let Some(drawable) = view.currentDrawable() else {
                return;
            };
            let Some(command_buffer) = self.ivars().command_queue.commandBuffer() else {
                return;
            };
            let Some(encoder) =
                command_buffer.renderCommandEncoderWithDescriptor(&render_pass_descriptor)
            else {
                return;
            };

            encoder.endEncoding();

            let drawable: &ProtocolObject<dyn MTLDrawable> = ProtocolObject::from_ref(&*drawable);
            command_buffer.presentDrawable(drawable);
            command_buffer.commit();

            let frames_drawn = self.ivars().frames_drawn.get() + 1;
            self.ivars().frames_drawn.set(frames_drawn);

            if frames_drawn == 1 || frames_drawn % 120 == 0 {
                eprintln!(
                    "[SilicaRAW Spike 001] Render loop drew {frames_drawn} frame(s) over {:?}",
                    self.ivars().started_at.elapsed()
                );
            }
        }
    }
);

impl SpikeMetalDelegate {
    fn new(
        mtm: MainThreadMarker,
        command_queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
    ) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(SpikeMetalDelegateIvars {
            command_queue,
            frames_drawn: Cell::new(0),
            started_at: Instant::now(),
        });

        unsafe { msg_send![super(this), init] }
    }
}

pub fn install<R: tauri::Runtime>(app: &mut tauri::App<R>) -> SpikeResult<()> {
    let mtm = MainThreadMarker::new().ok_or_else(|| {
        spike_error("Spike 001 must install the MTKView on the macOS main thread")
    })?;
    let webview_window = app
        .get_webview_window("main")
        .ok_or_else(|| spike_error("main webview window was not found"))?;

    let ns_window_ptr = webview_window.ns_window()? as *mut NSWindow;
    let ns_window = unsafe { ns_window_ptr.as_ref() }
        .ok_or_else(|| spike_error("Tauri returned a null NSWindow pointer"))?;
    let content_view = ns_window
        .contentView()
        .ok_or_else(|| spike_error("NSWindow has no content view"))?;
    let metal_device = MTLCreateSystemDefaultDevice()
        .ok_or_else(|| spike_error("MTLCreateSystemDefaultDevice returned nil"))?;
    let command_queue = metal_device
        .newCommandQueue()
        .ok_or_else(|| spike_error("Metal device could not create a command queue"))?;

    let bounds = content_view.bounds();
    let frame = spike_view_frame(bounds);
    let scale_factor = ns_window.backingScaleFactor();

    let metal_view = {
        let view = SpikeMetalView::new_with_device(mtm, frame, Some(&metal_device));
        view.setColorPixelFormat(MTLPixelFormat::BGRA8Unorm);
        view.setClearColor(MTLClearColor {
            red: 0.05,
            green: 0.20,
            blue: 0.22,
            alpha: 1.0,
        });
        view.setAutoResizeDrawable(true);
        view.setDrawableSize(NSSize::new(
            frame.size.width * scale_factor,
            frame.size.height * scale_factor,
        ));
        view.setPreferredFramesPerSecond(60);
        view.setPaused(false);
        view.setEnableSetNeedsDisplay(false);
        view.setAutoresizingMask(
            NSAutoresizingMaskOptions::ViewMinXMargin
                | NSAutoresizingMaskOptions::ViewWidthSizable
                | NSAutoresizingMaskOptions::ViewHeightSizable,
        );
        view
    };

    let delegate = SpikeMetalDelegate::new(mtm, command_queue);
    metal_view.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));

    content_view.addSubview(&metal_view);
    unsafe { retain_delegate_for_view_lifetime(&metal_view, delegate) };

    eprintln!(
        "[SilicaRAW Spike 001] Installed MTKView frame {:.0}x{:.0}pt, drawable {:.0}x{:.0}px, backing scale {:.1}",
        frame.size.width,
        frame.size.height,
        frame.size.width * scale_factor,
        frame.size.height * scale_factor,
        scale_factor
    );

    maybe_schedule_resize_probe(&webview_window);

    Ok(())
}

fn spike_view_frame(bounds: NSRect) -> NSRect {
    let margin = 24.0;
    let width = (bounds.size.width * 0.36).clamp(280.0, 560.0);
    let height = (bounds.size.height * 0.52).clamp(220.0, 520.0);
    let x = (bounds.size.width - width - margin).max(margin);

    NSRect::new(NSPoint::new(x, margin), NSSize::new(width, height))
}

fn log_pointer_event(name: &str, event: &NSEvent) {
    let point = event.locationInWindow();
    eprintln!(
        "[SilicaRAW Spike 001] {name} at {:.1},{:.1} button {} clicks {} delta {:.2},{:.2} pressure {:.2}",
        point.x,
        point.y,
        event.buttonNumber(),
        event.clickCount(),
        event.deltaX(),
        event.deltaY(),
        event.pressure()
    );
}

fn maybe_schedule_resize_probe<R: tauri::Runtime>(webview_window: &tauri::WebviewWindow<R>) {
    if std::env::var_os("SILICA_SPIKE_AUTO_RESIZE").is_none() {
        return;
    }

    let window = webview_window.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(900));
        match window.set_size(LogicalSize::new(980.0, 720.0)) {
            Ok(()) => eprintln!("[SilicaRAW Spike 001] Requested automatic window resize probe"),
            Err(error) => {
                eprintln!("[SilicaRAW Spike 001] Automatic window resize probe failed: {error}")
            }
        }
    });
}

unsafe fn retain_delegate_for_view_lifetime(
    metal_view: &MTKView,
    delegate: Retained<SpikeMetalDelegate>,
) {
    let view_ptr = metal_view as *const MTKView as *mut AnyObject;
    let delegate_ptr = Retained::into_raw(delegate) as *mut AnyObject;
    let key = &DELEGATE_ASSOCIATION_KEY as *const u8 as *const _;

    unsafe {
        objc2::ffi::objc_setAssociatedObject(
            view_ptr,
            key,
            delegate_ptr,
            objc2::ffi::OBJC_ASSOCIATION_RETAIN_NONATOMIC,
        );
        let _ = Retained::from_raw(delegate_ptr as *mut SpikeMetalDelegate);
    }
}

fn spike_error(message: &str) -> Box<dyn Error> {
    Box::new(io::Error::other(message.to_owned()))
}
