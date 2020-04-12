use crate::{
	assets::AssetStorage,
	doom::{
		components::{Transform, Velocity},
		map::{Map, MapDynamic},
	},
	geometry::{Interval, Line2, AABB2, AABB3},
};
use bitflags::bitflags;
use lazy_static::lazy_static;
use nalgebra::{Vector2, Vector3};
use specs::{
	Component, DenseVecStorage, Entities, Join, ReadExpect, ReadStorage, RunNow, World,
	WriteStorage,
};
use specs_derive::Component;
use std::time::Duration;

#[derive(Default)]
pub struct PhysicsSystem;

impl<'a> RunNow<'a> for PhysicsSystem {
	fn setup(&mut self, _world: &mut World) {}

	fn run_now(&mut self, world: &'a World) {
		let (
			entities,
			delta,
			map_storage,
			box_collider_component,
			map_dynamic_component,
			mut transform_component,
			mut velocity_component,
		) = world.system_data::<(
			Entities,
			ReadExpect<Duration>,
			ReadExpect<AssetStorage<Map>>,
			ReadStorage<BoxCollider>,
			ReadStorage<MapDynamic>,
			WriteStorage<Transform>,
			WriteStorage<Velocity>,
		)>();

		let map_dynamic = map_dynamic_component.join().next().unwrap();
		let map = map_storage.get(&map_dynamic.map).unwrap();

		// Clone the mask so that transform_component is free to be borrowed during the loop
		let transform_mask = transform_component.mask().clone();

		for (entity, box_collider, _, velocity) in (
			&entities,
			&box_collider_component,
			transform_mask,
			&mut velocity_component,
		)
			.join()
		{
			let move_tracer = MoveTracer {
				map,
				map_dynamic,
				transform_component: &transform_component,
				box_collider_component: &box_collider_component,
			};

			let entity_bbox = AABB3::from_radius_height(box_collider.radius, box_collider.height);
			let mut new_position = transform_component.get(entity).unwrap().position;
			let mut new_velocity = velocity.velocity;

			if new_velocity == nalgebra::zero::<Vector3<f32>>() {
				continue;
			}

			let time_left = *delta;

			for _ in 0..4 {
				let move_step = new_velocity * time_left.as_secs_f32();

				// TODO: variable solid mask
				if let Some(intersect) = move_tracer.trace(
					&entity_bbox.offset(new_position),
					move_step,
					SolidMask::NON_MONSTER,
				) {
					// Push back against the collision
					let change = intersect.normal * new_velocity.dot(&intersect.normal) * 1.01;
					new_velocity -= change;

					// Avoid bouncing too much
					if new_velocity.dot(&velocity.velocity) <= 0.0 {
						new_velocity = nalgebra::zero();
						break;
					}
				} else {
					new_position += move_step;
					break;
				}
			}

			let transform = transform_component.get_mut(entity).unwrap();
			transform.position = new_position;
			velocity.velocity = new_velocity;
		}
	}
}

#[derive(Clone, Component, Copy, Debug)]
pub struct BoxCollider {
	pub height: f32,
	pub radius: f32,
	pub solid_mask: SolidMask,
}

bitflags! {
	pub struct SolidMask: u16 {
		const NON_MONSTER = 0b01;
		const MONSTER = 0b10;
	}
}

#[derive(Clone, Debug)]
struct Intersect {
	fraction: f32,
	normal: Vector3<f32>,
	solid_mask: SolidMask,
}

struct MoveTracer<'a> {
	map: &'a Map,
	map_dynamic: &'a MapDynamic,
	transform_component: &'a WriteStorage<'a, Transform>,
	box_collider_component: &'a ReadStorage<'a, BoxCollider>,
}

