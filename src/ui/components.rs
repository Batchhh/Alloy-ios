use objc2::rc::{Allocated, Retained};
use objc2::{extern_class, msg_send, ClassType, MainThreadOnly};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_foundation::{MainThreadMarker, NSObject, NSString};
use objc2_ui_kit::{UIButton, UIColor, UIControlState, UIFont, UILabel, UIResponder, UISlider, UITextField, UIView};
use super::theme::Theme;

extern_class!(
    #[unsafe(super(NSObject))]
    #[name = "UIBlurEffect"]
    #[derive(Debug, PartialEq, Eq, Hash)]
    pub struct UIBlurEffect;
);

extern_class!(
    #[unsafe(super(UIView, UIResponder, NSObject))]
    #[name = "UIVisualEffectView"]
    #[derive(Debug, PartialEq, Eq, Hash)]
    pub struct UIVisualEffectView;
);

impl UIVisualEffectView {
    pub fn new(effect: &UIBlurEffect, _mtm: MainThreadMarker) -> Retained<Self> {
        unsafe {
            let view: Allocated<Self> = msg_send![UIVisualEffectView::class(), alloc];
            let view: Retained<Self> = msg_send![view, initWithEffect: effect];
            view
        }
    }
}

pub fn create_label(frame: CGRect, text: &str, size: f64, bold: bool, mtm: MainThreadMarker) -> Retained<UILabel> {
    let label: Retained<UILabel> = UILabel::initWithFrame(UILabel::alloc(mtm), frame);
    label.setText(Some(&NSString::from_str(text)));
    unsafe {
        label.setTextColor(Some(&Theme::text()));
        let font = if bold { UIFont::boldSystemFontOfSize(size) } else { UIFont::systemFontOfSize(size) };
        label.setFont(Some(&font));
    }
    label
}

fn styled_container(frame: CGRect, mtm: MainThreadMarker) -> Retained<UIButton> {
    let item: Retained<UIButton> = UIButton::initWithFrame(UIButton::alloc(mtm), frame);
    item.setBackgroundColor(Some(&Theme::container_background()));
    let layer = item.layer();
    layer.setCornerRadius(8.0);
    layer.setBorderWidth(1.0);
    unsafe { layer.setBorderColor(Some(&Theme::container_border().CGColor())); }
    item.setUserInteractionEnabled(true);
    item.setAdjustsImageWhenHighlighted(false);
    item
}


pub fn create_button(frame: CGRect, title: &str, background: Retained<UIColor>, mtm: MainThreadMarker) -> Retained<UIButton> {
    let button: Retained<UIButton> = UIButton::initWithFrame(UIButton::alloc(mtm), frame);
    button.setBackgroundColor(Some(&background));
    unsafe {
        button.setTitle_forState(Some(&NSString::from_str(title)), UIControlState::Normal);
        button.setTitleColor_forState(Some(&Theme::text()), UIControlState::Normal);
        if let Some(label) = button.titleLabel() { label.setFont(Some(&UIFont::systemFontOfSize(16.0))); }
    }
    button.setUserInteractionEnabled(true);
    button
}

pub fn create_toggle_button(frame: CGRect, mtm: MainThreadMarker) -> Retained<UIButton> {
    let button = UIButton::buttonWithType(objc2_ui_kit::UIButtonType::Custom, mtm);
    button.setFrame(frame);
    
    // Glassmorphism background
    let blur_effect = unsafe {
        let effect: Retained<UIBlurEffect> = msg_send![UIBlurEffect::class(), effectWithStyle: 2i64]; // Dark style
        effect
    };
    let effect_view = UIVisualEffectView::new(&blur_effect, mtm);
    effect_view.setFrame(button.bounds());
    effect_view.setUserInteractionEnabled(false);
    effect_view.layer().setCornerRadius(frame.size.width / 2.0);
    effect_view.setClipsToBounds(true);
    
    // Add blur effect
    button.addSubview(&effect_view);
    unsafe { button.sendSubviewToBack(&effect_view); }

    button.setBackgroundColor(Some(&UIColor::clearColor()));
    
    let layer = button.layer();
    layer.setCornerRadius(frame.size.width / 2.0);
    
    // Add a subtle gradient-like border
    layer.setBorderWidth(1.5);
    unsafe {
        layer.setBorderColor(Some(&Theme::toggle_button_border().CGColor()));
        let shadow_color = Theme::shadow().CGColor();
        layer.setShadowColor(Some(&shadow_color));
    }
    
    // Enhanced shadow
    layer.setShadowOffset(CGSize::new(0.0, 4.0));
    layer.setShadowRadius(10.0);
    layer.setShadowOpacity(0.5);
    
    unsafe {
        // Use a nicer icon or text
        button.setTitle_forState(Some(&NSString::from_str("M")), UIControlState::Normal); // M for Menu? Or Keep ≡ but styled better
        // Let's stick to ≡ but with better font
        button.setTitle_forState(Some(&NSString::from_str("≡")), UIControlState::Normal);
        button.setTitleColor_forState(Some(&Theme::text()), UIControlState::Normal);
        // Make it larger and bold
        if let Some(label) = button.titleLabel() { 
            let font = UIFont::boldSystemFontOfSize(32.0);
            label.setFont(Some(&font)); 
        }
    }
    button.setUserInteractionEnabled(true);
    button
}

