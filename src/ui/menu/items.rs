//! Items creation
use objc2::rc::Retained;
use objc2::MainThreadOnly;
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_foundation::MainThreadMarker;
use objc2_ui_kit::{UIButton, UIView};

use crate::ui::components::widgets::{
    create_label, create_slider, create_text_input, styled_container,
};
use crate::ui::theme::Theme;

/// Applies the card toggle border style (accent border when on, default when off)
fn apply_card_toggle_border(item: &UIButton, selected: bool) {
    unsafe {
        let color = if selected {
            Theme::accent()
        } else {
            Theme::container_border()
        };
        item.layer().setBorderColor(Some(&color.CGColor()));
        item.layer()
            .setBorderWidth(if selected { 1.5 } else { 0.5 });
    }
}

/// Creates a list item with a boolean toggle switch
///
/// # Arguments
/// * `frame` - Item frame
/// * `text` - Label text
/// * `mtm` - Main thread marker
pub fn create_toggle_item(frame: CGRect, text: &str, mtm: MainThreadMarker) -> Retained<UIButton> {
    let item = styled_container(frame, mtm);
    item.addSubview(&create_label(
        CGRect::new(CGPoint::new(12.0, 12.0), CGSize::new(180.0, 20.0)),
        text,
        15.0,
        false,
        mtm,
    ));
    let toggle_bg: Retained<UIView> = UIView::initWithFrame(
        UIView::alloc(mtm),
        CGRect::new(
            CGPoint::new(frame.size.width - 54.0, 10.0),
            CGSize::new(44.0, 24.0),
        ),
    );
    toggle_bg.setBackgroundColor(Some(&Theme::toggle_off()));
    toggle_bg.layer().setCornerRadius(12.0);
    toggle_bg.setTag(2);
    toggle_bg.setUserInteractionEnabled(false);

    let knob: Retained<UIView> = UIView::initWithFrame(
        UIView::alloc(mtm),
        CGRect::new(CGPoint::new(2.0, 2.0), CGSize::new(20.0, 20.0)),
    );
    knob.setBackgroundColor(Some(&Theme::accent()));
    knob.layer().setCornerRadius(10.0);
    knob.setTag(3);
    toggle_bg.addSubview(&knob);
    item.addSubview(&toggle_bg);
    item
}

/// Creates a list item that acts as a navigation button (with arrow)
///
/// # Arguments
/// * `frame` - Item frame
/// * `text` - Label text
/// * `mtm` - Main thread marker
pub fn create_button_item(frame: CGRect, text: &str, mtm: MainThreadMarker) -> Retained<UIButton> {
    let item = styled_container(frame, mtm);
    item.addSubview(&create_label(
        CGRect::new(CGPoint::new(12.0, 12.0), CGSize::new(200.0, 20.0)),
        text,
        15.0,
        false,
        mtm,
    ));
    let arrow = create_label(
        CGRect::new(
            CGPoint::new(frame.size.width - 30.0, 12.0),
            CGSize::new(20.0, 20.0),
        ),
        "›",
        20.0,
        false,
        mtm,
    );
    unsafe {
        arrow.setTextColor(Some(&Theme::text_secondary()));
    }
    item.addSubview(&arrow);
    item
}

/// Creates an action button item (e.g. for triggering a function)
///
/// Similar to a navigation button but styled for actions.
///
/// # Arguments
/// * `frame` - Item frame
/// * `text` - Button text
/// * `mtm` - Main thread marker
pub fn create_action_button_item(
    frame: CGRect,
    text: &str,
    mtm: MainThreadMarker,
) -> Retained<UIButton> {
    let item = styled_container(frame, mtm);
    let label = create_label(
        CGRect::new(
            CGPoint::new(14.0, 0.0),
            CGSize::new(frame.size.width - 50.0, frame.size.height),
        ),
        text,
        15.0,
        false,
        mtm,
    );
    label.setUserInteractionEnabled(false);
    item.addSubview(&label);
    let arrow_label = create_label(
        CGRect::new(
            CGPoint::new(frame.size.width - 30.0, (frame.size.height - 20.0) / 2.0),
            CGSize::new(20.0, 20.0),
        ),
        ">",
        18.0,
        true,
        mtm,
    );
    unsafe {
        arrow_label.setTextColor(Some(&Theme::arrow_muted()));
    }
    arrow_label.setUserInteractionEnabled(false);
    arrow_label.setTag(10);
    item.addSubview(&arrow_label);
    item
}

/// Creates a list item with a slider and value label
///
/// # Arguments
/// * `frame` - Item frame
/// * `label_text` - Label text
/// * `value` - Initial value
/// * `min` - Minimum value
/// * `max` - Maximum value
/// * `mtm` - Main thread marker
pub fn create_slider_item(
    frame: CGRect,
    label_text: &str,
    value: f32,
    min: f32,
    max: f32,
    mtm: MainThreadMarker,
) -> Retained<UIButton> {
    let item = styled_container(frame, mtm);

    let label_width = 120.0;
    item.addSubview(&create_label(
        CGRect::new(CGPoint::new(12.0, 12.0), CGSize::new(label_width, 20.0)),
        label_text,
        14.0,
        false,
        mtm,
    ));

    let value_label = create_label(
        CGRect::new(
            CGPoint::new(frame.size.width - 50.0, 12.0),
            CGSize::new(38.0, 20.0),
        ),
        &format!("{:.0}", value),
        12.0,
        true,
        mtm,
    );
    value_label.setTextAlignment(objc2_ui_kit::NSTextAlignment::Right);
    unsafe {
        value_label.setTextColor(Some(&Theme::accent()));
    }
    value_label.setTag(5);
    item.addSubview(&value_label);

    let slider_x = label_width + 12.0 + 5.0;
    let slider_width = frame.size.width - slider_x - 50.0 - 5.0;

    let slider = create_slider(
        CGRect::new(CGPoint::new(slider_x, 8.0), CGSize::new(slider_width, 30.0)),
        value,
        min,
        max,
        mtm,
    );
    slider.setTag(4);
    item.addSubview(&slider);
    item
}