impl<'a> MoveTracer<'a> {
	fn trace(
		&self,
		entity_bbox: &AABB3,
		move_step: Vector3<f32>,
		solid_mask: SolidMask,
	) -> Option<Intersect> {
		let mut ret: Option<Intersect> = None;

		if move_step[0] != 0.0 || move_step[1] != 0.0 {
			for linedef_index in 0..self.map.linedefs.len() {
				if let Some(intersect) = self.trace_linedef(&entity_bbox, move_step, linedef_index)
				{
					if intersect.fraction < ret.as_ref().map_or(1.0, |x| x.fraction)
						&& solid_mask.intersects(intersect.solid_mask)
					{
						ret = Some(intersect);
					}
				}
			}
		}

		if move_step[2] != 0.0 {
			for sector_index in 0..self.map.sectors.len() {
				if let Some(intersect) = self.trace_sector(&entity_bbox, move_step, sector_index) {
					if intersect.fraction < ret.as_ref().map_or(1.0, |x| x.fraction)
						&& solid_mask.intersects(intersect.solid_mask)
					{
						ret = Some(intersect);
					}
				}
			}
		}

		for (transform, box_collider) in
			(self.transform_component, self.box_collider_component).join()
		{
			if let Some(intersect) =
				self.trace_aabb(&entity_bbox, move_step, &box_collider, transform.position)
			{
				if intersect.fraction < ret.as_ref().map_or(1.0, |x| x.fraction)
					&& solid_mask.intersects(intersect.solid_mask)
				{
					ret = Some(intersect);
				}
			}
		}

		ret
	}

	fn trace_linedef(
		&self,
		entity_bbox: &AABB3,
		move_step: Vector3<f32>,
		linedef_index: usize,
	) -> Option<Intersect> {
		let linedef = &self.map.linedefs[linedef_index];

		let move_step2 = Vector2::new(move_step[0], move_step[1]);
		let entity_bbox2 = AABB2::from(entity_bbox);
		let move_bbox2 = entity_bbox2.union(&entity_bbox2.offset(move_step2));

		if !move_bbox2.overlaps(&linedef.bbox) {
			return None;
		}

		let entity_bbox_corners = [
			Vector2::new(entity_bbox2[0].min, entity_bbox2[1].min),
			Vector2::new(entity_bbox2[0].min, entity_bbox2[1].max),
			Vector2::new(entity_bbox2[0].max, entity_bbox2[1].max),
			Vector2::new(entity_bbox2[0].max, entity_bbox2[1].min),
		];

		let mut ret: Option<Intersect> = None;

		for i in 0..4 {
			// Intersect bbox corner with linedef
			if let Some((fraction, linedef_fraction)) =
				Line2::new(entity_bbox_corners[i], move_step2).intersect(&linedef.line)
			{
				if fraction >= 0.0
					&& fraction < ret.as_ref().map_or(1.0, |x| x.fraction)
					&& linedef_fraction >= 0.0
					&& linedef_fraction <= 1.0
				{
					ret = Some(Intersect {
						fraction,
						normal: if move_step2.dot(&linedef.normal) > 0.0 {
							// Flip the normal if we're on the left side of the linedef
							Vector3::new(-linedef.normal[0], -linedef.normal[1], 0.0)
						} else {
							Vector3::new(linedef.normal[0], linedef.normal[1], 0.0)
						},
						solid_mask: SolidMask::all(),
					});
				}
			}

			// Intersect linedef vertices with bbox edge
			let entity_bbox_edge = Line2::new(
				entity_bbox_corners[i],
				entity_bbox_corners[(i + 1) % 4] - entity_bbox_corners[i],
			);
			let linedef_vertices = [linedef.line.point, linedef.line.point + linedef.line.dir];

			for vertex in &linedef_vertices {
				if let Some((fraction, edge_fraction)) =
					Line2::new(*vertex, -move_step2).intersect(&entity_bbox_edge)
				{
					if fraction >= 0.0
						&& fraction < ret.as_ref().map_or(1.0, |x| x.fraction)
						&& edge_fraction >= 0.0 && edge_fraction <= 1.0
					{
						ret = Some(Intersect {
							fraction,
							normal: -BBOX_NORMALS[i],
							solid_mask: SolidMask::all(),
						});
					}
				}
			}
		}

		if let Some(ref mut intersect) = ret {
			if let [Some(front_sidedef), Some(back_sidedef)] = &linedef.sidedefs {
				let front_sector_dynamic = &self.map_dynamic.sectors[front_sidedef.sector_index];
				let back_sector_dynamic = &self.map_dynamic.sectors[back_sidedef.sector_index];
				let end_bbox = entity_bbox.offset(move_step * intersect.fraction);
				let interval = front_sector_dynamic
					.interval
					.intersection(back_sector_dynamic.interval);

				if end_bbox[2].is_inside(interval) {
					intersect.solid_mask = linedef.solid_mask;
				}
			}
		}

		ret
	}