pub fn create_toggle_item(frame: CGRect, text: &str, mtm: MainThreadMarker) -> Retained<UIButton> {
    let item = styled_container(frame, mtm);
    item.addSubview(&create_label(CGRect::new(CGPoint::new(12.0, 12.0), CGSize::new(180.0, 20.0)), text, 15.0, false, mtm));
    let toggle_bg: Retained<UIView> = UIView::initWithFrame(UIView::alloc(mtm), CGRect::new(CGPoint::new(frame.size.width - 54.0, 10.0), CGSize::new(44.0, 24.0)));
    toggle_bg.setBackgroundColor(Some(&Theme::toggle_off()));
    toggle_bg.layer().setCornerRadius(12.0);
    toggle_bg.setTag(2);
    toggle_bg.setUserInteractionEnabled(false);
    
    let knob: Retained<UIView> = UIView::initWithFrame(UIView::alloc(mtm), CGRect::new(CGPoint::new(2.0, 2.0), CGSize::new(20.0, 20.0)));
    knob.setBackgroundColor(Some(&Theme::accent()));
    knob.layer().setCornerRadius(10.0);
    knob.setTag(3);
    toggle_bg.addSubview(&knob);
    item.addSubview(&toggle_bg);
    item
}

pub fn create_button_item(frame: CGRect, text: &str, mtm: MainThreadMarker) -> Retained<UIButton> {
    let item = styled_container(frame, mtm);
    item.addSubview(&create_label(CGRect::new(CGPoint::new(12.0, 12.0), CGSize::new(200.0, 20.0)), text, 15.0, false, mtm));
    let arrow = create_label(CGRect::new(CGPoint::new(frame.size.width - 30.0, 12.0), CGSize::new(20.0, 20.0)), "›", 20.0, false, mtm);
    unsafe { arrow.setTextColor(Some(&Theme::text_secondary())); }
    item.addSubview(&arrow);
    item
}


pub fn create_back_button(frame: CGRect, mtm: MainThreadMarker) -> Retained<UIButton> {
    let button: Retained<UIButton> = UIButton::initWithFrame(UIButton::alloc(mtm), frame);
    unsafe {
        button.setTitle_forState(Some(&NSString::from_str("‹ Back")), UIControlState::Normal);
        button.setTitleColor_forState(Some(&Theme::text()), UIControlState::Normal);
        button.setTitleColor_forState(Some(&Theme::text_secondary()), UIControlState::Highlighted);
        if let Some(label) = button.titleLabel() { label.setFont(Some(&UIFont::systemFontOfSize(18.0))); }
        button.setContentHorizontalAlignment(objc2_ui_kit::UIControlContentHorizontalAlignment::Left);
        #[allow(deprecated)]
        button.setTitleEdgeInsets(objc2_ui_kit::UIEdgeInsets { top: 0.0, left: 0.0, bottom: 0.0, right: 0.0 });
    }
    button.setUserInteractionEnabled(true);
    button
}

pub fn create_slider(frame: CGRect, value: f32, min: f32, max: f32, mtm: MainThreadMarker) -> Retained<UISlider> {
    let slider: Retained<UISlider> = UISlider::initWithFrame(UISlider::alloc(mtm), frame);
    slider.setMinimumValue(min);
    slider.setMaximumValue(max);
    slider.setValue(value);
    unsafe {
        slider.setTintColor(Some(&Theme::accent()));
        slider.setThumbTintColor(Some(&Theme::accent()));
        slider.setMaximumTrackTintColor(Some(&Theme::slider_track_inactive()));
    }
    slider.setUserInteractionEnabled(true);
    slider
}

