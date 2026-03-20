use dispatch::Queue;
use objc2::rc::Retained;
use objc2::{msg_send, ClassType};
use objc2_core_foundation::{CGAffineTransform, CGPoint, CGRect, CGSize};
use objc2_foundation::{MainThreadMarker, NSString};
use objc2_quartz_core::CAShapeLayer;
use objc2_ui_kit::{UIColor, UIFont, UILabel, UIView};
use std::cell::RefCell;
use std::time::Duration;

use crate::ui::assets::icons;
use crate::ui::theme::Theme;
use crate::ui::utils::animations;
use crate::ui::utils::wrappers::{UIBlurEffect, UIVisualEffectView};
#[cfg(dev_release)]
use crate::utils::logger;

thread_local! {
    static ACTIVE_TOAST: RefCell<Option<Retained<UIView>>> = const { RefCell::new(None) };
}

#[derive(Clone, Copy)]
enum ToastType {
    Standard(ToastStatus),
    Loading,
    Welcome,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ToastStatus {
    Info,
    Success,
    Error,
}

/// Configuration for a toast notification
struct ToastConfig<'a> {
    text: &'a str,
    detail_text: Option<&'a str>,
    version: Option<&'a str>,
    toast_type: ToastType,
    on_finished: Option<Box<dyn FnOnce() + Send>>,
}

/// Presents a standard toast notification with the given configuration
///
/// # Arguments
///
/// * `text` - The text of the standard toast
/// * `status` - The status of the standard toast
/// * `on_finished` - The callback to be called when the standard toast is finished
pub fn show_toast(text: &str, status: ToastStatus) {
    present_toast(ToastConfig {
        text,
        detail_text: None,
        version: None,
        toast_type: ToastType::Standard(status),
        on_finished: None,
    });
}

/// Presents a loading toast notification with the given configuration
///
/// # Arguments
///
/// * `text` - The text of the loading toast
/// * `on_finished` - The callback to be called when the loading toast is finished
pub fn show_loading(text: &str) {
    present_toast(ToastConfig {
        text,
        detail_text: None,
        version: None,
        toast_type: ToastType::Loading,
        on_finished: None,
    });
}

/// Presents a welcome toast notification with the given configuration
///
/// # Arguments
///
/// * `title` - The title of the welcome toast
/// * `version` - The version of the welcome toast
/// * `description` - The description of the welcome toast
/// * `on_finished` - The callback to be called when the welcome toast is finished
pub fn show_welcome(
    title: &str,
    version: &str,
    description: &str,
    on_finished: Box<dyn FnOnce() + Send>,
) {
    present_toast(ToastConfig {
        text: title,
        detail_text: Some(description),
        version: Some(version),
        toast_type: ToastType::Welcome,
        on_finished: Some(on_finished),
    });
}

