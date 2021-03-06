use crate::{
	assets::{AssetHandle, AssetStorage},
	audio::Sound,
	doom::{
		client::{UseAction, UseEvent},
		components::Transform,
		map::{
			textures::{TextureType, Wall},
			LinedefRef, Map, MapDynamic, SectorRef, SidedefSlot,
		},
		physics::{BoxCollider, SectorTracer},
	},
	geometry::Side,
};
use shrev::{EventChannel, ReaderId};
use specs::{
	Component, DenseVecStorage, Entities, Entity, Join, ReadExpect, ReadStorage, RunNow, World,
	WriteExpect, WriteStorage,
};
use specs_derive::Component;
use std::time::Duration;

pub struct DoorUpdateSystem {
	use_event_reader: ReaderId<UseEvent>,
}

impl DoorUpdateSystem {
	pub fn new(use_event_reader: ReaderId<UseEvent>) -> DoorUpdateSystem {
		DoorUpdateSystem { use_event_reader }
	}
}

impl<'a> RunNow<'a> for DoorUpdateSystem {
	fn setup(&mut self, _world: &mut World) {}

	fn run_now(&mut self, world: &'a World) {
		let (
			entities,
			delta,
			use_event_channel,
			map_asset_storage,
			mut sound_queue,
			box_collider_component,
			linedef_ref_component,
			sector_ref_component,
			transform_component,
			use_action_component,
			mut door_active_component,
			mut map_dynamic_component,
			mut switch_active_component,
		) = world.system_data::<(
			Entities,
			ReadExpect<Duration>,
			ReadExpect<EventChannel<UseEvent>>,
			ReadExpect<AssetStorage<Map>>,
			WriteExpect<Vec<(AssetHandle<Sound>, Entity)>>,
			ReadStorage<BoxCollider>,
			ReadStorage<LinedefRef>,
			ReadStorage<SectorRef>,
			ReadStorage<Transform>,
			ReadStorage<UseAction>,
			WriteStorage<DoorActive>,
			WriteStorage<MapDynamic>,
			WriteStorage<SwitchActive>,
		)>();

		let tracer = SectorTracer {
			entities: &entities,
			transform_component: &transform_component,
			box_collider_component: &box_collider_component,
		};

		for use_event in use_event_channel.read(&mut self.use_event_reader) {
			if let Some(UseAction::DoorUse(door_use)) =
				use_action_component.get(use_event.linedef_entity)
			{
				let linedef_ref = linedef_ref_component.get(use_event.linedef_entity).unwrap();
				let map_dynamic = map_dynamic_component.get(linedef_ref.map_entity).unwrap();
				let map = map_asset_storage.get(&map_dynamic.map).unwrap();
				let linedef = &map.linedefs[linedef_ref.index];

				if let Some(back_sidedef) = &linedef.sidedefs[Side::Left as usize] {
					let sector_index = back_sidedef.sector_index;
					let sector = &map.sectors[sector_index];
					let sector_entity = map_dynamic.sectors[sector_index].entity;

					if let Some(door_active) = door_active_component.get_mut(sector_entity) {
						match door_active.state {
							DoorState::Closing => {
								// Re-open the door
								door_active.state = DoorState::Closed;
							}
							DoorState::Opening | DoorState::Open => {
								// Close the door early
								door_active.state = DoorState::Open;
								door_active.time_left = Duration::default();
							}
							DoorState::Closed => unreachable!(),
						}
					} else {
						if let Some(open_height) = sector
							.neighbours
							.iter()
							.map(|index| map_dynamic.sectors[*index].interval.max)
							.min_by(|x, y| x.partial_cmp(y).unwrap())
						{
							door_active_component
								.insert(
									sector_entity,
									DoorActive {
										open_sound: door_use.open_sound.clone(),
										open_height: open_height - 4.0,

										close_sound: door_use.close_sound.clone(),
										close_height: map_dynamic.sectors[sector_index]
											.interval
											.min,

										state: DoorState::Closed,
										speed: door_use.speed,
										time_left: door_use.wait_time,
										wait_time: door_use.wait_time,
									},
								)
								.unwrap();
						} else {
							log::error!(
								"Used door sector {} has no neighbouring sectors",
								sector_index
							);
						}
					}
				} else {
					log::error!("Used door linedef {} has no back sector", linedef_ref.index);
				}
			} else if let Some(UseAction::DoorSwitchUse(door_use)) =
				use_action_component.get(use_event.linedef_entity)
			{
				// Skip if switch is already in active state
				if switch_active_component
					.get(use_event.linedef_entity)
					.is_some()
				{
					continue;
				}

				let linedef_ref = linedef_ref_component.get(use_event.linedef_entity).unwrap();
				let map_dynamic = map_dynamic_component
					.get_mut(linedef_ref.map_entity)
					.unwrap();
				let map = map_asset_storage.get(&map_dynamic.map).unwrap();
				let linedef = &map.linedefs[linedef_ref.index];

				let mut used = false;

				// Activate all the doors with the same tag
				for (i, sector) in map
					.sectors
					.iter()
					.enumerate()
					.filter(|(_, s)| s.sector_tag == linedef.sector_tag)
				{
					let sector_entity = map_dynamic.sectors[i].entity;

					if door_active_component.get_mut(sector_entity).is_some() {
						continue;
					} else {
						used = true;
					}

					if let Some(open_height) = sector
						.neighbours
						.iter()
						.map(|index| map_dynamic.sectors[*index].interval.max)
						.min_by(|x, y| x.partial_cmp(y).unwrap())
					{
						door_active_component
							.insert(
								sector_entity,
								DoorActive {
									open_sound: door_use.open_sound.clone(),
									open_height: open_height - 4.0,

									close_sound: door_use.close_sound.clone(),
									close_height: map_dynamic.sectors[i].interval.min,

									state: DoorState::Closed,
									speed: door_use.speed,
									time_left: door_use.wait_time,
									wait_time: door_use.wait_time,
								},
							)
							.unwrap();
					} else {
						log::error!("Used door sector {}, has no neighbouring sectors", i);
					}
				}

				if used {
					// Flip the switch texture
					let sidedef = linedef.sidedefs[0].as_ref().unwrap();
					let linedef_dynamic = &mut map_dynamic.linedefs[linedef_ref.index];
					let sidedef_dynamic = linedef_dynamic.sidedefs[0].as_mut().unwrap();

					for slot in [SidedefSlot::Top, SidedefSlot::Middle, SidedefSlot::Bottom]
						.iter()
						.copied()
					{
						if let TextureType::Normal(texture) =
							&mut sidedef_dynamic.textures[slot as usize]
						{
							if let Some(new) = map.switches.get(texture) {
								// Change texture
								let old = std::mem::replace(texture, new.clone());

								// Play sound
								let sector_entity =
									map_dynamic.sectors[sidedef.sector_index].entity;
								sound_queue.push((door_use.switch_sound.clone(), sector_entity));

								// Add SwitchActive component
								switch_active_component
									.insert(
										use_event.linedef_entity,
										SwitchActive {
											sound: door_use.switch_sound.clone(),
											texture: old,
											texture_slot: slot,
											time_left: door_use.switch_time,
										},
									)
									.unwrap();

								break;
							}
						}
					}
				}
			}
		}

		let mut done = Vec::new();

		for (entity, sector_ref, door_active) in
			(&entities, &sector_ref_component, &mut door_active_component).join()
		{
			let map_dynamic = map_dynamic_component
				.get_mut(sector_ref.map_entity)
				.unwrap();
			let map = map_asset_storage.get(&map_dynamic.map).unwrap();
			let sector_dynamic = &mut map_dynamic.sectors[sector_ref.index];
			let sector = &map.sectors[sector_ref.index];

			match door_active.state {
				DoorState::Closed => {
					door_active.state = DoorState::Opening;

					// Play sound
					sound_queue.push((door_active.open_sound.clone(), entity));
				}
				DoorState::Opening => {
					let move_step = door_active.speed * delta.as_secs_f32();
					sector_dynamic.interval.max += move_step;

					if sector_dynamic.interval.max > door_active.open_height {
						sector_dynamic.interval.max = door_active.open_height;
						door_active.state = DoorState::Open;
						door_active.time_left = door_active.wait_time;
					}
				}
				DoorState::Open => {
					if let Some(new_time) = door_active.time_left.checked_sub(*delta) {
						door_active.time_left = new_time;
					} else {
						door_active.state = DoorState::Closing;

						// Play sound
						sound_queue.push((door_active.close_sound.clone(), entity));
					}
				}
				DoorState::Closing => {
					let move_step = -door_active.speed * delta.as_secs_f32();
					let trace = tracer.trace(
						-sector_dynamic.interval.max,
						-1.0,
						move_step,
						sector.subsectors.iter().map(|i| &map.subsectors[*i]),
					);

					// TODO use fraction
					if trace.collision.is_some() {
						// Hit something on the way down, re-open the door
						door_active.state = DoorState::Closed;
					} else {
						sector_dynamic.interval.max += move_step;

						if sector_dynamic.interval.max < door_active.close_height {
							done.push(entity);
						}
					}
				}
			}
		}

		for entity in &done {
			door_active_component.remove(*entity);
		}

		done.clear();

		for (entity, linedef_ref, switch_active) in (
			&entities,
			&linedef_ref_component,
			&mut switch_active_component,
		)
			.join()
		{
			if let Some(new_time) = switch_active.time_left.checked_sub(*delta) {
				switch_active.time_left = new_time;
			} else {
				let map_dynamic = map_dynamic_component
					.get_mut(linedef_ref.map_entity)
					.unwrap();
				let linedef_dynamic = &mut map_dynamic.linedefs[linedef_ref.index];
				let sidedef_dynamic = linedef_dynamic.sidedefs[0].as_mut().unwrap();
				let map = map_asset_storage.get(&map_dynamic.map).unwrap();
				let linedef = &map.linedefs[linedef_ref.index];
				let sidedef = linedef.sidedefs[0].as_ref().unwrap();
				let sector_entity = map_dynamic.sectors[sidedef.sector_index].entity;

				sidedef_dynamic.textures[switch_active.texture_slot as usize] =
					TextureType::Normal(switch_active.texture.clone());
				sound_queue.push((switch_active.sound.clone(), sector_entity));
				done.push(entity);
			}
		}

		for entity in &done {
			switch_active_component.remove(*entity);
		}
	}
}

