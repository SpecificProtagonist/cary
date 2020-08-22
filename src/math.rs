#[derive(Debug, Clone, Copy)]
pub struct Vec2(pub f32, pub f32);

impl Vec2 {
    pub fn zero() -> Self {
        Vec2(0.0, 0.0)
    }

    pub fn len(self) -> f32 {
        (self.0*self.0 + self.1*self.1).sqrt()
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

impl std::ops::Add<Vec2> for Bounds {
    type Output = Bounds;
    fn add(self, rhs: Vec2) -> Self::Output {
        Bounds {
            min: self.min + rhs,
            max: self.max + rhs
        }
    }
}

impl std::ops::AddAssign<Vec2> for Bounds {
    fn add_assign(&mut self, rhs: Vec2) {
        self.min += rhs;
        self.max += rhs;
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

    pub fn overlapps(self, other: Self) -> bool {
        (self.min.0 < other.max.0) & (self.max.0 > other.min.0)
        &
        (self.min.1 < other.max.1) & (self.max.1 > other.min.1)
    }

    pub fn contains(self, pos: Vec2) -> bool {
        (pos.0  > self.min.0) & (pos.0 < self.max.0) & (pos.1 > self.min.1) & (pos.1 < self.max.1)
    }

    /*
    pub fn check_move_against(self, movement: Vec2, obstacle: Bounds) -> Vec2 {
        let eps = 0.0005;
        let move_x = if (movement.0 > eps) & (self.max.0 < obstacle.min.0) {
            let distance = (obstacle.min.0 - self.max.0 - eps).min(movement.0).max(0.0);
            Vec2(distance, movement.1 * distance/movement.0)
        } else if (movement.0 < -eps) & (self.min.0 > obstacle.max.1) {
            let distance = (self.min.0 - obstacle.max.0 - eps).min(-movement.1).max(0.0);
            Vec2(-distance, movement.1 * distance/movement.0)
        } else {
            movement
        };
        let move_y = if (movement.1 > eps) & (self.max.1 < obstacle.min.1) {
            let distance = (obstacle.min.1 - self.max.1 - eps).min(movement.1).max(0.0);
            //println!("movement: {}, self.max.1: {}, obstacle.min.1: {} -> min: {} -> distance: {}",
            //    movement.1, self.max.1, obstacle.min.1, (obstacle.min.1 - self.max.1 - eps).min(movement.1), distance);
            Vec2(movement.0 * distance/movement.1, distance)
        } else if (movement.1 < -eps) & (self.min.1 > obstacle.max.1) {
            let distance = (self.min.1 - obstacle.max.1 - eps).min(-movement.1).max(0.0);
            //println!("diff: {:?} --> {:?}:   {:?};   max: {:?}, distance: {:?}",
            //    self.min.1 - obstacle.max.1, self.min.1 > obstacle.max.1, movement.1, (self.min.1 - obstacle.max.1 - eps).min(-movement.1), distance);
            Vec2(movement.0 * distance/movement.1, -distance)
        } else {
            movement
        };
        /*if (move_x.len() < movement.len() - eps) | (move_y.len() < movement.len() - eps) {
            println!("Collision: {:?}", self.center());
        }*/
        //println!("Bounds: {:?}, attempt: {:?}, obstacle: {:?}  --> move_x: {:?}, move_y: {:?}",
        //    self, movement, obstacle, move_x, move_y);
        if move_y.len() < move_x.len() - eps {
            let bounds = self.moved(move_y);
            if (bounds.min.0 <= obstacle.max.0) & (obstacle.min.0 <= bounds.max.0) {
                println!("Collision y 1");
                move_y
            } else {
                // No collision if moved by move_y
                let bounds = self.moved(move_x);
                if (bounds.min.1 <= obstacle.max.1) & (obstacle.min.1 <= bounds.max.1) {
                    println!("Collision x 1");
                    move_x
                } else {
                    movement
                }
            }
        } else if move_x.len() < move_y.len() - eps {
            let bounds = self.moved(move_x);
            if (bounds.min.1 <= obstacle.max.1) & (obstacle.min.1 <= bounds.max.1) {
                println!("Collision x 2");
                move_x
            } else {
                // No collision if moved by move_x
                let bounds = self.moved(move_y);
                if (bounds.min.0 <= obstacle.max.0) & (obstacle.min.0 <= bounds.max.0) {
                    println!("Collision y 2");
                    move_y
                } else {
                    movement
                }
            }
        } else {
            movement
        }
    }
    */
}