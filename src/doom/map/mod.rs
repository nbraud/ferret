pub mod load;
pub mod meshes;
pub mod textures;

use crate::{
	assets::{AssetHandle, AssetStorage},
	component::EntityTemplate,
	doom::{
		components::{SpawnOnCeiling, SpawnPoint, Transform},
		data::{LinedefTypes, MobjTypes, SectorTypes},
		map::{
			load::LinedefFlags,
			textures::{Flat, TextureType, Wall},
		},
		physics::{BoxCollider, SolidMask},
	},
	geometry::{Angle, Interval, Line2, Plane2, Plane3, Side, AABB2, AABB3},
	quadtree::Quadtree,
};
use anyhow::anyhow;
use bitflags::bitflags;
use nalgebra::{Vector2, Vector3};
use serde::Deserialize;
use specs::{
	storage::StorageEntry, Component, DenseVecStorage, Entity, Join, ReadExpect, ReadStorage,
	World, WorldExt, WriteExpect, WriteStorage,
};
use specs_derive::Component;
use std::{collections::HashMap, fmt::Debug, time::Duration};

#[derive(Debug)]
pub struct Map {
	pub anims_flat: HashMap<AssetHandle<Flat>, Anim<Flat>>,
	pub anims_wall: HashMap<AssetHandle<Wall>, Anim<Wall>>,
	pub bbox: AABB2,
	pub linedefs: Vec<Linedef>,
	pub nodes: Vec<Node>,
	pub sectors: Vec<Sector>,
	pub subsectors: Vec<Subsector>,
	pub sky: AssetHandle<Wall>,
	pub switches: HashMap<AssetHandle<Wall>, AssetHandle<Wall>>,
}

#[derive(Clone, Component, Debug)]
pub struct MapDynamic {
	pub anim_states_flat: HashMap<AssetHandle<Flat>, AnimState>,
	pub anim_states_wall: HashMap<AssetHandle<Wall>, AnimState>,
	pub map: AssetHandle<Map>,
	pub linedefs: Vec<LinedefDynamic>,
	pub sectors: Vec<SectorDynamic>,
}

#[derive(Clone, Debug)]
pub struct Anim<T> {
	pub frames: Vec<AssetHandle<T>>,
	pub frame_time: Duration,
}

#[derive(Clone, Copy, Debug)]
pub struct AnimState {
	pub frame: usize,
	pub time_left: Duration,
}

pub struct Thing {
	pub position: Vector2<f32>,
	pub angle: Angle,
	pub doomednum: u16,
	pub flags: ThingFlags,
}

bitflags! {
	#[derive(Deserialize)]
	pub struct ThingFlags: u16 {
		const EASY = 0b00000000_00000001;
		const NORMAL = 0b00000000_00000010;
		const HARD = 0b00000000_00000100;
		const MPONLY = 0b00000000_00001000;
	}
}

#[derive(Clone, Debug)]
pub struct Linedef {
	pub line: Line2,
	pub normal: Vector2<f32>,
	pub planes: Vec<Plane3>,
	pub bbox: AABB2,
	pub flags: LinedefFlags,
	pub solid_mask: SolidMask,
	pub special_type: u16,
	pub sector_tag: u16,
	pub sidedefs: [Option<Sidedef>; 2],
}

#[derive(Clone, Debug)]
pub struct LinedefDynamic {
	pub entity: Entity,
	pub sidedefs: [Option<SidedefDynamic>; 2],
	pub texture_offset: Vector2<f32>,
}

#[derive(Clone, Debug)]
pub struct Sidedef {
	pub texture_offset: Vector2<f32>,
	pub textures: [TextureType<Wall>; 3],
	pub sector_index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SidedefSlot {
	Top = 0,
	Bottom = 1,
	Middle = 2,
}

#[derive(Clone, Debug)]
pub struct SidedefDynamic {
	pub textures: [TextureType<Wall>; 3],
}

#[derive(Clone, Component, Debug)]
pub struct LinedefRef {
	pub map_entity: Entity,
	pub index: usize,
}

#[derive(Clone, Debug)]
pub struct Seg {
	pub line: Line2,
	pub normal: Vector2<f32>,
	pub linedef: Option<(usize, Side)>,
}

#[derive(Clone, Debug)]
pub struct Subsector {
	pub segs: Vec<Seg>,
	pub bbox: AABB2,
	pub planes: Vec<Plane3>,
	pub linedefs: Vec<usize>,
	pub sector_index: usize,
}

#[derive(Clone, Debug)]
pub struct Node {
	pub plane: Plane2,
	pub linedefs: Vec<usize>,
	pub child_bboxes: [AABB2; 2],
	pub child_indices: [NodeChild; 2],
}

#[derive(Copy, Clone, Debug)]
pub enum NodeChild {
	Subsector(usize),
	Node(usize),
}

#[derive(Clone, Debug)]
pub struct Sector {
	pub interval: Interval,
	pub textures: [TextureType<Flat>; 2],
	pub light_level: f32,
	pub special_type: u16,
	pub sector_tag: u16,
	pub linedefs: Vec<usize>,
	pub subsectors: Vec<usize>,
	pub neighbours: Vec<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SectorSlot {
	Floor = 0,
	Ceiling = 1,
}

#[derive(Clone, Debug)]
pub struct SectorDynamic {
	pub entity: Entity,
	pub light_level: f32,
	pub interval: Interval,
}

#[derive(Clone, Component, Debug)]
pub struct SectorRef {
	pub map_entity: Entity,
	pub index: usize,
}

impl Map {
	pub fn find_subsector(&self, point: Vector2<f32>) -> &Subsector {
		let mut child = NodeChild::Node(0);

		loop {
			child = match child {
				NodeChild::Subsector(index) => return &self.subsectors[index],
				NodeChild::Node(index) => {
					let node = &self.nodes[index];
					let dot = point.dot(&node.plane.normal) - node.plane.distance;
					node.child_indices[(dot <= 0.0) as usize]
				}
			};
		}
	}

