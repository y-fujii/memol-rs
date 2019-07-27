// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use crate::misc;
use std::*;

// irreducible && x >= 0, x = 0 is interpreted as +0.
#[derive(Copy, Clone, Debug)]
pub struct Ratio {
    pub y: i64,
    pub x: i64,
}

impl From<i64> for Ratio {
    fn from(n: i64) -> Self {
        Ratio { y: n, x: 1 }
    }
}

impl cmp::PartialEq for Ratio {
    fn eq(&self, other: &Self) -> bool {
        self.y * other.x == other.y * self.x
    }
}

impl cmp::Eq for Ratio {}

impl PartialOrd for Ratio {
    fn partial_cmp(&self, other: &Ratio) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::Ord for Ratio {
    fn cmp(&self, other: &Ratio) -> cmp::Ordering {
        let lhs = self.y * other.x;
        let rhs = other.y * self.x;
        lhs.cmp(&rhs)
    }
}

impl hash::Hash for Ratio {
    fn hash<T: hash::Hasher>(&self, state: &mut T) {
        self.y.hash(state);
        self.x.hash(state);
    }
}

impl ops::Neg for Ratio {
    type Output = Ratio;

    fn neg(self) -> Ratio {
        Ratio { y: -self.y, x: self.x }
    }
}

impl<T: Into<Ratio>> ops::Add<T> for Ratio {
    type Output = Ratio;

    fn add(self, other: T) -> Ratio {
        let other: Ratio = other.into();
        Ratio::new(self.y * other.x + self.x * other.y, self.x * other.x)
    }
}

impl<T: Into<Ratio>> ops::Sub<T> for Ratio {
    type Output = Ratio;

    fn sub(self, other: T) -> Ratio {
        let other: Ratio = other.into();
        Ratio::new(self.y * other.x - self.x * other.y, self.x * other.x)
    }
}

impl<T: Into<Ratio>> ops::Mul<T> for Ratio {
    type Output = Ratio;

    fn mul(self, other: T) -> Ratio {
        let other: Ratio = other.into();
        Ratio::new(self.y * other.y, self.x * other.x)
    }
}

impl<T: Into<Ratio>> ops::Div<T> for Ratio {
    type Output = Ratio;

    fn div(self, other: T) -> Ratio {
        let other: Ratio = other.into();
        Ratio::new(self.y * other.x, self.x * other.y)
    }
}

impl ops::Mul<Ratio> for i64 {
    type Output = Ratio;

    fn mul(self, other: Ratio) -> Ratio {
        Ratio::new(other.y * self, other.x)
    }
}

impl ops::Div<Ratio> for i64 {
    type Output = Ratio;

    fn div(self, other: Ratio) -> Ratio {
        Ratio::new(other.x * self, other.y)
    }
}

impl Ratio {
    pub fn new(y: i64, x: i64) -> Ratio {
        let t = misc::gcd(y, x);
        Ratio { y: y / t, x: x / t }
    }

    pub fn zero() -> Ratio {
        Ratio { y: 0, x: 1 }
    }

    pub fn one() -> Ratio {
        Ratio { y: 1, x: 1 }
    }

    pub fn inf() -> Ratio {
        Ratio { y: 1, x: 0 }
    }

    pub fn floor(self) -> i64 {
        misc::idiv(self.y, self.x)
    }

    pub fn ceil(self) -> i64 {
        misc::idiv(self.y + self.x - 1, self.x)
    }

    pub fn round(self) -> i64 {
        misc::idiv(self.y * 2 + self.x, self.x * 2)
    }

    pub fn to_float(self) -> f64 {
        self.y as f64 / self.x as f64
    }
}
