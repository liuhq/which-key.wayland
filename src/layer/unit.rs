use std::ops::Deref;

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
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
