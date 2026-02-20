use objc2::rc::Retained;
use objc2::{extern_class, msg_send, ClassType};
use objc2_foundation::NSObject;

extern_class!(
    #[unsafe(super(NSObject))]
    #[name = "UISelectionFeedbackGenerator"]
    #[derive(Debug, PartialEq, Eq, Hash)]
    /// Wrapper for `UISelectionFeedbackGenerator` to provide haptic feedback
    pub struct UISelectionFeedbackGenerator;
);

impl UISelectionFeedbackGenerator {
    pub fn new() -> Retained<Self> {
        unsafe { msg_send![Self::class(), new] }
    }

    /// Prepares the generator for use (lowers latency)
    pub fn prepare(&self) {
        unsafe {
            let _: () = msg_send![self, prepare];
        }
    }

    /// Triggers the feedback (selection change haptic)
    pub fn selectionChanged(&self) {
        unsafe {
            let _: () = msg_send![self, selectionChanged];
        }
    }
}

unsafe impl Send for UISelectionFeedbackGenerator {}
unsafe impl Sync for UISelectionFeedbackGenerator {}
