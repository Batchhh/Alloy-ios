//! components creation
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, Class};
use objc2::{msg_send, sel, ClassType, MainThreadOnly};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_foundation::{MainThreadMarker, NSString};
use objc2_ui_kit::{
    UIButton, UIColor, UIControlEvents, UIControlState, UIFont, UILabel, UISlider, UITextField,
    UIView,
};

use crate::ui::theme::Theme;

/// Creates a UILabel with standard styling
///
/// # Arguments
/// * `frame` - The size and position of the label
/// * `text` - The text content
/// * `size` - Font size
/// * `bold` - Whether to use bold font
/// * `mtm` - Main thread marker
pub fn create_label(
    frame: CGRect,
    text: &str,
    size: f64,
    bold: bool,
    mtm: MainThreadMarker,
) -> Retained<UILabel> {
    let label: Retained<UILabel> = UILabel::initWithFrame(UILabel::alloc(mtm), frame);
    label.setText(Some(&NSString::from_str(text)));
    unsafe {
        label.setTextColor(Some(&Theme::text()));
        let font = if bold {
            UIFont::boldSystemFontOfSize(size)
        } else {
            UIFont::systemFontOfSize(size)
        };
        label.setFont(Some(&font));
    }
    label
}

/// Creates a section header label (smaller, formatted text)
pub fn create_section_header(
    frame: CGRect,
    text: &str,
    mtm: MainThreadMarker,
) -> Retained<UILabel> {
    let label: Retained<UILabel> = UILabel::initWithFrame(UILabel::alloc(mtm), frame);
    label.setText(Some(&NSString::from_str(text)));
    unsafe {
        label.setTextColor(Some(&Theme::text_secondary()));
        label.setFont(Some(&UIFont::boldSystemFontOfSize(13.0)));
        let layer = label.layer();
    }
    label
}

/// Creates a styled container button with shadow and rounded corners
///
/// Used as the background for list items.
pub fn styled_container(frame: CGRect, mtm: MainThreadMarker) -> Retained<UIButton> {
    let item: Retained<UIButton> = UIButton::initWithFrame(UIButton::alloc(mtm), frame);
    item.setBackgroundColor(Some(&Theme::container_background()));
    let layer = item.layer();
    layer.setCornerRadius(14.0);
    unsafe {
        layer.setCornerCurve(objc2_quartz_core::kCACornerCurveContinuous);
    }

    layer.setBorderWidth(0.5);
    unsafe {
        layer.setBorderColor(Some(&Theme::container_border().CGColor()));
        let shadow_color = Theme::shadow().CGColor();
        layer.setShadowColor(Some(&shadow_color));
        layer.setShadowOffset(CGSize::new(0.0, 2.0));
        layer.setShadowRadius(4.0);
        layer.setShadowOpacity(0.05);
    }

    item.setUserInteractionEnabled(true);

    item.setAdjustsImageWhenHighlighted(false);
    item
}

/// Creates a styled remote button (standard colored button)
///
/// # Arguments
/// * `frame` - Button frame
/// * `title` - Button title
/// * `background` - Background color
/// * `mtm` - Main thread marker
pub fn create_button(
    frame: CGRect,
    title: &str,
    background: Retained<UIColor>,
    mtm: MainThreadMarker,
) -> Retained<UIButton> {
    let button: Retained<UIButton> = UIButton::initWithFrame(UIButton::alloc(mtm), frame);
    button.setBackgroundColor(Some(&background));
    unsafe {
        button.setTitle_forState(Some(&NSString::from_str(title)), UIControlState::Normal);
        button.setTitleColor_forState(Some(&Theme::text()), UIControlState::Normal);
        if let Some(label) = button.titleLabel() {
            label.setFont(Some(&UIFont::systemFontOfSize(16.0)));
        }
    }
    button.setUserInteractionEnabled(true);
    button
}