pub fn create_action_button_item(frame: CGRect, text: &str, mtm: MainThreadMarker) -> Retained<UIButton> {
    let item = styled_container(frame, mtm);
    let label = create_label(CGRect::new(CGPoint::new(14.0, 0.0), CGSize::new(frame.size.width - 50.0, frame.size.height)), text, 15.0, false, mtm);
    label.setUserInteractionEnabled(false);
    item.addSubview(&label);
    let arrow_label = create_label(CGRect::new(CGPoint::new(frame.size.width - 30.0, (frame.size.height - 20.0) / 2.0), CGSize::new(20.0, 20.0)), ">", 18.0, true, mtm);
    unsafe { arrow_label.setTextColor(Some(&Theme::arrow_muted())); }
    arrow_label.setUserInteractionEnabled(false);
    arrow_label.setTag(10);
    item.addSubview(&arrow_label);
    item
}

pub fn create_slider_item(frame: CGRect, label_text: &str, value: f32, min: f32, max: f32, mtm: MainThreadMarker) -> Retained<UIButton> {
    let item = styled_container(frame, mtm);
    item.addSubview(&create_label(CGRect::new(CGPoint::new(12.0, 8.0), CGSize::new(150.0, 20.0)), label_text, 14.0, false, mtm));
    
    let value_label = create_label(CGRect::new(CGPoint::new(frame.size.width - 62.0, 8.0), CGSize::new(50.0, 20.0)), &format!("{:.2}", value), 14.0, true, mtm);
    value_label.setTextAlignment(objc2_ui_kit::NSTextAlignment::Right);
    unsafe { value_label.setTextColor(Some(&Theme::accent())); }
    value_label.setTag(5);
    item.addSubview(&value_label);
    
    let slider = create_slider(CGRect::new(CGPoint::new(12.0, 32.0), CGSize::new(frame.size.width - 24.0, 30.0)), value, min, max, mtm);
    slider.setTag(4);
    item.addSubview(&slider);
    item
}

pub fn create_text_input(frame: CGRect, placeholder: &str, mtm: MainThreadMarker) -> Retained<UITextField> {
    let input: Retained<UITextField> = UITextField::initWithFrame(UITextField::alloc(mtm), frame);
    input.setBackgroundColor(Some(&Theme::input_background()));
    let layer = input.layer();
    layer.setCornerRadius(8.0);
    layer.setBorderWidth(1.0);
    unsafe {
        layer.setBorderColor(Some(&Theme::input_border().CGColor()));
        input.setTextColor(Some(&Theme::text()));
    }
    input.setPlaceholder(Some(&NSString::from_str(placeholder)));
    input.setBackgroundColor(Some(&Theme::input_placeholder_background()));
    let padding = UIView::initWithFrame(UIView::alloc(mtm), CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(10.0, 10.0)));
    input.setLeftView(Some(&padding));
    input.setLeftViewMode(objc2_ui_kit::UITextFieldViewMode::Always);
    input
}

pub fn create_text_input_item(frame: CGRect, label_text: &str, placeholder: &str, mtm: MainThreadMarker) -> Retained<UIButton> {
    let item = styled_container(frame, mtm);
    item.addSubview(&create_label(CGRect::new(CGPoint::new(12.0, 8.0), CGSize::new(200.0, 20.0)), label_text, 14.0, false, mtm));
    let input = create_text_input(CGRect::new(CGPoint::new(12.0, 32.0), CGSize::new(frame.size.width - 24.0, 32.0)), placeholder, mtm);
    input.setTag(6);
    item.addSubview(&input);
    item
}

pub fn create_header(frame: CGRect, title: &str, mtm: MainThreadMarker) -> Retained<UIButton> {
    let header: Retained<UIButton> = UIButton::initWithFrame(UIButton::alloc(mtm), frame);
    header.setBackgroundColor(Some(&Theme::container_background()));
    let layer = header.layer();
    layer.setCornerRadius(8.0);
    layer.setBorderWidth(1.0);
    unsafe { layer.setBorderColor(Some(&Theme::container_border().CGColor())); }
    header.setUserInteractionEnabled(true);
    let arrow_label = create_label(CGRect::new(CGPoint::new(16.0, 0.0), CGSize::new(20.0, frame.size.height)), "‹", 24.0, false, mtm);
    unsafe { arrow_label.setTextColor(Some(&Theme::text())); }
    header.addSubview(&arrow_label);
    let title_label = create_label(CGRect::new(CGPoint::new(frame.size.width - 216.0, 0.0), CGSize::new(200.0, frame.size.height)), title, 20.0, true, mtm);
    title_label.setTextAlignment(objc2_ui_kit::NSTextAlignment::Right);
    header.addSubview(&title_label);
    header
}
