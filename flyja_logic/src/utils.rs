use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, Neg, Sub, SubAssign};
pub trait MapUnit:
    Copy
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + SubAssign
    + Div<Output = Self>
    + Sum<Self>
    + PartialOrd
{
    fn zero() -> Self;
    fn two() -> Self;
    fn to_f32(&self) -> f32;
    fn from_f32(val: f32) -> Self;
    fn mul_f32(&self, val: f32) -> Self;
}

pub trait MinusAbleMatUnit: MapUnit + Neg<Output = Self> {}

macro_rules! impl_unit {
    ($Type:ident, $value:expr, $value_2:expr) => {
        impl MapUnit for $Type {
            fn zero() -> Self {
                $value
            }
            fn two() -> Self {
                $value_2
            }
            fn to_f32(&self) -> f32 {
                *self as f32
            }
            fn from_f32(val: f32) -> Self {
                val as $Type
            }
            fn mul_f32(&self, val: f32) -> Self {
                ((*self as f32) * val) as $Type
            }
        }
    };
}
macro_rules! impl_minus_able_unit {
    ($Type:ident, $value:expr, $value_2:expr) => {
        impl_unit!($Type, $value, $value_2);
        impl MinusAbleMatUnit for $Type {}
    };
}
impl_minus_able_unit!(f32, 0., 2.);
impl_minus_able_unit!(i32, 0, 2);
impl_unit!(u32, 0, 2);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Size<T = f32> {
    pub width: T,
    pub height: T,
}

impl<T: MapUnit> Add for Size<T> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            width: self.width + other.width,
            height: self.height + other.height,
        }
    }
}

impl<T: MapUnit> Sub for Size<T> {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            width: self.width - other.width,
            height: self.height - other.height,
        }
    }
}

impl<T: MapUnit> Size<T> {
    pub fn split_h(&self, pieces: T) -> Self {
        Self {
            width: self.width / pieces,
            ..*self
        }
    }
    pub fn split_v(&self, pieces: T) -> Self {
        Self {
            height: self.height / pieces,
            ..*self
        }
    }
    pub fn split(&self, pieces: T, direction: Direction) -> Self {
        match direction {
            Direction::Left | Direction::Right => self.split_h(pieces),
            Direction::Top | Direction::Bottom => self.split_v(pieces),
        }
    }

    pub fn percent_h(&self, percent: f32) -> Self {
        Self {
            width: self.width.mul_f32(percent),
            height: self.height,
        }
    }
    pub fn percent_v(&self, percent: f32) -> Self {
        Self {
            width: self.width,
            height: self.height.mul_f32(percent),
        }
    }
    pub fn percent(&self, percent: f32, way: InsertWay) -> Self {
        match way {
            InsertWay::Horizontal => self.percent_h(percent),
            InsertWay::Vertical => self.percent_v(percent),
        }
    }
}

impl Size<f32> {
    pub fn whole() -> Self {
        Self {
            width: 1.,
            height: 1.,
        }
    }
    /// This compute the change of the percent
    pub fn change_expand(&self, way: InsertWay) -> Self {
        match way {
            InsertWay::Horizontal => Self {
                width: self.width,
                height: 0.,
            },
            InsertWay::Vertical => Self {
                width: 0.,
                height: self.height,
            },
        }
    }
}

// We use Size<f32> to show the percentage of the size in the upper element
pub type Percentage = Size<f32>;

impl<T: MapUnit> AddAssign for Size<T> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
impl<T: MapUnit> SubAssign for Size<T> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl<T: MapUnit> Size<T> {
    pub fn zero() -> Self {
        Self {
            width: T::zero(),
            height: T::zero(),
        }
    }
}

impl<T: MinusAbleMatUnit> Size<T> {
    /// There is not minus width or height in element
    /// so every time we apply drag, we need to take care about it
    pub fn size_legal(&self) -> bool {
        let Self { width, height } = *self;
        width >= T::zero() || height >= T::zero()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Position<T = f32> {
    pub x: T,
    pub y: T,
}

impl<T: MapUnit> Add for Position<T> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T: MapUnit> AddAssign for Position<T> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<T: MapUnit> Sub for Position<T> {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
impl<T: MapUnit> SubAssign for Position<T> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl<T: MapUnit> Position<T> {
    pub fn zero() -> Self {
        Self {
            x: T::zero(),
            y: T::zero(),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SizeAndPos<T = f32> {
    pub size: Size<T>,
    pub position: Position<T>,
}

impl<T: MapUnit> Add for SizeAndPos<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            size: self.size + rhs.size,
            position: self.position + rhs.position,
        }
    }
}

impl<T: MapUnit> AddAssign for SizeAndPos<T> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<T: MapUnit> Sub for SizeAndPos<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            size: self.size - rhs.size,
            position: self.position - rhs.position,
        }
    }
}

impl<T: MapUnit> SubAssign for SizeAndPos<T> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InsertWay {
    Vertical,
    #[default]
    Horizontal,
}

