use specs::{Component, storage::*};
use nalgebra as na;

pub type Point2 = na::Point2<f32>;
pub type Vector2 = na::Vector2<f32>;

#[derive(Copy, Clone, Component)]
#[storage(DenseVecStorage)]
pub struct Pos(pub Point2);

#[derive(Copy, Clone, Component)]
#[storage(DenseVecStorage)]
pub struct Vel(pub Vector2);

#[derive(Copy, Clone, Component)]
#[storage(DenseVecStorage)]
pub struct Acc(pub Vector2);