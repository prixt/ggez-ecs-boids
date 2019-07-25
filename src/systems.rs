use ggez::{Context, graphics::*};
use specs::prelude::*;
use std::sync::Mutex;
use nalgebra as na;
use crate::components::*;
use crate::neighborhood::{get_area, Neighborhood};
use crate::globals::*;

// type Point2 = na::Point2<f32>;
type Vector2 = na::Vector2<f32>;

pub struct VelocitySystem;
impl<'a> System<'a> for VelocitySystem {
	type SystemData = (
		Entities<'a>,
		ReadStorage<'a, Vel>,
		WriteStorage<'a, Pos>,
		WriteExpect<'a, Neighborhood>,
	);

	fn run(&mut self, (entities, velocities, mut positions, neighborhood): Self::SystemData) {
		let nh = Mutex::new(neighborhood);
		(&entities, &velocities, &mut positions).par_join()
			.for_each(|(ent, vel, pos)| {
				let (prev_x, prev_y) = get_area(pos.0, AREA_SIZE, AREA_SIZE);
				pos.0 += vel.0 / 60.0;
				wrap_pos_within_limits(&mut pos.0, 800.0, 600.0);
				let (curr_x, curr_y) = get_area(pos.0, AREA_SIZE, AREA_SIZE);
				if prev_x != curr_x || prev_y != curr_y {
					let mut nh = nh.lock().unwrap();
					nh.remove(prev_x, prev_y, ent.id());
					nh.insert(curr_x, curr_y, ent.id());
				}
			});
	}
}

fn wrap_pos_within_limits(pos: &mut nalgebra::Point2<f32>, screen_width: f32, screen_height: f32) {
	let x = (pos.x % screen_width + screen_width) % screen_width;
	let y = (pos.y % screen_height + screen_height) % screen_height;
	assert!(x >= 0.0 && y >= 0.0, "pos.x:{}, pos.y:{}, x:{}, y:{}", pos.x, pos.y, x, y);
	pos.x = x; pos.y = y;
}

pub struct BoidSystem;
impl<'a> System<'a> for BoidSystem {
	type SystemData = (
		Entities<'a>,
		ReadExpect<'a, Neighborhood>,
		ReadStorage<'a, Pos>,
		ReadStorage<'a, Vel>,
		WriteStorage<'a, Acc>,
	);

	fn run(&mut self, (ent, nh, pos_storage, vel_storage, mut acc_storage): Self::SystemData) {
		use nalgebra::distance_squared;
		(&ent, &pos_storage, &mut acc_storage).par_join()
			.for_each(|(ent, pos, acc)| {
				let (area_x, area_y) = get_area(pos.0, AREA_SIZE, AREA_SIZE);
				let mut bitset = BitSet::new();
				bitset |= nh.get(area_x-1, area_y-1);
				bitset |= nh.get(area_x-1, area_y  );
				bitset |= nh.get(area_x-1, area_y+1);
				bitset |= nh.get(area_x  , area_y-1);
				bitset |= nh.get(area_x  , area_y  );
				bitset |= nh.get(area_x  , area_y+1);
				bitset |= nh.get(area_x+1, area_y-1);
				bitset |= nh.get(area_x+1, area_y  );
				bitset |= nh.get(area_x+1, area_y+1);
				bitset.remove(ent.id());

				let mut total_position = Vector2::new(0.0,0.0);
				let mut position_count = 0;
				let mut total_velocity = Vector2::new(0.0,0.0);
				let mut velocity_count = 0;
				let mut total_repulsion = Vector2::new(0.0,0.0);
				for (_, npos, nvel) in (&bitset, &pos_storage, &vel_storage).join() {
					let dist = distance_squared(&pos.0, &npos.0);
					if dist <= COHESION_RANGE * COHESION_RANGE {
						total_position += npos.0 - pos.0;
						position_count += 1;
					}
					if dist <= ALIGN_RANGE * ALIGN_RANGE {
						total_velocity += nvel.0;
						velocity_count += 1;
					}
					if dist <= REPULSION_RANGE * REPULSION_RANGE {
						total_repulsion -= npos.0 - pos.0;
					}
				}
				acc.0 = Vector2::new(0.0, 0.0);
				if position_count != 0 {
					acc.0 += total_position * COHESION_MAGNITUDE / position_count as f32;
				}
				if velocity_count != 0 {
					acc.0 += total_velocity * ALIGN_MAGNITUDE / velocity_count as f32;
				}
				acc.0 += total_repulsion * REPULSION_MAGNITUDE;
			});
	}
}

pub struct AccelSystem;
impl<'a> System<'a> for AccelSystem {
	type SystemData = (
		WriteStorage<'a, Vel>,
		ReadStorage<'a, Acc>,
	);

	fn run(&mut self, (mut vel, acc): Self::SystemData) {
		(&mut vel, &acc).par_join()
			.for_each(|(vel, acc)| {
				vel.0 += acc.0 / 60.0;
				if vel.0.dot(&vel.0) > SPEED_LIMIT * SPEED_LIMIT {
					vel.0 = vel.0.normalize() * SPEED_LIMIT;
				}
			});
	}
}

pub struct DrawSystem<'draw>(&'draw mut Context);
impl<'draw> DrawSystem<'draw> {
	pub fn new(ctx: &'draw mut Context) -> Self {
		Self(ctx)
	}
}
impl<'draw, 'world> System<'world> for DrawSystem<'draw> {
	type SystemData = (
		ReadStorage<'world, Pos>,
		ReadStorage<'world, Vel>,
	);

	fn run(&mut self, (pos, vel): Self::SystemData) {
		let mut mesh = MeshBuilder::new();
		let rot1 = na::Rotation2::new(std::f32::consts::PI * 0.75);
		let rot2 = na::Rotation2::new(std::f32::consts::PI * -0.75);
		(&pos, &vel).join()
			.for_each(|(pos, vel)| {
				let unitv = vel.0.normalize() * 3.0;
				mesh.polygon(
					DrawMode::fill(),
					&[pos.0 + unitv, pos.0 + (rot1 * unitv), pos.0 + (rot2 * unitv)],
					WHITE,
				).unwrap();
			});
		let mesh = mesh.build(self.0).unwrap();
		draw(self.0, &mesh, (Point2::new(0.0, 0.0), 0.0, WHITE)).unwrap();
	}
}