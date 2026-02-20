use objc2::msg_send;
use objc2::rc::Retained;
use objc2_core_foundation::CGPoint;
use objc2_ui_kit::UIBezierPath;

/// Returns the path for the success icon (Shield + Checkmark)
pub fn success_path() -> Retained<UIBezierPath> {
    let path = UIBezierPath::bezierPath();

    // Shield (Curved sides)
    path.moveToPoint(CGPoint::new(4.0, 4.0));
    path.addLineToPoint(CGPoint::new(16.0, 4.0));
    unsafe {
        let _: () = msg_send![
            &path,
            addCurveToPoint: CGPoint::new(10.0, 18.0)
            controlPoint1: CGPoint::new(16.0, 11.0)
            controlPoint2: CGPoint::new(13.0, 16.0)
        ];
        let _: () = msg_send![
            &path,
            addCurveToPoint: CGPoint::new(4.0, 4.0)
            controlPoint1: CGPoint::new(7.0, 16.0)
            controlPoint2: CGPoint::new(4.0, 11.0)
        ];
    }

    // Checkmark (Centered and balanced)
    path.moveToPoint(CGPoint::new(7.0, 10.0));
    path.addLineToPoint(CGPoint::new(9.5, 12.5));
    path.addLineToPoint(CGPoint::new(13.5, 7.5));
    path
}

/// Returns the path for the error icon (Triangle + Exclamation)
pub fn error_path() -> Retained<UIBezierPath> {
    let path = UIBezierPath::bezierPath();
    // Triangle
    path.moveToPoint(CGPoint::new(10.0, 3.0));
    path.addLineToPoint(CGPoint::new(18.0, 17.0));
    path.addLineToPoint(CGPoint::new(2.0, 17.0));
    path.closePath();

    // Exclamation
    path.moveToPoint(CGPoint::new(10.0, 7.0));
    path.addLineToPoint(CGPoint::new(10.0, 12.0));

    // Dot
    path.moveToPoint(CGPoint::new(10.0, 14.5));
    path.addLineToPoint(CGPoint::new(10.0, 14.5));
    path
}

/// Returns the path for the info icon (Hexagon + 'i')
pub fn info_path() -> Retained<UIBezierPath> {
    let path = UIBezierPath::bezierPath();
    // Hexagon
    path.moveToPoint(CGPoint::new(10.0, 2.0));
    path.addLineToPoint(CGPoint::new(17.0, 6.0));
    path.addLineToPoint(CGPoint::new(17.0, 14.0));
    path.addLineToPoint(CGPoint::new(10.0, 18.0));
    path.addLineToPoint(CGPoint::new(3.0, 14.0));
    path.addLineToPoint(CGPoint::new(3.0, 6.0));
    path.closePath();

    // Info 'i'
    // Dot
    path.moveToPoint(CGPoint::new(10.0, 6.0));
    path.addLineToPoint(CGPoint::new(10.0, 6.0));

    // Line
    path.moveToPoint(CGPoint::new(10.0, 9.0));
    path.addLineToPoint(CGPoint::new(10.0, 14.0));
    path
}