impl InsertWay {
    pub(crate) fn fit_direction(&self, direction: Direction) -> bool {
        match direction {
            Direction::Left | Direction::Right => *self == InsertWay::Horizontal,
            Direction::Top | Direction::Bottom => *self == InsertWay::Vertical,
        }
    }
}

impl From<InsertWay> for Direction {
    fn from(value: InsertWay) -> Self {
        match value {
            InsertWay::Vertical => Self::Bottom,
            InsertWay::Horizontal => Self::Right,
        }
    }
}

impl<T: MapUnit> SizeAndPos<T> {
    fn top(&mut self) -> Self {
        let width = self.size.width;
        let height = self.size.height / T::two();
        self.size.width = width;
        self.size.height = height;
        let y = self.position.y;
        self.position.y += height;
        Self {
            size: Size { width, height },
            position: Position {
                x: self.position.x,
                y,
            },
        }
    }
    fn bottom(&mut self) -> Self {
        let width = self.size.width;
        let height = self.size.height / T::two();
        self.size.width = width;
        self.size.height = height;
        let y = self.position.y + height;
        Self {
            size: Size { width, height },
            position: Position {
                x: self.position.x,
                y,
            },
        }
    }
    fn left(&mut self) -> Self {
        let width = self.size.width / T::two();
        let height = self.size.height;
        self.size.width = width;
        self.size.height = height;
        let x = self.position.x;
        self.position.x += width;
        Self {
            size: Size { width, height },
            position: Position {
                x,
                y: self.position.y,
            },
        }
    }
    fn right(&mut self) -> Self {
        let width = self.size.width / T::two();
        let height = self.size.height;
        self.size.width = width;
        self.size.height = height;
        let x = self.position.x + width;
        Self {
            size: Size { width, height },
            position: Position {
                x,
                y: self.position.y,
            },
        }
    }
    pub fn split(&mut self, direction: Direction) -> Self {
        match direction {
            Direction::Left => self.left(),
            Direction::Right => self.right(),
            Direction::Top => self.top(),
            Direction::Bottom => self.bottom(),
        }
    }
}
impl<T: MinusAbleMatUnit> SizeAndPos<T> {
    /// Apply every drag change to drag, we need check if it is illegal
    pub fn size_legal(&self) -> bool {
        self.size.size_legal()
    }
    /// Compute the change of drag
    /// The `transfer` means the transference on the map
    /// For example, if you drag it from up to bottom, for 10, the transfer is +10, direction Top
    /// If it is from bottom to up, it is -10 direction Bottom
    ///
    /// Same logic in Left and Right
    pub fn drag_change(transfer: T, direction: Direction) -> Self {
        match direction {
            Direction::Left => Self {
                size: Size {
                    width: -transfer,
                    height: T::zero(),
                },
                position: Position {
                    x: transfer,
                    y: T::zero(),
                },
            },
            Direction::Right => Self {
                size: Size {
                    width: transfer,
                    height: T::zero(),
                },
                position: Position {
                    x: T::zero(),
                    y: T::zero(),
                },
            },
            Direction::Top => Self {
                size: Size {
                    width: T::zero(),
                    height: -transfer,
                },
                position: Position {
                    x: T::zero(),
                    y: transfer,
                },
            },
            Direction::Bottom => Self {
                size: Size {
                    width: T::zero(),
                    height: transfer,
                },
                position: Position {
                    x: T::zero(),
                    y: T::zero(),
                },
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Top,
    Bottom,
}

impl Direction {
    pub(crate) fn is_end(&self) -> bool {
        match self {
            Direction::Left | Direction::Top => false,
            Direction::Right | Direction::Bottom => true,
        }
    }
    /// get the opposite value
    pub fn opposite(&self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Top,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

impl Direction {
    pub fn expend_way(insert_way: InsertWay, start: bool) -> Self {
        match (insert_way, start) {
            (InsertWay::Horizontal, true) => Self::Left,
            (InsertWay::Horizontal, false) => Self::Right,
            (InsertWay::Vertical, true) => Self::Top,
            (InsertWay::Vertical, false) => Self::Bottom,
        }
    }
}

impl<T: MinusAbleMatUnit> SizeAndPos<T> {
    pub fn change_disappear(&self, direction: Direction) -> Self {
        match direction {
            Direction::Top => Self {
                position: Position {
                    x: T::zero(),
                    y: -self.size.height,
                },
                size: Size {
                    width: T::zero(),
                    height: self.size.height,
                },
            },
            Direction::Bottom => Self {
                position: Position::zero(),
                size: Size {
                    width: T::zero(),
                    height: self.size.height,
                },
            },
            Direction::Left => Self {
                position: Position {
                    x: -self.size.width,
                    y: T::zero(),
                },
                size: Size {
                    width: self.size.width,
                    height: T::zero(),
                },
            },
            Direction::Right => Self {
                position: Position::zero(),
                size: Size {
                    width: self.size.width,
                    height: T::zero(),
                },
            },
        }
    }
}
