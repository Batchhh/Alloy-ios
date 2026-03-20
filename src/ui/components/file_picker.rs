//! File picker
use dispatch::Queue;
use objc2::{
    define_class, msg_send,
    rc::Retained,
    runtime::{AnyClass, AnyObject, Bool, NSObject, NSObjectProtocol},
    ClassType, MainThreadOnly,
};
use objc2_foundation::{MainThreadMarker, NSArray, NSURL};
use objc2_ui_kit::UIApplication;
use parking_lot::Mutex;
use std::ffi::c_void;

/// Callback type for when a folder is selected
type FolderCallback = Box<dyn FnOnce(Retained<NSURL>) + Send + 'static>;

static PENDING_CALLBACK: Mutex<Option<FolderCallback>> = Mutex::new(None);
static PICKED_URL: Mutex<Option<Retained<NSURL>>> = Mutex::new(None);

/// Executes a closure with the picked path, handling security scoping
///
/// # Arguments
/// * `f` - Closure to execute with the picked path
pub fn with_picked_path<F>(f: F) -> Option<()>
where
    F: FnOnce(&str),
{
    let guard = PICKED_URL.lock();
    if let Some(url) = guard.as_ref() {
        let started: bool = unsafe { msg_send![url, startAccessingSecurityScopedResource] };

        if let Some(path) = url.path() {
            f(&path.to_string());
        }

        if started {
            unsafe {
                let _: () = msg_send![url, stopAccessingSecurityScopedResource];
            }
        }
        Some(())
    } else {
        None
    }
}

/// Returns true if a picked path is set
pub fn has_picked_path() -> bool {
    PICKED_URL.lock().is_some()
}

/// Sets the picked path
pub fn set_picked_path(url: Retained<NSURL>) {
    *PICKED_URL.lock() = Some(url);
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "FolderPickerDelegate"]
    /// Delegate for UIDocumentPickerViewController
    pub struct FolderPickerDelegate;

    impl FolderPickerDelegate {
        /// Called when user picks a folder
        #[unsafe(method(documentPicker:didPickDocumentsAtURLs:))]
        fn did_pick_documents(&self, _picker: &AnyObject, urls: &NSArray<NSURL>) {
            if urls.count() > 0 {
                let url = urls.objectAtIndex(0);

                if let Some(callback) = PENDING_CALLBACK.lock().take() {
                    callback(url);
                }
            }
        }

        /// Called when user cancels the picker
        #[unsafe(method(documentPickerWasCancelled:))]
        fn document_picker_was_cancelled(&self, _picker: &AnyObject) {
            let _ = PENDING_CALLBACK.lock().take();
        }
    }
);

unsafe impl NSObjectProtocol for FolderPickerDelegate {}

impl FolderPickerDelegate {
    pub fn new(_mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![Self::class(), new] }
    }
}

thread_local! {
    static DELEGATE: std::cell::RefCell<Option<Retained<FolderPickerDelegate>>> = const { std::cell::RefCell::new(None) };
}

/// Shows a folder picker and calls the callback with the selected URL
///
/// # Arguments
/// * `on_select` - Callback function called with the selected folder URL
pub fn pick_folder<F>(on_select: F)
where
    F: FnOnce(Retained<NSURL>) + Send + 'static,
{
    *PENDING_CALLBACK.lock() = Some(Box::new(on_select));

    Queue::main().exec_async(|| {
        if let Some(mtm) = MainThreadMarker::new() {
            unsafe {
                let delegate = FolderPickerDelegate::new(mtm);
                DELEGATE.with(|d| *d.borrow_mut() = Some(delegate.clone()));

                let picker_cls = AnyClass::get(c"UIDocumentPickerViewController")
                    .expect("UIDocumentPickerViewController class not found");

                let ut_type_cls = AnyClass::get(c"UTType").expect("UTType class not found");
                let folder_id = objc2_foundation::NSString::from_str("public.folder");
                let folder_type: *mut AnyObject =
                    msg_send![ut_type_cls, typeWithIdentifier: &*folder_id];

                let types_array: Retained<NSArray<AnyObject>> =
                    NSArray::from_slice(&[&*folder_type]);

                let picker: *mut AnyObject = msg_send![picker_cls, alloc];
                let picker: *mut AnyObject =
                    msg_send![picker, initForOpeningContentTypes: &*types_array];
                let _: () = msg_send![picker, setDelegate: &*delegate as *const _ as *const c_void];
                let _: () = msg_send![picker, setAllowsMultipleSelection: Bool::NO];

                let app = UIApplication::sharedApplication(mtm);
                #[allow(deprecated)]
                let window_opt = if let Some(w) = app.keyWindow() {
                    Some(w)
                } else {
                    #[allow(deprecated)]
                    let windows = app.windows();
                    if windows.count() > 0 {
                        Some(windows.objectAtIndex(0))
                    } else {
                        None
                    }
                };

                if let Some(window) = window_opt {
                    if let Some(root) = window.rootViewController() {
                        let _: () = msg_send![
                            &root,
                            presentViewController: picker,
                            animated: true,
                            completion: std::ptr::null::<c_void>()
                        ];
                    }
                }
            }
        }
    });
}