/// Presents a toast notification with the given configuration
///
/// # Arguments
///
/// * `config` - The configuration for the toast
fn present_toast(config: ToastConfig) {
    let (text, detail_text, version_text, toast_type) = (
        config.text.to_string(),
        config.detail_text.map(|s| s.to_string()),
        config.version.map(|s| s.to_string()),
        config.toast_type,
    );
    let on_finished = std::sync::Arc::new(std::sync::Mutex::new(config.on_finished));

    Queue::main().exec_async(move || {
        if let Some(mtm) = MainThreadMarker::new() {
            unsafe {
                ACTIVE_TOAST.with(|t| t.borrow().as_ref().map(|old| old.removeFromSuperview()));

                let window_opt = crate::ui::window::get_window(mtm);

                let window = match window_opt {
                    Some(w) => w,
                    None => {
                        #[cfg(dev_release)]
                        logger::warning("Failed to get window for toast!");
                        return;
                    }
                };

                let wb = window.bounds();
                let top_padding = window.safeAreaInsets().top + 12.0;
                let (cw, ch) = (126.0, 37.0);
                let cr = ch / 2.0;
                let (ew, eh, er) = match toast_type {
                    ToastType::Welcome => (350.0_f64.min(wb.size.width - 32.0), 80.0, 40.0),
                    _ => (300.0_f64.min(wb.size.width - 32.0), 50.0, 25.0),
                };
                let (sx, sy, ex) = (
                    (wb.size.width - cw) / 2.0,
                    top_padding,
                    (wb.size.width - ew) / 2.0,
                );

                let (container, effect) = create_container_view(
                    mtm,
                    CGRect::new(CGPoint::new(sx, sy), CGSize::new(cw, ch)),
                    cr,
                );

                let dots = if matches!(toast_type, ToastType::Loading) {
                    create_loading_dots(mtm, ew - 46.0, (eh - 6.0) / 2.0, &container)
                } else {
                    Vec::new()
                };

                let (status_dot, status_layer) = if let ToastType::Standard(status) = toast_type {
                    let (c, l) = create_status_icon(
                        mtm,
                        status,
                        CGRect::new(
                            CGPoint::new(ew - 36.0, (eh - 18.0) / 2.0),
                            CGSize::new(18.0, 18.0),
                        ),
                        &container,
                    );
                    (Some(c), Some(l))
                } else {
                    (None, None)
                };

                let (welcome_icon, welcome_layer) = if matches!(toast_type, ToastType::Welcome) {
                    let (c, l) = create_welcome_icon(
                        mtm,
                        CGRect::new(
                            CGPoint::new(24.0, ((eh - 40.0) / 2.0) + 5.0),
                            CGSize::new(40.0, 40.0),
                        ),
                        &container,
                    );
                    (Some(c), Some(l))
                } else {
                    (None, None)
                };

                let text_label = UILabel::new(mtm);
                let lf = match toast_type {
                    ToastType::Welcome => {
                        CGRect::new(CGPoint::new(80.0, 18.0), CGSize::new(ew - 100.0, 26.0))
                    }
                    ToastType::Loading => {
                        CGRect::new(CGPoint::new(20.0, 0.0), CGSize::new(ew - 70.0, eh))
                    }
                    ToastType::Standard(_) => {
                        CGRect::new(CGPoint::new(20.0, 0.0), CGSize::new(ew - 40.0, eh))
                    }
                };
                text_label.setFrame(lf);
                text_label.setText(Some(&NSString::from_str(&text)));
                text_label.setTextColor(Some(&Theme::text()));
                let title_size = text_label.intrinsicContentSize();

                let (version_bg, _version_label) =
                    if let (Some(ver), ToastType::Welcome) = (version_text.as_ref(), toast_type) {
                        let (bg, lbl) = create_version_badge(
                            mtm,
                            ver,
                            80.0 + title_size.width + 8.0,
                            31.0,
                            &container,
                        );
                        (Some(bg), Some(lbl))
                    } else {
                        (None, None)
                    };

                text_label.setFont(Some(&UIFont::boldSystemFontOfSize(
                    if matches!(toast_type, ToastType::Welcome) {
                        17.0
                    } else {
                        15.0
                    },
                )));
                text_label.setTextAlignment(objc2_ui_kit::NSTextAlignment::Left);
                text_label.setNumberOfLines(1);
                text_label.setAlpha(0.0);
                container.addSubview(&text_label);

                let desc_label = detail_text.as_ref().map(|desc| {
                    let dl = UILabel::new(mtm);
                    dl.setFrame(CGRect::new(
                        CGPoint::new(80.0, 46.0),
                        CGSize::new(ew - 100.0, 20.0),
                    ));
                    dl.setText(Some(&NSString::from_str(desc)));
                    dl.setTextColor(Some(&Theme::text_secondary()));
                    dl.setFont(Some(&UIFont::systemFontOfSize(15.0)));
                    dl.setTextAlignment(objc2_ui_kit::NSTextAlignment::Left);
                    dl.setNumberOfLines(1);
                    dl.setAlpha(0.0);
                    container.addSubview(&dl);
                    dl
                });

                container.setTransform(CGAffineTransform {
                    a: 0.8,
                    b: 0.0,
                    c: 0.0,
                    d: 0.8,
                    tx: 0.0,
                    ty: -30.0,
                });
                container.setAlpha(0.0);
                window.addSubview(&container);
                ACTIVE_TOAST.with(|t| *t.borrow_mut() = Some(container.clone()));

                let c1 = container.clone();
                animations::animate_spring_with_delay(
                    0.5,
                    0.0,
                    0.82,
                    0.0,
                    0,
                    move || {
                        c1.setTransform(CGAffineTransform {
                            a: 1.0,
                            b: 0.0,
                            c: 0.0,
                            d: 1.0,
                            tx: 0.0,
                            ty: 0.0,
                        });
                        c1.setAlpha(1.0);
                    },
                    Some({
                        let (c2, eff, txt, dots_e, dots_b, desc, sd, sl, wi, wl, vb) = (
                            container.clone(),
                            effect.clone(),
                            text_label.clone(),
                            dots.clone(),
                            dots.clone(),
                            desc_label.clone(),
                            status_dot.clone(),
                            status_layer.clone(),
                            welcome_icon.clone(),
                            welcome_layer.clone(),
                            version_bg.clone(),
                        );
                        move |_| {
                            animations::animate_spring_with_delay(
                                0.5,
                                0.05,
                                0.7,
                                0.4,
                                0,
                                {
                                    let (c, e, t, d, desc, sd, wi, vb) = (
                                        c2.clone(),
                                        eff.clone(),
                                        txt.clone(),
                                        dots_e.clone(),
                                        desc.clone(),
                                        sd.clone(),
                                        wi.clone(),
                                        vb.clone(),
                                    );
                                    move || {
                                        c.setFrame(CGRect::new(
                                            CGPoint::new(ex, top_padding),
                                            CGSize::new(ew, eh),
                                        ));
                                        e.setFrame(CGRect::new(
                                            CGPoint::new(0.0, 0.0),
                                            CGSize::new(ew, eh),
                                        ));
                                        e.layer().setCornerRadius(er);
                                        t.setAlpha(1.0);
                                        d.iter().for_each(|dot| dot.setAlpha(1.0));
                                        if let Some(d) = desc.as_ref() {
                                            d.setAlpha(1.0)
                                        }
                                        if let Some(s) = sd.as_ref() {
                                            s.setAlpha(1.0)
                                        }
                                        if let Some(w) = wi.as_ref() {
                                            w.setAlpha(1.0)
                                        }
                                        if let Some(v) = vb.as_ref() {
                                            v.setAlpha(1.0)
                                        }
                                    }
                                },
                                None::<fn(bool)>,
                            );

                            dots_b.iter().enumerate().for_each(|(i, dot)| {
                                let d = dot.clone();
                                animations::animate_with_delay(
                                    0.4,
                                    i as f64 * 0.15,
                                    24,
                                    move || {
                                        d.setTransform(CGAffineTransform {
                                            a: 0.8,
                                            b: 0.0,
                                            c: 0.0,
                                            d: 0.8,
                                            tx: 0.0,
                                            ty: -8.0,
                                        });
                                    },
                                    None::<fn(bool)>,
                                );
                            });
                            if let Some(l) = sl.as_ref() {
                                animations::animate_stroke_end(l, 0.5, false)
                            }
                            if let Some(l) = wl.as_ref() {
                                animations::animate_stroke_end(l, 1.2, true)
                            }
                        }
                    }),
                );

                if matches!(toast_type, ToastType::Loading) {
                    return;
                }

                let c_raw = Retained::into_raw(container) as usize;
                let dur = if matches!(toast_type, ToastType::Welcome) {
                    3500
                } else {
                    2000
                };

                Queue::main().exec_after(Duration::from_millis(dur), move || {
                    let container = Retained::<UIView>::from_raw(c_raw as *mut UIView).unwrap();
                    let cf = container.clone();

                    animations::animate_with_delay(
                        0.2,
                        0.0,
                        0,
                        move || {
                            let sv = cf.subviews();
                            (0..sv.count())
                                .skip(1)
                                .for_each(|i| sv.objectAtIndex(i).setAlpha(0.0));
                        },
                        Some({
                            let c3 = container.clone();
                            move |_| {
                                animations::animate_spring_with_delay(
                                    0.5,
                                    0.0,
                                    0.8,
                                    0.2,
                                    0,
                                    {
                                        let c = c3.clone();
                                        move || {
                                            c.setFrame(CGRect::new(
                                                CGPoint::new(sx, top_padding),
                                                CGSize::new(cw, ch),
                                            ));
                                            if let Some(e) = c.subviews().firstObject() {
                                                let ev: &UIView = &e;
                                                ev.setFrame(CGRect::new(
                                                    CGPoint::new(0.0, 0.0),
                                                    CGSize::new(cw, ch),
                                                ));
                                                ev.layer().setCornerRadius(cr);
                                            }
                                        }
                                    },
                                    Some({
                                        let c4 = c3.clone();
                                        let of = on_finished.clone();
                                        move |_| {
                                            animations::animate(
                                                0.3,
                                                {
                                                    let c = c4.clone();
                                                    move || {
                                                        c.setAlpha(0.0);
                                                        c.setTransform(CGAffineTransform {
                                                            a: 0.8,
                                                            b: 0.0,
                                                            c: 0.0,
                                                            d: 0.8,
                                                            tx: 0.0,
                                                            ty: -20.0,
                                                        });
                                                    }
                                                },
                                                Some({
                                                    let c = c4.clone();
                                                    let of = of.clone();
                                                    move |_| {
                                                        c.removeFromSuperview();
                                                        ACTIVE_TOAST
                                                            .with(|t| *t.borrow_mut() = None);
                                                        if let Some(cb) = of
                                                            .lock()
                                                            .ok()
                                                            .and_then(|mut l| l.take())
                                                        {
                                                            cb()
                                                        }
                                                    }
                                                }),
                                            );
                                        }
                                    }),
                                );
                            }
                        }),
                    );
                });
            }
        }
    });
}

