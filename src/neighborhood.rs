use specs::{BitSet, world::Index};
type Point2 = nalgebra::Point2<f32>;

pub struct Neighborhood{
	areas: Vec<BitSet>,
	width: i32, height: i32,
}

impl Neighborhood {
	pub fn new(width: i32, height: i32) -> Self {
		let mut areas = Vec::with_capacity( (width * height) as usize );
		for _ in 0..(width*height) {
			areas.push(BitSet::new());
		}
		Self{areas,width,height}
	}

	pub fn get(&self, x: i32, y: i32) -> &BitSet {
		let x = (x + self.width) % self.width;
		let y = (y + self.height) % self.height;
		&self.areas[(self.width * y + x) as usize]
	}

	pub fn insert(&mut self, x: i32, y: i32, id: Index) -> bool {
		self.areas[(self.width * y + x) as usize].add(id)
	}

	pub fn remove(&mut self, x: i32, y: i32, id: Index) -> bool {
		self.areas[(self.width * y + x) as usize].remove(id)
	}
}

pub fn get_area(pos: Point2, area_width: f32, area_height: f32) -> (i32, i32) {
		let px = (pos.x / area_width) as i32;
		let py = (pos.y / area_height) as i32;
		(px, py)
}