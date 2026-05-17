use std::ops::Deref;

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Px<T>(T);

impl<T> Deref for Px<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Px<u32>> for Px<f32> {
    fn from(value: Px<u32>) -> Self {
        Px(*value as f32)
    }
}

impl From<Px<f32>> for Px<u32> {
    fn from(value: Px<f32>) -> Self {
        Px(value.round().max(0.0) as u32)
    }
}

#[derive(Clone, Copy, Default)]
pub struct Size<T> {
    width: Px<T>,
    height: Px<T>,
}

impl<T: Clone + Copy> Size<T> {
    pub fn new(width: T, height: T) -> Self {
        Self {
            width: Px(width),
            height: Px(height),
        }
    }

    pub fn width(&self) -> T {
        *self.width
    }

    pub fn height(&self) -> T {
        *self.height
    }
}

impl From<Size<u32>> for Size<f32> {
    fn from(value: Size<u32>) -> Self {
        Self {
            width: value.width.into(),
            height: value.height.into(),
        }
    }
}

impl From<Size<f32>> for Size<u32> {
    fn from(value: Size<f32>) -> Self {
        Self {
            width: value.width.into(),
            height: value.height.into(),
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Offset<T> {
    x: Px<T>,
    y: Px<T>,
}

impl<T: Clone + Copy> Offset<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x: Px(x), y: Px(y) }
    }

    pub fn x(&self) -> T {
        *self.x
    }

    pub fn y(&self) -> T {
        *self.y
    }
}

impl From<Offset<u32>> for Offset<f32> {
    fn from(value: Offset<u32>) -> Self {
        Self {
            x: value.x.into(),
            y: value.y.into(),
        }
    }
}

impl From<Offset<f32>> for Offset<u32> {
    fn from(value: Offset<f32>) -> Self {
        Self {
            x: value.x.into(),
            y: value.y.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn px_default_is_zero() {
        let p: Px<u32> = Px::default();
        assert_eq!(*p, 0);
    }

    #[test]
    fn px_deref() {
        let p = Px(42u32);
        assert_eq!(*p, 42);
    }

    #[test]
    fn px_from_u32_to_f32() {
        let p: Px<u32> = Px(42);
        let pf: Px<f32> = p.into();
        assert!((*pf - 42.0f32).abs() < f32::EPSILON);
    }

    #[test]
    fn px_from_f32_to_u32() {
        let pf = Px(3.7f32);
        let p: Px<u32> = pf.into();
        assert_eq!(*p, 4);
    }

    #[test]
    fn px_from_f32_to_u32_negative_clamped() {
        let pf = Px(-5.0f32);
        let p: Px<u32> = pf.into();
        assert_eq!(*p, 0);
    }

    #[test]
    fn px_clone_and_eq() {
        let a = Px(42u32);
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn size_new_and_accessors() {
        let s = Size::new(100u32, 200u32);
        assert_eq!(s.width(), 100);
        assert_eq!(s.height(), 200);
    }

    #[test]
    fn size_default() {
        let s = Size::<u32>::default();
        assert_eq!(s.width(), 0);
        assert_eq!(s.height(), 0);
    }

    #[test]
    fn size_from_u32_to_f32() {
        let s32 = Size::new(100u32, 200u32);
        let sf: Size<f32> = s32.into();
        assert!((sf.width() - 100.0f32).abs() < f32::EPSILON);
        assert!((sf.height() - 200.0f32).abs() < f32::EPSILON);
    }

    #[test]
    fn size_from_f32_to_u32() {
        let sf = Size::new(100.7f32, 200.3f32);
        let s32: Size<u32> = sf.into();
        assert_eq!(s32.width(), 101);
        assert_eq!(s32.height(), 200);
    }

    #[test]
    fn size_clone() {
        let s = Size::new(10u32, 20u32);
        let s2 = s;
        assert_eq!(s2.width(), 10);
        assert_eq!(s2.height(), 20);
    }

    #[test]
    fn offset_new_and_accessors() {
        let o = Offset::new(5u32, 10u32);
        assert_eq!(o.x(), 5);
        assert_eq!(o.y(), 10);
    }

    #[test]
    fn offset_default() {
        let o = Offset::<u32>::default();
        assert_eq!(o.x(), 0);
        assert_eq!(o.y(), 0);
    }

    #[test]
    fn offset_from_u32_to_f32() {
        let o32 = Offset::new(5u32, 10u32);
        let of: Offset<f32> = o32.into();
        assert!((of.x() - 5.0f32).abs() < f32::EPSILON);
        assert!((of.y() - 10.0f32).abs() < f32::EPSILON);
    }

    #[test]
    fn offset_from_f32_to_u32() {
        let of = Offset::new(5.7f32, 10.2f32);
        let o32: Offset<u32> = of.into();
        assert_eq!(o32.x(), 6);
        assert_eq!(o32.y(), 10);
    }

    #[test]
    fn offset_clone() {
        let o = Offset::new(5u32, 10u32);
        let o2 = o;
        assert_eq!(o2.x(), 5);
        assert_eq!(o2.y(), 10);
    }
}