fn create_container_view(
    mtm: MainThreadMarker,
    frame: CGRect,
    cr: f64,
) -> (Retained<UIView>, Retained<UIVisualEffectView>) {
    let c = UIView::new(mtm);
    c.setFrame(frame);
    let l = c.layer();
    #[allow(clippy::missing_transmute_annotations)]
    l.setShadowColor(Some(unsafe {
        std::mem::transmute(UIColor::blackColor().CGColor())
    }));
    l.setShadowOpacity(0.15);
    l.setShadowOffset(CGSize::new(0.0, 5.0));
    l.setShadowRadius(12.0);
    let be: Retained<UIBlurEffect> =
        unsafe { msg_send![UIBlurEffect::class(), effectWithStyle: 2i64] };
    let ev = UIVisualEffectView::new(&be, mtm);
    ev.setFrame(c.bounds());
    ev.setUserInteractionEnabled(false);
    ev.layer().setCornerRadius(cr);
    ev.layer()
        .setCornerCurve(unsafe { objc2_quartz_core::kCACornerCurveContinuous });
    ev.setClipsToBounds(true);
    c.addSubview(&ev);
    (c, ev)
}

fn create_loading_dots(
    mtm: MainThreadMarker,
    sx: f64,
    y: f64,
    container: &UIView,
) -> Vec<Retained<UIView>> {
    (0..3)
        .map(|i| {
            let d = UIView::new(mtm);
            d.setFrame(CGRect::new(
                CGPoint::new(sx + (i as f64 * 10.0), y),
                CGSize::new(6.0, 6.0),
            ));
            d.setBackgroundColor(Some(&Theme::text()));
            d.layer().setCornerRadius(3.0);
            d.setAlpha(0.0);
            container.addSubview(&d);
            d
        })
        .collect()
}