/// Creates a UISlider with standard styling and custom thumb image
///
/// # Arguments
/// * `frame` - Slider frame
/// * `value` - Initial value
/// * `min` - Minimum value
/// * `max` - Maximum value
/// * `mtm` - Main thread marker
pub fn create_slider(
    frame: CGRect,
    value: f32,
    min: f32,
    max: f32,
    mtm: MainThreadMarker,
) -> Retained<UISlider> {
    let slider: Retained<UISlider> = UISlider::initWithFrame(UISlider::alloc(mtm), frame);
    slider.setMinimumValue(min);
    slider.setMaximumValue(max);
    slider.setValue(value);
    unsafe {
        slider.setTintColor(Some(&Theme::accent()));
        slider.setThumbTintColor(Some(&Theme::accent()));
        slider.setMaximumTrackTintColor(Some(&Theme::slider_track_inactive()));

        let image_cls = Class::get(c"UIImage").expect("UIImage class not found");
        let config_cls = Class::get(c"UIImageSymbolConfiguration")
            .expect("UIImageSymbolConfiguration class not found");

        // UIImageSymbolScaleSmall = 1
        let scale: i32 = 1;
        let config: Retained<AnyObject> = msg_send![config_cls, configurationWithScale: scale];

        let symbol_name = NSString::from_str("circle.fill");
        let image: Retained<AnyObject> =
            msg_send![image_cls, systemImageNamed: &*symbol_name, withConfiguration: &*config];

        let _: () = msg_send![&slider, setThumbImage: &*image, forState: 0_usize]; // Normal
        let _: () = msg_send![&slider, setThumbImage: &*image, forState: 1_usize];
    }
    slider.setContinuous(true);
    slider.setUserInteractionEnabled(true);
    slider
}

/// Creates a UITextField with standard styling, padding, and clear button
///
/// # Arguments
/// * `frame` - Text field frame
/// * `placeholder` - Placeholder text
/// * `mtm` - Main thread marker
pub fn create_text_input(
    frame: CGRect,
    placeholder: &str,
    mtm: MainThreadMarker,
) -> Retained<UITextField> {
    let input: Retained<UITextField> = UITextField::initWithFrame(UITextField::alloc(mtm), frame);
    input.setBackgroundColor(Some(&Theme::input_background()));
    let layer = input.layer();
    layer.setCornerRadius(10.0);
    unsafe {
        layer.setCornerCurve(objc2_quartz_core::kCACornerCurveContinuous);
    }
    input.setPlaceholder(Some(&NSString::from_str(placeholder)));

    unsafe {
        input.setTextColor(Some(&Theme::text()));
    }

    let padding = UIView::initWithFrame(
        UIView::alloc(mtm),
        CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(12.0, 10.0)),
    );
    input.setLeftView(Some(&padding));
    input.setLeftViewMode(objc2_ui_kit::UITextFieldViewMode::Always);

    input.setLeftView(Some(&padding));
    input.setLeftViewMode(objc2_ui_kit::UITextFieldViewMode::Always);

    let clear_btn = UIButton::initWithFrame(
        UIButton::alloc(mtm),
        CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(20.0, 20.0)),
    );
    let image_cls = Class::get(c"UIImage").expect("UIImage class not found");
    let symbol_name = NSString::from_str("xmark.circle.fill");
    let image: Retained<AnyObject> =
        unsafe { msg_send![image_cls, systemImageNamed: &*symbol_name] };

    unsafe {
        let _: () = msg_send![&clear_btn, setImage: &*image forState: 0_usize];
        clear_btn.setTintColor(Some(&Theme::text()));
    }

    let delegate = crate::ui::utils::delegate::TextFieldDelegate::shared(mtm);
    unsafe {
        clear_btn.addTarget_action_forControlEvents(
            Some(&delegate),
            sel!(clearText:),
            UIControlEvents::TouchUpInside,
        );
    }

    let container = UIView::initWithFrame(
        UIView::alloc(mtm),
        CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(23.0, 20.0)),
    );
    container.addSubview(&clear_btn);

    input.setRightView(Some(&container));
    input.setRightViewMode(objc2_ui_kit::UITextFieldViewMode::WhileEditing);

    unsafe {
        use objc2::msg_send;
        let _: () = msg_send![&input, setReturnKeyType: 9_isize]; // UIReturnKeyDone = 9
    }

    input
}
