

use std::ops::{Add, AddAssign};

struct Complex<T> {
    re: T,
    im: T
}

impl<T> Add for Complex<T>
where
    T: Add<Output = T>,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Complex {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}


impl<T> AddAssign for Complex<T>
where
    T: AddAssign<T>,
{
    fn add_assign(&mut self, rhs: Complex<T>) {
        self.re += rhs.re;
        self.im += rhs.im;
    }
}


impl<T: PartialEq> PartialEq for Complex<T> {
    fn eq(&self, other: &Complex<T>) -> bool {
        self.re == other.re && self.im == other.im
    }
}

#[test]
pub fn t() {
    let x = Complex { re: 5, im: 2 };
    let y = Complex { re: 2, im: 5 };
    if x == y {
        println!("X=Y")
    } else {
        println!("X!=Y")
    }
}