#[derive(Clone, Debug)]
pub struct DoorUse {
	pub open_sound: AssetHandle<Sound>,
	pub close_sound: AssetHandle<Sound>,
	pub speed: f32,
	pub wait_time: Duration,
}

#[derive(Clone, Debug)]
pub struct DoorSwitchUse {
	pub open_sound: AssetHandle<Sound>,
	pub close_sound: AssetHandle<Sound>,
	pub switch_sound: AssetHandle<Sound>,
	pub switch_time: Duration,
	pub speed: f32,
	pub wait_time: Duration,
}

#[derive(Clone, Component, Debug)]
pub struct DoorActive {
	pub open_sound: AssetHandle<Sound>,
	pub open_height: f32,

	pub close_sound: AssetHandle<Sound>,
	pub close_height: f32,

	pub state: DoorState,
	pub speed: f32,
	pub time_left: Duration,
	pub wait_time: Duration,
}

#[derive(Clone, Component, Debug)]
pub struct SwitchActive {
	sound: AssetHandle<Sound>,
	texture: AssetHandle<Wall>,
	texture_slot: SidedefSlot,
	time_left: Duration,
}

#[derive(Clone, Copy, Debug)]
pub enum DoorState {
	Closed,
	Opening,
	Open,
	Closing,
}