fn create_status_icon(
    mtm: MainThreadMarker,
    status: ToastStatus,
    frame: CGRect,
    container: &UIView,
) -> (Retained<UIView>, Retained<CAShapeLayer>) {
    let ic = UIView::new(mtm);
    ic.setFrame(frame);
    ic.setAlpha(0.0);
    container.addSubview(&ic);
    let l = CAShapeLayer::new();
    l.setFrame(ic.bounds());
    l.setFillColor(None);
    l.setLineWidth(1.8);
    unsafe {
        l.setLineCap(objc2_quartz_core::kCALineCapRound);
        l.setLineJoin(objc2_quartz_core::kCALineJoinRound);
    }
    let col = match status {
        ToastStatus::Success => UIColor::systemGreenColor(),
        ToastStatus::Error => UIColor::systemRedColor(),
        ToastStatus::Info => Theme::text(),
    };
    #[allow(clippy::missing_transmute_annotations)]
    l.setStrokeColor(Some(unsafe { std::mem::transmute(col.CGColor()) }));
    let path = match status {
        ToastStatus::Success => icons::success_path(),
        ToastStatus::Error => icons::error_path(),
        ToastStatus::Info => icons::info_path(),
    };
    #[allow(clippy::missing_transmute_annotations)]
    l.setPath(Some(unsafe { std::mem::transmute(path.CGPath()) }));
    l.setStrokeEnd(0.0);
    ic.layer().addSublayer(&l);
    (ic, l)
}

