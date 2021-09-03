use std::cmp::Ordering;
use std::convert::TryFrom;
use std::ops::{Add, Div, Mul, Sub};

use num::{Float, Num, NumCast};

#[macro_export]
macro_rules! cast {
    ($num: expr) => {
        num::cast($num).unwrap()
    };
}

#[derive(Debug, Copy, Clone)]
pub struct Point<T: Num + NumCast + PartialOrd + Copy> {
    pub x: T,
    pub y: T,
}

impl<T: Float> Point<T> {
    pub fn is_irregular(&self) -> bool {
        self.x.is_nan() || self.y.is_nan() || self.x.is_infinite() || self.y.is_infinite()
    }
}

impl<T: Num + NumCast + PartialOrd + Copy> Add for Point<T> {
    type Output = Point<T>;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T: Num + NumCast + PartialOrd + Copy> Sub for Point<T> {
    type Output = Point<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T: Num + NumCast + PartialOrd + Copy> Mul<T> for Point<T> {
    type Output = Point<T>;

    fn mul(self, rhs: T) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<T: Num + NumCast + PartialOrd + Copy> Div<T> for Point<T> {
    type Output = Point<T>;

    fn div(self, rhs: T) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl<T: Num + NumCast + PartialOrd + Copy> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T: Num + NumCast + PartialOrd + Copy> From<Point<T>> for (T, T) {
    fn from(p: Point<T>) -> Self {
        (p.x, p.y)
    }
}

impl<T: Num + NumCast + PartialOrd + Copy> From<&Point<T>> for (T, T) {
    fn from(p: &Point<T>) -> Self {
        (p.x, p.y)
    }
}

impl<T: Num + NumCast + PartialOrd + Copy, U: Num + NumCast + PartialOrd + Copy> From<&Point<T>>
    for Point<U>
{
    fn from(v: &Point<T>) -> Self {
        Self {
            x: cast!(v.x),
            y: cast!(v.y),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Rectangle<T: Num + NumCast + PartialOrd + Copy> {
    pub x: T,
    pub y: T,
    pub w: T,
    pub h: T,
}

impl<T: Float> Rectangle<T> {
    pub fn is_irregular(&self) -> bool {
        self.x.is_nan()
            || self.y.is_nan()
            || self.w.is_nan()
            || self.h.is_nan()
            || self.x.is_infinite()
            || self.y.is_infinite()
            || self.w.is_infinite()
            || self.h.is_infinite()
    }
}

impl<T: Num + NumCast + PartialOrd + Copy> Rectangle<T> {
    pub fn new(x: T, y: T, w: T, h: T) -> Self {
        Rectangle { x, y, w, h }
    }
}

impl<T: Num + NumCast + PartialOrd + Copy, U: Num + NumCast + PartialOrd + Copy> From<&Rectangle<T>>
    for Rectangle<U>
{
    fn from(v: &Rectangle<T>) -> Self {
        Self {
            x: cast!(v.x),
            y: cast!(v.y),
            w: cast!(v.w),
            h: cast!(v.h),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ControlPoints<T: Num + NumCast + PartialOrd + Copy> {
    pub p1: Point<T>,
    pub p2: Point<T>,
    pub p3: Point<T>,
    pub p4: Point<T>,
}

impl<T: Float> ControlPoints<T> {
    pub fn is_irregular(&self) -> bool {
        self.p1.is_irregular()
            || self.p2.is_irregular()
            || self.p3.is_irregular()
            || self.p4.is_irregular()
    }
}

impl<T: Num + NumCast + PartialOrd + Copy> Add<Point<T>> for ControlPoints<T> {
    type Output = ControlPoints<T>;

    fn add(self, rhs: Point<T>) -> Self::Output {
        Self {
            p1: self.p1 + rhs,
            p2: self.p2 + rhs,
            p3: self.p3 + rhs,
            p4: self.p4 + rhs,
        }
    }
}

impl<T: Num + NumCast + PartialOrd + Copy> Sub<Point<T>> for ControlPoints<T> {
    type Output = ControlPoints<T>;

    fn sub(self, rhs: Point<T>) -> Self::Output {
        Self {
            p1: self.p1 - rhs,
            p2: self.p2 - rhs,
            p3: self.p3 - rhs,
            p4: self.p4 - rhs,
        }
    }
}

impl<T: Num + NumCast + PartialOrd + Copy> Mul<T> for ControlPoints<T> {
    type Output = ControlPoints<T>;

    fn mul(self, rhs: T) -> Self::Output {
        Self {
            p1: self.p1 * rhs,
            p2: self.p2 * rhs,
            p3: self.p3 * rhs,
            p4: self.p4 * rhs,
        }
    }
}

impl<T: Num + NumCast + PartialOrd + Copy> Div<T> for ControlPoints<T> {
    type Output = ControlPoints<T>;

    fn div(self, rhs: T) -> Self::Output {
        Self {
            p1: self.p1 / rhs,
            p2: self.p2 / rhs,
            p3: self.p3 / rhs,
            p4: self.p4 / rhs,
        }
    }
}

impl<T: Num + NumCast + PartialOrd + Copy> Add<ControlPoints<T>> for ControlPoints<T> {
    type Output = ControlPoints<T>;

    fn add(self, rhs: ControlPoints<T>) -> Self::Output {
        Self {
            p1: self.p1 + rhs.p1,
            p2: self.p2 + rhs.p2,
            p3: self.p3 + rhs.p3,
            p4: self.p4 + rhs.p4,
        }
    }
}

impl<T: Num + NumCast + PartialOrd + Copy> Sub<ControlPoints<T>> for ControlPoints<T> {
    type Output = ControlPoints<T>;

    fn sub(self, rhs: ControlPoints<T>) -> Self::Output {
        Self {
            p1: self.p1 - rhs.p1,
            p2: self.p2 - rhs.p2,
            p3: self.p3 - rhs.p3,
            p4: self.p4 - rhs.p4,
        }
    }
}

impl<T: Num + NumCast + PartialOrd + Copy> ControlPoints<T> {
    pub fn new(p1: Point<T>, p2: Point<T>, p3: Point<T>, p4: Point<T>) -> Self {
        ControlPoints { p1, p2, p3, p4 }
    }
    pub fn cross(&self) -> Point<T> {
        let [(x1, y1), (x3, y3), (x2, y2), (x4, y4)]: [(T, T); 4] = self.into();
        let x1_2 = x1 - x2;
        let x3_4 = x3 - x4;
        let y1_2 = y1 - y2;
        let y3_4 = y3 - y4;
        let x0 =
            (x3_4 * (x2 * y1 - x1 * y2) - x1_2 * (x4 * y3 - x3 * y4)) / (x3_4 * y1_2 - x1_2 * y3_4);
        let y0 =
            (y3_4 * (y2 * x1 - y1 * x2) - y1_2 * (y4 * x3 - y3 * x4)) / (y3_4 * x1_2 - y1_2 * x3_4);
        Point::new(x0, y0)
    }
    pub fn center(&self) -> Point<T> {
        let x_0 = (self.p1.x + self.p3.x) / cast!(2);
        let y_0 = (self.p2.y + self.p4.y) / cast!(2);

        Point::new(x_0, y_0)
    }
    pub fn enlarge(&self, scale: T, use_center: bool) -> Self {
        let origin = if use_center {
            self.center()
        } else {
            self.cross()
        };

        *self + (*self) * scale - origin * scale
    }
    pub fn centralize_y(&self) -> Self {
        let cross = self.cross();
        let center = self.center();

        Self {
            p1: center - cross + self.p1,
            p2: self.p2,
            p3: center - cross + self.p3,
            p4: self.p4,
        }
    }
    #[allow(clippy::match_like_matches_macro)]
    pub fn is_convex(&self) -> Option<bool> {
        let Point { x: _, y: y0 } = self.cross();
        Some(y0.partial_cmp(&self.p2.y)?.is_gt() || y0.partial_cmp(&self.p4.y)?.is_lt())
    }
    //noinspection RsBorrowChecker
    pub fn shift_origin(&self) -> Option<(Point<T>, Point<T>, Self)> {
        let cross = self.cross();

        let left_top = self.p1 - cross + self.p2;
        let right_top = self.p3 - cross + self.p2;
        let left_bottom = self.p1 - cross + self.p4;
        let right_bottom = self.p3 - cross + self.p4;

        let bound_left_top = Point::new(
            poset_min(left_top.x, left_bottom.x)? - cast!(4),
            poset_min(left_top.y, right_top.y)? - cast!(4),
        );
        let bound_right_bottom = Point::new(
            poset_max(right_top.x, right_bottom.x)? + cast!(4),
            poset_max(left_bottom.y, right_bottom.y)? + cast!(4),
        );

        Some((bound_left_top, bound_right_bottom, *self - bound_left_top))
    }
}

impl<T: Num + NumCast + PartialOrd + Copy> From<&Rectangle<T>> for ControlPoints<T> {
    fn from(rect: &Rectangle<T>) -> Self {
        Self {
            p1: Point {
                x: rect.x,
                y: rect.y + rect.h / num::cast(2).unwrap(),
            },
            p2: Point {
                x: rect.x + rect.w / num::cast(2).unwrap(),
                y: rect.y,
            },
            p3: Point {
                x: rect.x + rect.w,
                y: rect.y + rect.h / num::cast(2).unwrap(),
            },
            p4: Point {
                x: rect.x + rect.w / num::cast(2).unwrap(),
                y: rect.y + rect.h,
            },
        }
    }
}

impl<T: Num + NumCast + PartialOrd + Copy, U: Num + NumCast + PartialOrd + Copy>
    From<&ControlPoints<T>> for ControlPoints<U>
{
    fn from(v: &ControlPoints<T>) -> Self {
        Self {
            p1: (&v.p1).into(),
            p2: (&v.p2).into(),
            p3: (&v.p3).into(),
            p4: (&v.p4).into(),
        }
    }
}

impl<T: Num + NumCast + PartialOrd + Copy> TryFrom<&Vec<Point<T>>> for ControlPoints<T> {
    type Error = ();

    fn try_from(value: &Vec<Point<T>>) -> Result<Self, Self::Error> {
        if value.len() != 4 {
            Err(())
        } else {
            Ok(ControlPoints::new(value[0], value[1], value[2], value[3]))
        }
    }
}

impl<T: Num + NumCast + PartialOrd + Copy> TryFrom<Vec<Point<T>>> for ControlPoints<T> {
    type Error = ();

    fn try_from(value: Vec<Point<T>>) -> Result<Self, Self::Error> {
        if value.len() != 4 {
            Err(())
        } else {
            Ok(ControlPoints::new(value[0], value[1], value[2], value[3]))
        }
    }
}

impl<T: Num + NumCast + PartialOrd + Copy> From<&ControlPoints<T>> for [(T, T); 4] {
    fn from(pts: &ControlPoints<T>) -> Self {
        let ControlPoints { p1, p2, p3, p4 } = pts;
        [p1.into(), p2.into(), p3.into(), p4.into()]
    }
}

pub fn user_abs_minus<T: Num + NumCast + PartialOrd + Copy>(m: T, n: T) -> Option<T> {
    m.partial_cmp(&n).map(|ord| {
        if let Ordering::Less = ord {
            n - m
        } else {
            m - n
        }
    })
}

pub fn poset_min<T: PartialOrd>(m: T, n: T) -> Option<T> {
    m.partial_cmp(&n)
        .map(|ord| if let Ordering::Less = ord { m } else { n })
}

pub fn poset_max<T: PartialOrd>(m: T, n: T) -> Option<T> {
    m.partial_cmp(&n)
        .map(|ord| if let Ordering::Greater = ord { m } else { n })
}
