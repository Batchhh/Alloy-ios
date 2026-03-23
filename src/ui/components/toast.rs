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
    toast_type: ToastType,
}

/// Presents a standard toast notification with the given configuration
///
/// # Arguments
///
/// * `text` - The text of the standard toast
/// * `status` - The status of the standard toast
pub fn show_toast(text: &str, status: ToastStatus) {
    present_toast(ToastConfig {
        text,
        toast_type: ToastType::Standard(status),
    });
}

/// Presents a loading toast notification with the given configuration
///
/// # Arguments
///
/// * `text` - The text of the loading toast
pub fn show_loading(text: &str) {
    present_toast(ToastConfig {
        text,
        toast_type: ToastType::Loading,
    });
}

/// Presents a toast notification with the given configuration
///
/// # Arguments
///
/// * `config` - The configuration for the toast
fn present_toast(config: ToastConfig) {
    let (text, toast_type) = (config.text.to_string(), config.toast_type);

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
                let (ew, eh, er) = (300.0_f64.min(wb.size.width - 32.0), 50.0, 25.0);
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

                let text_label = UILabel::new(mtm);
                let lf = match toast_type {
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

                text_label.setFont(Some(&UIFont::boldSystemFontOfSize(15.0)));
                text_label.setTextAlignment(objc2_ui_kit::NSTextAlignment::Left);
                text_label.setNumberOfLines(1);
                text_label.setAlpha(0.0);
                container.addSubview(&text_label);

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
                        let c2 = container.clone();
                        let eff = effect.clone();
                        let txt = text_label.clone();
                        let dots = dots.clone();
                        let sd = status_dot.clone();
                        let sl = status_layer.clone();
                        move |_| {
                            animations::animate_spring_with_delay(
                                0.5,
                                0.05,
                                0.7,
                                0.4,
                                0,
                                {
                                    let c = c2.clone();
                                    let e = eff.clone();
                                    let t = txt.clone();
                                    let d = dots.clone();
                                    let sd = sd.clone();
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
                                        if let Some(s) = sd.as_ref() {
                                            s.setAlpha(1.0)
                                        }
                                    }
                                },
                                None::<fn(bool)>,
                            );

                            dots.iter().enumerate().for_each(|(i, dot)| {
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
                        }
                    }),
                );

                if matches!(toast_type, ToastType::Loading) {
                    return;
                }

                let c_raw = Retained::into_raw(container) as usize;

                Queue::main().exec_after(Duration::from_millis(2000), move || {
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
                                                    move |_| {
                                                        c.removeFromSuperview();
                                                        ACTIVE_TOAST
                                                            .with(|t| *t.borrow_mut() = None);
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
