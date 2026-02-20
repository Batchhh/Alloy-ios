use objc2::rc::{Allocated, Retained};
use objc2::{extern_class, msg_send, ClassType};
use objc2_foundation::{MainThreadMarker, NSObject};
use objc2_ui_kit::{UIResponder, UIView};

extern_class!(
    #[unsafe(super(NSObject))]
    #[name = "UIBlurEffect"]
    #[derive(Debug, PartialEq, Eq, Hash)]
    /// Wrapper for `UIBlurEffect`
    pub struct UIBlurEffect;
);

extern_class!(
    #[unsafe(super(UIView, UIResponder, NSObject))]
    #[name = "UIVisualEffectView"]
    #[derive(Debug, PartialEq, Eq, Hash)]
    /// Wrapper for `UIVisualEffectView`
    pub struct UIVisualEffectView;
);

extern_class!(
    #[unsafe(super(objc2_quartz_core::CALayer, NSObject))]
    #[name = "CAGradientLayer"]
    #[derive(Debug, PartialEq, Eq, Hash)]
    /// Wrapper for `CAGradientLayer`
    pub struct CAGradientLayer;
);

impl CAGradientLayer {
    /// Creates a new gradient layer
    pub fn new() -> Retained<Self> {
        unsafe { msg_send![Self::class(), layer] }
    }

    /// Sets the colors for the gradient
    pub fn setColors(&self, colors: &objc2_foundation::NSArray<NSObject>) {
        unsafe { msg_send![self, setColors: colors] }
    }
}

impl UIVisualEffectView {
    /// Creates a new visual effect view with the specified effect
    pub fn new(effect: &UIBlurEffect, _mtm: MainThreadMarker) -> Retained<Self> {
        unsafe {
            let view: Allocated<Self> = msg_send![UIVisualEffectView::class(), alloc];
            let view: Retained<Self> = msg_send![view, initWithEffect: effect];
            view
        }
    }

    /// Returns the content view where subviews should be added
    ///
    /// iOS requires adding subviews to contentView, not directly to UIVisualEffectView
    pub fn contentView(&self) -> Retained<UIView> {
        unsafe { msg_send![self, contentView] }
    }
}
