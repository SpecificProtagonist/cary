#[derive(Debug, Clone, Copy)]
pub struct Vec2(pub f32, pub f32);

impl Vec2 {
    pub fn zero() -> Self {
        Vec2(0.0, 0.0)
    }

    pub fn len(self) -> f32 {
        (self.0*self.0 + self.1+self.1).sqrt()
    }

    pub fn dist(self, other: Vec2) -> f32 {
        (self-other).len()
    }

    pub fn norm(self) -> Self {
        self / self.len()
    }

    /// Clockwise rotation
    pub fn rotated(self, turns: f32) -> Self {
        Vec2(turns.cos() * self.0 + turns.sin() * self.1,
             turns.cos() * self.1 - turns.sin() * self.0)
    }

    pub fn lerp(self, destination: Vec2, progress: f32) -> Self {
        self + (destination - self) * progress.min(1.0).max(0.0) // clamp is still unstable
    }
}

impl std::ops::Add<Vec2> for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Self::Output {
        Vec2(self.0+rhs.0, self.1+rhs.1)
    }
}

impl std::ops::AddAssign<Vec2> for Vec2 {
    fn add_assign(&mut self, rhs: Vec2) {
        *self = Vec2(self.0+rhs.0, self.1+rhs.1)
    }
}

impl std::ops::Sub<Vec2> for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Vec2) -> Self::Output {
        Vec2(self.0-rhs.0, self.1-rhs.1)
    }
}

impl std::ops::SubAssign<Vec2> for Vec2 {
    fn sub_assign(&mut self, rhs: Vec2) {
        *self = Vec2(self.0-rhs.0, self.1-rhs.1)
    }
}

impl std::ops::Mul<Vec2> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: Vec2) -> Self::Output {
        Vec2(self.0*rhs.0, self.1*rhs.1)
    }
}

impl std::ops::Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: f32) -> Self::Output {
        Vec2(self.0*rhs, self.1*rhs)
    }
}

impl std::ops::MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, rhs: f32) {
        *self = Vec2(self.0 * rhs, self.1 * rhs);
    }
}

impl std::ops::Mul<Vec2> for f32 {
    type Output = Vec2;
    fn mul(self, rhs: Vec2) -> Self::Output {
        Vec2(self*rhs.0, self*rhs.1)
    }
}

impl std::ops::Div<f32> for Vec2 {
    type Output = Vec2;
    fn div(self, rhs: f32) -> Self::Output {
        Vec2(self.0/rhs, self.1/rhs)
    }
}

impl std::ops::DivAssign<f32> for Vec2 {
    fn div_assign(&mut self, rhs: f32) {
        *self = Vec2(self.0 / rhs, self.1 / rhs);
    }
}



/// Axis-aligned bounding box
#[derive(Debug, Copy, Clone)]
pub struct Bounds {
    pub min: Vec2,
    pub max: Vec2
}

impl Bounds {
    pub fn around(center: Vec2, size: Vec2) -> Self {
        Bounds {
            min: center - size/2.0,
            max: center + size/2.0
        }
    }

    pub fn center(self) -> Vec2 {
        (self.max + self.min) / 2.0
    }

    pub fn size(self) -> Vec2 {
        self.max - self.min
    }

    pub fn moved(self, vec: Vec2) -> Self {
        Bounds {
            min: self.min + vec,
            max: self.max + vec
        }
    }

    pub fn overlapps(self, other: Self) -> bool {
        (self.min.0 < other.max.0) & (self.max.0 > other.min.0)
        &
        (self.min.1 < other.max.1) & (self.max.1 > other.min.1)
    }

    pub fn check_move_against(self, movement: Vec2, obstacle: Bounds) -> Vec2 {
        let eps = 0.001;
        let move_x = if movement.0 > eps {
            let distance = (obstacle.min.0 - self.max.0 - eps).max(movement.0).max(0.0);
            Vec2(distance, movement.1 * distance/movement.0)
        } else if movement.0 < -eps {
            let distance = (self.min.0 - obstacle.max.0 - eps).max(-movement.1).max(0.0);
            Vec2(-distance, movement.1 * distance/movement.0)
        } else {
            movement
        };
        let move_y = if movement.1 > eps {
            let distance = (obstacle.min.1 - self.max.1 - eps).min(movement.1).max(0.0);
            Vec2(distance, movement.0 * distance/movement.1)
        } else if movement.1 < -eps {
            let distance = (self.min.1 - obstacle.max.1 - eps).min(-movement.0).max(0.0);
            Vec2(-distance, movement.0 * distance/movement.1)
        } else {
            movement
        };
        println!("Bounds: {:?}, attempt: {:?}, obstacle: {:?}  --> move_x: {:?}, move_y: {:?}",
            self, movement, obstacle, move_x, move_y);
        if move_x.len() < move_y.len() {
            move_x
        } else {
            move_y
        }
    }
}