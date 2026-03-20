//! for text field delegate
use objc2::{
    define_class, msg_send,
    rc::Retained,
    runtime::{NSObject, NSObjectProtocol},
    ClassType, MainThreadOnly,
};
use objc2_foundation::MainThreadMarker;
use objc2_ui_kit::{UITextField, UITextFieldDelegate, UIView};

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "TextFieldDelegate"]
    /// Delegate for handling text field events (Return key, Clear button)
    pub struct TextFieldDelegate;

    impl TextFieldDelegate {
        #[unsafe(method(textFieldShouldReturn:))]
        fn text_field_should_return(&self, text_field: &UITextField) -> bool {
            let _: () = unsafe { msg_send![text_field, resignFirstResponder] };
            true
        }

        #[unsafe(method(clearText:))]
        fn clear_text(&self, sender: &UIView) {
             let mut current_view: Option<Retained<UIView>> = sender.superview();
             while let Some(view) = current_view {
                 if view.isKindOfClass(UITextField::class()) {
                     let tf: Retained<UITextField> = unsafe { Retained::cast_unchecked(view) };
                     tf.setText(Some(&objc2_foundation::NSString::new()));
                     break;
                 }
                 current_view = view.superview();
             }
        }
    }
);

unsafe impl NSObjectProtocol for TextFieldDelegate {}
unsafe impl UITextFieldDelegate for TextFieldDelegate {}

impl TextFieldDelegate {
    pub fn new(_mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![Self::class(), new] }
    }

    /// Returns a shared instance of the delegate
    pub fn shared(mtm: MainThreadMarker) -> Retained<Self> {
        thread_local! {
            static DELEGATE: std::cell::OnceCell<Retained<TextFieldDelegate>> = const { std::cell::OnceCell::new() };
        }
        DELEGATE.with(|d| d.get_or_init(|| TextFieldDelegate::new(mtm)).clone())
    }
}