/// Creates a list item with a text input field
///
/// # Arguments
/// * `frame` - Item frame
/// * `label_text` - Label text
/// * `placeholder` - Input placeholder
/// * `mtm` - Main thread marker
pub fn create_text_input_item(
    frame: CGRect,
    label_text: &str,
    placeholder: &str,
    mtm: MainThreadMarker,
) -> Retained<UIButton> {
    let item = styled_container(frame, mtm);
    item.addSubview(&create_label(
        CGRect::new(CGPoint::new(12.0, 8.0), CGSize::new(200.0, 20.0)),
        label_text,
        14.0,
        false,
        mtm,
    ));
    let input = create_text_input(
        CGRect::new(
            CGPoint::new(12.0, 32.0),
            CGSize::new(frame.size.width - 24.0, 32.0),
        ),
        placeholder,
        mtm,
    );
    input.setTag(6);

    let delegate = crate::ui::utils::delegate::TextFieldDelegate::shared(mtm);
    let delegate_ref = objc2::runtime::ProtocolObject::from_ref(&*delegate);
    input.setDelegate(Some(delegate_ref));

    item.addSubview(&input);
    item
}

/// Creates a dropdown list item
///
/// # Arguments
/// * `frame` - Item frame
/// * `label_text` - Label text
/// * `current_option` - Currently selected option text
/// * `mtm` - Main thread marker
pub fn create_dropdown_item(
    frame: CGRect,
    label_text: &str,
    current_option: &str,
    mtm: MainThreadMarker,
) -> Retained<UIButton> {
    let item = styled_container(frame, mtm);
    item.addSubview(&create_label(
        CGRect::new(CGPoint::new(12.0, 12.0), CGSize::new(200.0, 20.0)),
        label_text,
        15.0,
        false,
        mtm,
    ));

    let value_label = create_label(
        CGRect::new(
            CGPoint::new(frame.size.width - 160.0, 12.0),
            CGSize::new(140.0, 20.0),
        ),
        current_option,
        15.0,
        true,
        mtm,
    );
    value_label.setTextAlignment(objc2_ui_kit::NSTextAlignment::Right);
    unsafe {
        value_label.setTextColor(Some(&Theme::accent()));
    }
    value_label.setTag(7);
    item.addSubview(&value_label);
    item
}

/// Creates a two-row slider item whose card border indicates toggle state.
///
/// Tap the card (outside the slider track) to toggle. The border changes
/// between accent (on) and default (off).
pub fn create_slider_toggle_item(
    frame: CGRect,
    label_text: &str,
    value: f32,
    min: f32,
    max: f32,
    toggle_selected: bool,
    mtm: MainThreadMarker,
) -> Retained<UIButton> {
    let item = styled_container(frame, mtm);
    apply_card_toggle_border(&item, toggle_selected);

    let value_width = 42.0;
    let value_x = frame.size.width - value_width - 12.0;
    let label_width = value_x - 22.0;

    item.addSubview(&create_label(
        CGRect::new(CGPoint::new(12.0, 10.0), CGSize::new(label_width, 20.0)),
        label_text,
        14.0,
        false,
        mtm,
    ));

    let value_label = create_label(
        CGRect::new(CGPoint::new(value_x, 10.0), CGSize::new(value_width, 20.0)),
        &format!("{:.0}", value),
        12.0,
        true,
        mtm,
    );
    value_label.setTextAlignment(objc2_ui_kit::NSTextAlignment::Right);
    unsafe {
        value_label.setTextColor(Some(&Theme::accent()));
    }
    value_label.setTag(5);
    item.addSubview(&value_label);

    let slider = create_slider(
        CGRect::new(
            CGPoint::new(12.0, 38.0),
            CGSize::new(frame.size.width - 24.0, 30.0),
        ),
        value,
        min,
        max,
        mtm,
    );
    slider.setTag(4);
    item.addSubview(&slider);
    item
}

/// Creates a single-row input item whose card border indicates toggle state.
///
/// Tap the card (outside the text field) to toggle. The border changes
/// between accent (on) and default (off).
pub fn create_text_input_toggle_item(
    frame: CGRect,
    label_text: &str,
    placeholder: &str,
    toggle_selected: bool,
    mtm: MainThreadMarker,
) -> Retained<UIButton> {
    let item = styled_container(frame, mtm);
    apply_card_toggle_border(&item, toggle_selected);

    let input_x = 136.0;
    let input_width = frame.size.width - input_x - 12.0;

    item.addSubview(&create_label(
        CGRect::new(CGPoint::new(12.0, 15.0), CGSize::new(116.0, 20.0)),
        label_text,
        14.0,
        false,
        mtm,
    ));

    let input = create_text_input(
        CGRect::new(CGPoint::new(input_x, 9.0), CGSize::new(input_width, 32.0)),
        placeholder,
        mtm,
    );
    input.setTag(6);

    let delegate = crate::ui::utils::delegate::TextFieldDelegate::shared(mtm);
    let delegate_ref = objc2::runtime::ProtocolObject::from_ref(&*delegate);
    input.setDelegate(Some(delegate_ref));
    item.addSubview(&input);
    item
}