/// Returns the path for the welcome icon (Dragon Head)
pub fn dragon_head_path() -> Retained<UIBezierPath> {
    let path = UIBezierPath::bezierPath();
    path.moveToPoint(CGPoint::new(21.25, 8.81));
    path.addLineToPoint(CGPoint::new(18.41, 8.10));
    unsafe {
        let _: () = msg_send![
            &path,
            addCurveToPoint: CGPoint::new(17.76, 7.34)
            controlPoint1: CGPoint::new(18.06, 8.01)
            controlPoint2: CGPoint::new(17.79, 7.71)
        ];
    }
    unsafe {
        let _: () = msg_send![
            &path,
            addCurveToPoint: CGPoint::new(18.23, 6.46)
            controlPoint1: CGPoint::new(17.72, 6.98)
            controlPoint2: CGPoint::new(17.91, 6.63)
        ];
    }
    path.addLineToPoint(CGPoint::new(20.46, 5.35));
    path.addLineToPoint(CGPoint::new(18.10, 3.58));
    unsafe {
        let _: () = msg_send![
            &path,
            addCurveToPoint: CGPoint::new(17.79, 2.60)
            controlPoint1: CGPoint::new(17.80, 3.35)
            controlPoint2: CGPoint::new(17.67, 2.96)
        ];
    }
    unsafe {
        let _: () = msg_send![
            &path,
            addCurveToPoint: CGPoint::new(18.62, 2.00)
            controlPoint1: CGPoint::new(17.91, 2.24)
            controlPoint2: CGPoint::new(18.25, 2.00)
        ];
    }
    path.addLineToPoint(CGPoint::new(24.75, 2.00));
    path.addLineToPoint(CGPoint::new(26.50, 2.00));
    path.addLineToPoint(CGPoint::new(27.38, 2.00));
    unsafe {
        let _: () = msg_send![
            &path,
            addCurveToPoint: CGPoint::new(31.57, 4.10)
            controlPoint1: CGPoint::new(29.03, 2.00)
            controlPoint2: CGPoint::new(30.59, 2.78)
        ];
    }
    path.addLineToPoint(CGPoint::new(34.73, 8.30));
    unsafe {
        let _: () = msg_send![
            &path,
            addCurveToPoint: CGPoint::new(35.25, 9.88)
            controlPoint1: CGPoint::new(35.06, 8.75)
            controlPoint2: CGPoint::new(35.25, 9.31)
        ];
    }
    unsafe {
        let _: () = msg_send![
            &path,
            addCurveToPoint: CGPoint::new(32.62, 12.50)
            controlPoint1: CGPoint::new(35.25, 11.32)
            controlPoint2: CGPoint::new(34.07, 12.50)
        ];
    }
    path.addLineToPoint(CGPoint::new(31.45, 12.50));
    unsafe {
        let _: () = msg_send![
            &path,
            addCurveToPoint: CGPoint::new(28.97, 11.48)
            controlPoint1: CGPoint::new(30.52, 12.50)
            controlPoint2: CGPoint::new(29.63, 12.13)
        ];
    }
    path.addLineToPoint(CGPoint::new(28.25, 10.75));
    path.addLineToPoint(CGPoint::new(26.50, 10.75));
    path.addLineToPoint(CGPoint::new(26.50, 11.93));
    unsafe {
        let _: () = msg_send![
            &path,
            addCurveToPoint: CGPoint::new(28.35, 15.27)
            controlPoint1: CGPoint::new(26.50, 13.28)
            controlPoint2: CGPoint::new(27.20, 14.55)
        ];
    }
    path.addLineToPoint(CGPoint::new(34.18, 18.91));
    unsafe {
        let _: () = msg_send![
            &path,
            addCurveToPoint: CGPoint::new(37.00, 24.00)
            controlPoint1: CGPoint::new(35.93, 20.01)
            controlPoint2: CGPoint::new(37.00, 21.93)
        ];
    }
    unsafe {
        let _: () = msg_send![
            &path,
            addCurveToPoint: CGPoint::new(31.00, 30.00)
            controlPoint1: CGPoint::new(37.00, 27.31)
            controlPoint2: CGPoint::new(34.31, 30.00)
        ];
    }
    path.addLineToPoint(CGPoint::new(29.12, 30.00));
    path.addLineToPoint(CGPoint::new(25.62, 30.00));
    path.addLineToPoint(CGPoint::new(3.77, 30.00));
    unsafe {
        let _: () = msg_send![
            &path,
            addCurveToPoint: CGPoint::new(3.24, 29.92)
            controlPoint1: CGPoint::new(3.59, 30.00)
            controlPoint2: CGPoint::new(3.41, 29.98)
        ];
    }
    unsafe {
        let _: () = msg_send![
            &path,
            addCurveToPoint: CGPoint::new(2.13, 28.91)
            controlPoint1: CGPoint::new(2.74, 29.77)
            controlPoint2: CGPoint::new(2.33, 29.40)
        ];
    }
    unsafe {
        let _: () = msg_send![
            &path,
            addCurveToPoint: CGPoint::new(2.00, 28.33)
            controlPoint1: CGPoint::new(2.05, 28.73)
            controlPoint2: CGPoint::new(2.01, 28.53)
        ];
    }
    unsafe {
        let _: () = msg_send![
            &path,
            addCurveToPoint: CGPoint::new(2.07, 27.74)
            controlPoint1: CGPoint::new(1.99, 28.12)
            controlPoint2: CGPoint::new(2.02, 27.93)
        ];
    }
    unsafe {
        let _: () = msg_send![
            &path,
            addCurveToPoint: CGPoint::new(3.09, 26.63)
            controlPoint1: CGPoint::new(2.22, 27.24)
            controlPoint2: CGPoint::new(2.60, 26.83)
        ];
    }
    unsafe {
        let _: () = msg_send![
            &path,
            addCurveToPoint: CGPoint::new(3.61, 26.51)
            controlPoint1: CGPoint::new(3.25, 26.56)
            controlPoint2: CGPoint::new(3.43, 26.52)
        ];
    }
    path.addLineToPoint(CGPoint::new(25.70, 24.53));
    unsafe {
        let _: () = msg_send![
            &path,
            addCurveToPoint: CGPoint::new(26.50, 23.65)
            controlPoint1: CGPoint::new(26.15, 24.49)
            controlPoint2: CGPoint::new(26.50, 24.11)
        ];
    }
    unsafe {
        let _: () = msg_send![
            &path,
            addCurveToPoint: CGPoint::new(26.24, 23.03)
            controlPoint1: CGPoint::new(26.50, 23.42)
            controlPoint2: CGPoint::new(26.41, 23.19)
        ];
    }
    path.addLineToPoint(CGPoint::new(23.81, 20.60));
    unsafe {
        let _: () = msg_send![
            &path,
            addCurveToPoint: CGPoint::new(21.25, 14.41)
            controlPoint1: CGPoint::new(22.17, 18.96)
            controlPoint2: CGPoint::new(21.25, 16.73)
        ];
    }
    path.addLineToPoint(CGPoint::new(21.25, 11.93));
    path.addLineToPoint(CGPoint::new(21.25, 8.81));
    path.closePath();
    path.moveToPoint(CGPoint::new(30.00, 5.95));
    unsafe {
        let _: () = msg_send![
            &path,
            addCurveToPoint: CGPoint::new(30.00, 5.94)
            controlPoint1: CGPoint::new(30.00, 5.95)
            controlPoint2: CGPoint::new(30.00, 5.94)
        ];
    }
    unsafe {
        let _: () = msg_send![
            &path,
            addCurveToPoint: CGPoint::new(30.00, 5.92)
            controlPoint1: CGPoint::new(30.00, 5.93)
            controlPoint2: CGPoint::new(30.00, 5.93)
        ];
    }
    path.addLineToPoint(CGPoint::new(30.00, 5.95));
    path.closePath();
    path.moveToPoint(CGPoint::new(29.93, 6.36));
    path.addLineToPoint(CGPoint::new(27.39, 5.72));
    unsafe {
        let _: () = msg_send![
            &path,
            addCurveToPoint: CGPoint::new(27.38, 5.94)
            controlPoint1: CGPoint::new(27.38, 5.80)
            controlPoint2: CGPoint::new(27.38, 5.87)
        ];
    }
    unsafe {
        let _: () = msg_send![
            &path,
            addCurveToPoint: CGPoint::new(28.69, 7.25)
            controlPoint1: CGPoint::new(27.38, 6.66)
            controlPoint2: CGPoint::new(27.96, 7.25)
        ];
    }
    unsafe {
        let _: () = msg_send![
            &path,
            addCurveToPoint: CGPoint::new(29.93, 6.36)
            controlPoint1: CGPoint::new(29.27, 7.25)
            controlPoint2: CGPoint::new(29.75, 6.88)
        ];
    }
    path.closePath();
    path.moveToPoint(CGPoint::new(9.16, 8.37));
    unsafe {
        let _: () = msg_send![
            &path,
            addCurveToPoint: CGPoint::new(12.36, 8.15)
            controlPoint1: CGPoint::new(10.05, 7.58)
            controlPoint2: CGPoint::new(11.37, 7.49)
        ];
    }
    path.addLineToPoint(CGPoint::new(19.50, 12.90));
    path.addLineToPoint(CGPoint::new(19.50, 14.41));
    unsafe {
        let _: () = msg_send![
                &path,
                addCurveToPoint: CGPoint::new(20.81, 19.49)
                controlPoint1: CGPoint::new(19.50, 16.20)
                controlPoint2: CGPoint::new(19.96, 17.95)
        ];
    }
    path.addLineToPoint(CGPoint::new(8.12, 19.49));
    unsafe {
        let _: () = msg_send![
            &path,
            addCurveToPoint: CGPoint::new(7.30, 18.93)
            controlPoint1: CGPoint::new(7.76, 19.49)
            controlPoint2: CGPoint::new(7.43, 19.26)
        ];
    }
    unsafe {
        let _: () = msg_send![
            &path,
            addCurveToPoint: CGPoint::new(7.56, 17.96)
            controlPoint1: CGPoint::new(7.18, 18.59)
            controlPoint2: CGPoint::new(7.28, 18.20)
        ];
    }
    path.addLineToPoint(CGPoint::new(11.35, 14.70));
    path.addLineToPoint(CGPoint::new(3.01, 15.99));
    unsafe {
        let _: () = msg_send![
            &path,
            addCurveToPoint: CGPoint::new(2.08, 15.50)
            controlPoint1: CGPoint::new(2.62, 16.05)
            controlPoint2: CGPoint::new(2.25, 15.85)
        ];
    }
    unsafe {
        let _: () = msg_send![
                &path,
                addCurveToPoint: CGPoint::new(2.29, 14.47)
                controlPoint1: CGPoint::new(1.92, 15.15)
                controlPoint2: CGPoint::new(2.00, 14.73)
        ];
    }
    path.addLineToPoint(CGPoint::new(9.16, 8.37));
    path.closePath();
    path
}

/// Returns the path for the menu icon (Hamburger)
pub fn menu_path() -> Retained<UIBezierPath> {
    let path = UIBezierPath::bezierPath();

    // Top bar
    path.moveToPoint(CGPoint::new(4.0, 12.0));
    path.addLineToPoint(CGPoint::new(20.0, 12.0));

    // Middle bar
    path.moveToPoint(CGPoint::new(4.0, 6.0));
    path.addLineToPoint(CGPoint::new(20.0, 6.0));

    // Bottom bar
    path.moveToPoint(CGPoint::new(4.0, 18.0));
    path.addLineToPoint(CGPoint::new(20.0, 18.0));

    path
}