	pub fn traverse_nodes<F: FnMut(NodeChild)>(&self, node: NodeChild, bbox: &AABB2, func: &mut F) {
		func(node);

		if let NodeChild::Node(index) = node {
			let node = &self.nodes[index];
			let sides = [
				Vector2::new(bbox[0].min, bbox[1].min).dot(&node.plane.normal)
					- node.plane.distance,
				Vector2::new(bbox[0].min, bbox[1].max).dot(&node.plane.normal)
					- node.plane.distance,
				Vector2::new(bbox[0].max, bbox[1].min).dot(&node.plane.normal)
					- node.plane.distance,
				Vector2::new(bbox[0].max, bbox[1].max).dot(&node.plane.normal)
					- node.plane.distance,
			];

			if sides.iter().any(|x| *x >= 0.0) {
				self.traverse_nodes(node.child_indices[Side::Right as usize], bbox, func);
			}

			if sides.iter().any(|x| *x <= 0.0) {
				self.traverse_nodes(node.child_indices[Side::Left as usize], bbox, func);
			}
		}
	}
}

pub fn spawn_things(
	things: Vec<Thing>,
	world: &World,
	map_handle: &AssetHandle<Map>,
) -> anyhow::Result<()> {
	for (_i, thing) in things.into_iter().enumerate() {
		// Fetch entity template
		let (entity_types, template_storage, mut quadtree) = world.system_data::<(
			ReadExpect<MobjTypes>,
			ReadExpect<AssetStorage<EntityTemplate>>,
			WriteExpect<Quadtree>,
		)>();
		let handle = entity_types
			.doomednums
			.get(&thing.doomednum)
			.ok_or(anyhow!("Doomednum not found: {}", thing.doomednum))?;
		let template = template_storage.get(handle).unwrap();

		// Create entity and add components
		let entity = world.entities().create();
		template.add_to_entity(entity, world)?;

		// Set entity transform
		let z = {
			let (map_storage, mut spawn_on_ceiling_component) = world
				.system_data::<(ReadExpect<AssetStorage<Map>>, WriteStorage<SpawnOnCeiling>)>();
			let map = map_storage.get(&map_handle).unwrap();
			let ssect = map.find_subsector(thing.position);
			let sector = &map.sectors[ssect.sector_index];

			if let StorageEntry::Occupied(entry) = spawn_on_ceiling_component.entry(entity)? {
				sector.interval.max - entry.remove().offset
			} else {
				sector.interval.min
			}
		};

		let (box_collider_component, mut transform_component) =
			world.system_data::<(ReadStorage<BoxCollider>, WriteStorage<Transform>)>();
		transform_component.insert(
			entity,
			Transform {
				position: Vector3::new(thing.position[0], thing.position[1], z),
				rotation: Vector3::new(0.into(), 0.into(), thing.angle),
			},
		)?;

		// Add to quadtree
		if let Some(box_collider) = box_collider_component.get(entity) {
			let transform = transform_component.get(entity).unwrap();
			let bbox = AABB3::from_radius_height(box_collider.radius, box_collider.height);
			quadtree.insert(entity, &AABB2::from(&bbox.offset(transform.position)));
		}
	}

	Ok(())
}

pub fn spawn_player(world: &World) -> anyhow::Result<Entity> {
	// Get spawn point transform
	let transform = {
		let (transform, spawn_point) =
			world.system_data::<(ReadStorage<Transform>, ReadStorage<SpawnPoint>)>();

		(&transform, &spawn_point)
			.join()
			.find_map(|(t, s)| if s.player_num == 1 { Some(*t) } else { None })
			.unwrap()
	};

	// Fetch entity template
	let (entity_types, template_storage, mut quadtree) = world.system_data::<(
		ReadExpect<MobjTypes>,
		ReadExpect<AssetStorage<EntityTemplate>>,
		WriteExpect<Quadtree>,
	)>();
	let handle = entity_types
		.names
		.get("PLAYER")
		.ok_or(anyhow!("Entity type not found: {}", "PLAYER"))?;
	let template = template_storage.get(handle).unwrap();

	// Create entity and add components
	let entity = world.entities().create();
	template.add_to_entity(entity, world)?;

	// Set entity transform
	let (box_collider_component, mut transform_component) =
		world.system_data::<(ReadStorage<BoxCollider>, WriteStorage<Transform>)>();
	transform_component.insert(entity, transform)?;

	// Add to quadtree
	if let Some(box_collider) = box_collider_component.get(entity) {
		let transform = transform_component.get(entity).unwrap();
		let bbox = AABB3::from_radius_height(box_collider.radius, box_collider.height);
		quadtree.insert(entity, &AABB2::from(&bbox.offset(transform.position)));
	}

	Ok(entity)
}

pub fn spawn_map_entities(world: &World, map_handle: &AssetHandle<Map>) -> anyhow::Result<()> {
	let (
		map_storage,
		mut map_dynamic_component,
		template_storage,
		linedef_types,
		mut linedef_ref_component,
		sector_types,
		mut sector_ref_component,
		mut transform_component,
	) = world.system_data::<(
		ReadExpect<AssetStorage<Map>>,
		WriteStorage<MapDynamic>,
		ReadExpect<AssetStorage<EntityTemplate>>,
		ReadExpect<LinedefTypes>,
		WriteStorage<LinedefRef>,
		ReadExpect<SectorTypes>,
		WriteStorage<SectorRef>,
		WriteStorage<Transform>,
	)>();
	let map = map_storage.get(&map_handle).unwrap();

	// Create map entity
	let map_entity = world.entities().create();
	let anim_states_flat = map
		.anims_flat
		.iter()
		.map(|(k, v)| {
			(
				k.clone(),
				AnimState {
					frame: 0,
					time_left: v.frame_time,
				},
			)
		})
		.collect();
	let anim_states_wall = map
		.anims_wall
		.iter()
		.map(|(k, v)| {
			(
				k.clone(),
				AnimState {
					frame: 0,
					time_left: v.frame_time,
				},
			)
		})
		.collect();

	let mut map_dynamic = MapDynamic {
		anim_states_flat,
		anim_states_wall,
		map: map_handle.clone(),
		linedefs: Vec::with_capacity(map.linedefs.len()),
		sectors: Vec::with_capacity(map.sectors.len()),
	};

	// Create linedef entities
	for (i, linedef) in map.linedefs.iter().enumerate() {
		// Create entity and set reference
		let entity = world.entities().create();
		let sidedefs = [
			linedef.sidedefs[0].as_ref().map(|sidedef| SidedefDynamic {
				textures: sidedef.textures.clone(),
			}),
			linedef.sidedefs[1].as_ref().map(|sidedef| SidedefDynamic {
				textures: sidedef.textures.clone(),
			}),
		];
		map_dynamic.linedefs.push(LinedefDynamic {
			entity,
			sidedefs,
			texture_offset: Vector2::new(0.0, 0.0),
		});
		linedef_ref_component.insert(
			entity,
			LinedefRef {
				map_entity,
				index: i,
			},
		)?;

		if linedef.special_type == 0 {
			continue;
		}

		// Fetch and add entity template
		let handle = linedef_types
			.doomednums
			.get(&linedef.special_type)
			.ok_or(anyhow!(
				"Linedef special type not found: {}",
				linedef.special_type
			))?;
		let template = template_storage.get(handle).unwrap();
		template.add_to_entity(entity, world)?;
	}

	// Create sector entities
	for (i, sector) in map.sectors.iter().enumerate() {
		// Create entity and set reference
		let entity = world.entities().create();
		map_dynamic.sectors.push(SectorDynamic {
			entity,
			light_level: sector.light_level,
			interval: sector.interval,
		});
		sector_ref_component.insert(
			entity,
			SectorRef {
				map_entity,
				index: i,
			},
		)?;

		// Find midpoint of sector for sound purposes
		let mut bbox = AABB2::empty();

		for linedef in map.linedefs.iter() {
			for sidedef in linedef.sidedefs.iter().flatten() {
				if sidedef.sector_index == i {
					bbox.add_point(linedef.line.point);
					bbox.add_point(linedef.line.point + linedef.line.dir);
				}
			}
		}

		let midpoint = (bbox.min() + bbox.max()) / 2.0;

		transform_component.insert(
			entity,
			Transform {
				position: Vector3::new(midpoint[0], midpoint[1], 0.0),
				rotation: Vector3::new(0.into(), 0.into(), 0.into()),
			},
		)?;

		if sector.special_type == 0 {
			continue;
		}

		// Fetch and add entity template
		let handle = sector_types
			.doomednums
			.get(&sector.special_type)
			.ok_or(anyhow!(
				"Sector special type not found: {}",
				sector.special_type
			))?;
		let template = template_storage.get(handle).unwrap();
		template.add_to_entity(entity, world)?;
	}

	map_dynamic_component.insert(map_entity, map_dynamic)?;

	Ok(())
}
