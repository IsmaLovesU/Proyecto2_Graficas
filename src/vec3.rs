use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub const fn splat(value: f32) -> Self {
        Self::new(value, value, value)
    }

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn normalized(self) -> Self {
        let length = self.length();
        if length <= f32::EPSILON {
            self
        } else {
            self * (1.0 / length)
        }
    }

    pub fn reflect(self, normal: Self) -> Self {
        self - normal * (2.0 * self.dot(normal))
    }

    /// Ley de Snell en forma vectorial. `self` es la direccion incidente
    /// (normalizada, viajando hacia la superficie) y `ior` el indice de
    /// refraccion del material del otro lado. Devuelve `None` cuando el
    /// angulo es tan rasante que ocurre reflexion interna total, caso en el
    /// que no existe rayo refractado posible.
    pub fn refract(self, normal: Self, ior: f32) -> Option<Self> {
        let mut cos_i = self.dot(normal).clamp(-1.0, 1.0);
        let (n, eta) = if cos_i < 0.0 {
            // El rayo viene de afuera del material y entra.
            (normal, 1.0 / ior)
        } else {
            // El rayo ya esta dentro y sale: la normal efectiva se invierte.
            cos_i = -cos_i;
            (-normal, ior)
        };

        let k = 1.0 - eta * eta * (1.0 - cos_i * cos_i);
        if k < 0.0 {
            None
        } else {
            Some(self * eta + n * (eta * -cos_i - k.sqrt()))
        }
    }

    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self::Output {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

/// Multiplicacion componente a componente: sirve para tenir un color con el
/// color/intensidad de una luz, o un texel con el tono de un material.
impl Mul<Vec3> for Vec3 {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        Self::new(self.x * other.x, self.y * other.y, self.z * other.z)
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;

    fn div(self, scalar: f32) -> Self::Output {
        Self::new(self.x / scalar, self.y / scalar, self.z / scalar)
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}