	fn trace_sector(
		&self,
		entity_bbox: &AABB3,
		move_step: Vector3<f32>,
		sector_index: usize,
	) -> Option<Intersect> {
		let sector = &self.map.sectors[sector_index];
		let sector_dynamic = &self.map_dynamic.sectors[sector_index];

		let intersect = if move_step[2] > 0.0 {
			Intersect {
				fraction: (sector_dynamic.interval.max - entity_bbox[2].max) / move_step[2],
				normal: Vector3::new(0.0, 0.0, -1.0),
				solid_mask: SolidMask::all(),
			}
		} else {
			Intersect {
				fraction: (sector_dynamic.interval.min - entity_bbox[2].min) / move_step[2],
				normal: Vector3::new(0.0, 0.0, 1.0),
				solid_mask: SolidMask::all(),
			}
		};

		if intersect.fraction < 0.0 || intersect.fraction > 1.0 {
			return None;
		}

		let entity_bbox2 = AABB2::from(&(entity_bbox.offset(move_step * intersect.fraction)));
		let entity_bbox_corners = [
			Vector2::new(entity_bbox2[0].min, entity_bbox2[1].min),
			Vector2::new(entity_bbox2[0].min, entity_bbox2[1].max),
			Vector2::new(entity_bbox2[0].max, entity_bbox2[1].max),
			Vector2::new(entity_bbox2[0].max, entity_bbox2[1].min),
		];

		// Separating axis theorem
		for subsector in sector.subsectors.iter().map(|i| &self.map.subsectors[*i]) {
			if !entity_bbox2.overlaps(&subsector.bbox) {
				continue;
			}

			if subsector.segs.iter().all(|seg| {
				Interval::from_iterator(entity_bbox_corners.iter().map(|c| seg.normal.dot(c)))
					.overlaps(seg.interval)
			}) {
				// All axes had overlap, so the subsector as a whole does overlap
				return Some(intersect);
			}
		}

		// No overlapping subsectors were found
		None
	}

	fn trace_aabb(
		&self,
		entity_bbox: &AABB3,
		move_step: Vector3<f32>,
		box_collider: &BoxCollider,
		other_position: Vector3<f32>,
	) -> Option<Intersect> {
		let other_bbox = AABB3::from_radius_height(box_collider.radius, box_collider.height)
			.offset(other_position);

		// Don't collide against self
		if entity_bbox == &other_bbox {
			return None;
		}

		let intervals = Vector3::from_iterator((0..3).map(|i| {
			Interval::new(
				// TODO: handle case where move_step.dir[i] == 0.0
				(other_bbox[i].min - entity_bbox[i].max) / move_step[i],
				(other_bbox[i].max - entity_bbox[i].min) / move_step[i],
			)
			.normalize()
		}));

		let intersection = intervals[0]
			.intersection(intervals[1])
			.intersection(intervals[2]);

		if intersection.is_empty() || intersection.min < 0.0 || intersection.min > 1.0 {
			return None;
		}

		Some(Intersect {
			fraction: intersection.min,
			// TODO: make less ugly/more generic
			normal: if intersection.min == intervals[0].min {
				if move_step[0] > 0.0 {
					Vector3::new(-1.0, 0.0, 0.0)
				} else {
					Vector3::new(1.0, 0.0, 0.0)
				}
			} else if intersection.min == intervals[1].min {
				if move_step[1] > 0.0 {
					Vector3::new(0.0, -1.0, 0.0)
				} else {
					Vector3::new(0.0, 1.0, 0.0)
				}
			} else {
				if move_step[2] > 0.0 {
					Vector3::new(0.0, 0.0, -1.0)
				} else {
					Vector3::new(0.0, 0.0, 1.0)
				}
			},
			solid_mask: box_collider.solid_mask,
		})
	}
}

lazy_static! {
	static ref BBOX_NORMALS: [Vector3<f32>; 4] = [
		Vector3::new(1.0, 0.0, 0.0),   // right
		Vector3::new(0.0, 1.0, 0.0),   // up
		Vector3::new(-1.0, 0.0, 0.0),  // left
		Vector3::new(0.0, -1.0, 0.0),  // down
	];
}