fn create_welcome_icon(
    mtm: MainThreadMarker,
    frame: CGRect,
    container: &UIView,
) -> (Retained<UIView>, Retained<CAShapeLayer>) {
    let ic = UIView::new(mtm);
    ic.setFrame(frame);
    ic.setAlpha(0.0);
    container.addSubview(&ic);
    let l = CAShapeLayer::new();
    l.setFrame(ic.bounds());
    l.setFillColor(None);
    l.setLineWidth(2.5);
    unsafe {
        l.setLineCap(objc2_quartz_core::kCALineCapRound);
        l.setLineJoin(objc2_quartz_core::kCALineJoinRound);
    }
    #[allow(clippy::missing_transmute_annotations)]
    l.setStrokeColor(Some(unsafe {
        std::mem::transmute(Theme::accent().CGColor())
    }));
    #[allow(clippy::missing_transmute_annotations)]
    l.setPath(Some(unsafe {
        std::mem::transmute(icons::dragon_head_path().CGPath())
    }));
    l.setStrokeEnd(0.0);
    ic.layer().addSublayer(&l);
    (ic, l)
}

fn create_version_badge(
    mtm: MainThreadMarker,
    version: &str,
    start_x: f64,
    center_y: f64,
    container: &UIView,
) -> (Retained<UIView>, Retained<UILabel>) {
    let (bg, lbl) = (UIView::new(mtm), UILabel::new(mtm));
    let ver_ns = NSString::from_str(version);

    lbl.setText(Some(&ver_ns));
    unsafe {
        lbl.setFont(Some(&UIFont::boldSystemFontOfSize(10.0)));
        lbl.setTextColor(Some(&Theme::accent()));
    }
    lbl.setTextAlignment(objc2_ui_kit::NSTextAlignment::Center);

    let bs = lbl.intrinsicContentSize();
    let (bw, bh) = (bs.width + 12.0, 18.0);

    let y = center_y - (bh / 2.0) + 1.0;

    bg.setFrame(CGRect::new(CGPoint::new(start_x, y), CGSize::new(bw, bh)));
    bg.setBackgroundColor(Some(&Theme::accent_soft()));
    bg.layer().setCornerRadius(bh / 2.0);
    unsafe {
        bg.layer()
            .setCornerCurve(objc2_quartz_core::kCACornerCurveContinuous);
    }

    bg.setAlpha(0.0);

    lbl.setFrame(bg.bounds());
    lbl.setAlpha(1.0);

    bg.addSubview(&lbl);
    container.addSubview(&bg);

    (bg, lbl)
}
