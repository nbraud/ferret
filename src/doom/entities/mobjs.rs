#![allow(unused_variables)]
use crate::{
	assets::{AssetHandle, AssetStorage},
	component::EntityTemplate,
	doom::{
		components::{SpawnOnCeiling, SpawnPoint, SpriteRender, Velocity},
		sprite::Sprite,
		wad::WadLoader,
	},
};
use specs::{World, WriteExpect};
use std::collections::HashMap;

pub struct MobjTypes {
	pub names: HashMap<&'static str, AssetHandle<EntityTemplate>>,
	pub doomednums: HashMap<u16, AssetHandle<EntityTemplate>>,
}

impl MobjTypes {
	#[rustfmt::skip]
	pub fn new(world: &World) -> MobjTypes {
		let (mut template_storage, mut sprite_storage, mut loader) = world.system_data::<(
			WriteExpect<AssetStorage<EntityTemplate>>,
			WriteExpect<AssetStorage<Sprite>>,
			WriteExpect<WadLoader>,
		)>();

		let mut names = HashMap::new();
		let mut doomednums = HashMap::new();

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpawnPoint { player_num: 1 })
		});
		doomednums.insert(1, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpawnPoint { player_num: 2 })
		});
		doomednums.insert(2, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpawnPoint { player_num: 3 })
		});
		doomednums.insert(3, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpawnPoint { player_num: 4 })
		});
		doomednums.insert(4, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
		});
		doomednums.insert(11, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("PLAY", &mut *loader),
					frame: 0,
					full_bright: false,
				})
				.with_component(Velocity::default())
		});
		names.insert("PLAYER", handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("POSS", &mut *loader),
					frame: 0,
					full_bright: false,
				})
				.with_component(Velocity::default())
		});
		names.insert("POSSESSED", handle.clone());
		doomednums.insert(3004, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("SPOS", &mut *loader),
					frame: 0,
					full_bright: false,
				})
				.with_component(Velocity::default())
		});
		names.insert("SHOTGUY", handle.clone());
		doomednums.insert(9, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("VILE", &mut *loader),
					frame: 0,
					full_bright: false,
				})
				.with_component(Velocity::default())
		});
		names.insert("VILE", handle.clone());
		doomednums.insert(64, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("FIRE", &mut *loader),
					frame: 0,
					full_bright: true,
				})
		});
		names.insert("FIRE", handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("SKEL", &mut *loader),
					frame: 0,
					full_bright: false,
				})
				.with_component(Velocity::default())
		});
		names.insert("UNDEAD", handle.clone());
		doomednums.insert(66, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("FATB", &mut *loader),
					frame: 0,
					full_bright: true,
				})
				.with_component(Velocity::default())
		});
		names.insert("TRACER", handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("PUFF", &mut *loader),
					frame: 1,
					full_bright: false,
				})
		});
		names.insert("SMOKE", handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("FATT", &mut *loader),
					frame: 0,
					full_bright: false,
				})
				.with_component(Velocity::default())
		});
		names.insert("FATSO", handle.clone());
		doomednums.insert(67, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("MANF", &mut *loader),
					frame: 0,
					full_bright: true,
				})
				.with_component(Velocity::default())
		});
		names.insert("FATSHOT", handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("CPOS", &mut *loader),
					frame: 0,
					full_bright: false,
				})
				.with_component(Velocity::default())
		});
		names.insert("CHAINGUY", handle.clone());
		doomednums.insert(65, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("TROO", &mut *loader),
					frame: 0,
					full_bright: false,
				})
				.with_component(Velocity::default())
		});
		names.insert("TROOP", handle.clone());
		doomednums.insert(3001, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("SARG", &mut *loader),
					frame: 0,
					full_bright: false,
				})
				.with_component(Velocity::default())
		});
		names.insert("SERGEANT", handle.clone());
		doomednums.insert(3002, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("SARG", &mut *loader),
					frame: 0,
					full_bright: false,
				})
				.with_component(Velocity::default())
		});
		names.insert("SHADOWS", handle.clone());
		doomednums.insert(58, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("HEAD", &mut *loader),
					frame: 0,
					full_bright: false,
				})
				.with_component(Velocity::default())
		});
		names.insert("HEAD", handle.clone());
		doomednums.insert(3005, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("BOSS", &mut *loader),
					frame: 0,
					full_bright: false,
				})
				.with_component(Velocity::default())
		});
		names.insert("BRUISER", handle.clone());
		doomednums.insert(3003, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("BAL7", &mut *loader),
					frame: 0,
					full_bright: true,
				})
				.with_component(Velocity::default())
		});
		names.insert("BRUISERSHOT", handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("BOS2", &mut *loader),
					frame: 0,
					full_bright: false,
				})
				.with_component(Velocity::default())
		});
		names.insert("KNIGHT", handle.clone());
		doomednums.insert(69, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("SKUL", &mut *loader),
					frame: 0,
					full_bright: true,
				})
				.with_component(Velocity::default())
		});
		names.insert("SKULL", handle.clone());
		doomednums.insert(3006, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("SPID", &mut *loader),
					frame: 0,
					full_bright: false,
				})
				.with_component(Velocity::default())
		});
		names.insert("SPIDER", handle.clone());
		doomednums.insert(7, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("BSPI", &mut *loader),
					frame: 0,
					full_bright: false,
				})
				.with_component(Velocity::default())
		});
		names.insert("BABY", handle.clone());
		doomednums.insert(68, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("CYBR", &mut *loader),
					frame: 0,
					full_bright: false,
				})
				.with_component(Velocity::default())
		});
		names.insert("CYBORG", handle.clone());
		doomednums.insert(16, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("PAIN", &mut *loader),
					frame: 0,
					full_bright: false,
				})
				.with_component(Velocity::default())
		});
		names.insert("PAIN", handle.clone());
		doomednums.insert(71, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("SSWV", &mut *loader),
					frame: 0,
					full_bright: false,
				})
				.with_component(Velocity::default())
		});
		names.insert("WOLFSS", handle.clone());
		doomednums.insert(84, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpawnOnCeiling {
					offset: 72.0,
				})
				.with_component(SpriteRender {
					sprite: sprite_storage.load("KEEN", &mut *loader),
					frame: 0,
					full_bright: false,
				})
				.with_component(Velocity::default())
		});
		names.insert("KEEN", handle.clone());
		doomednums.insert(72, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("BBRN", &mut *loader),
					frame: 0,
					full_bright: false,
				})
				.with_component(Velocity::default())
		});
		names.insert("BOSSBRAIN", handle.clone());
		doomednums.insert(88, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("SSWV", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("BOSSSPIT", handle.clone());
		doomednums.insert(89, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
		});
		names.insert("BOSSTARGET", handle.clone());
		doomednums.insert(87, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("BOSF", &mut *loader),
					frame: 0,
					full_bright: true,
				})
				.with_component(Velocity::default())
		});
		names.insert("SPAWNSHOT", handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("FIRE", &mut *loader),
					frame: 0,
					full_bright: true,
				})
		});
		names.insert("SPAWNFIRE", handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("BAR1", &mut *loader),
					frame: 0,
					full_bright: false,
				})
				.with_component(Velocity::default())
		});
		names.insert("BARREL", handle.clone());
		doomednums.insert(2035, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("BAL1", &mut *loader),
					frame: 0,
					full_bright: true,
				})
				.with_component(Velocity::default())
		});
		names.insert("TROOPSHOT", handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("BAL2", &mut *loader),
					frame: 0,
					full_bright: true,
				})
				.with_component(Velocity::default())
		});
		names.insert("HEADSHOT", handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("MISL", &mut *loader),
					frame: 0,
					full_bright: true,
				})
				.with_component(Velocity::default())
		});
		names.insert("ROCKET", handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("PLSS", &mut *loader),
					frame: 0,
					full_bright: true,
				})
				.with_component(Velocity::default())
		});
		names.insert("PLASMA", handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("BFS1", &mut *loader),
					frame: 0,
					full_bright: true,
				})
				.with_component(Velocity::default())
		});
		names.insert("BFG", handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("APLS", &mut *loader),
					frame: 0,
					full_bright: true,
				})
				.with_component(Velocity::default())
		});
		names.insert("ARACHPLAZ", handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("PUFF", &mut *loader),
					frame: 0,
					full_bright: true,
				})
		});
		names.insert("PUFF", handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("BLUD", &mut *loader),
					frame: 2,
					full_bright: false,
				})
		});
		names.insert("BLOOD", handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("TFOG", &mut *loader),
					frame: 0,
					full_bright: true,
				})
		});
		names.insert("TFOG", handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("IFOG", &mut *loader),
					frame: 0,
					full_bright: true,
				})
		});
		names.insert("IFOG", handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
		});
		names.insert("TELEPORTMAN", handle.clone());
		doomednums.insert(14, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("BFE2", &mut *loader),
					frame: 0,
					full_bright: true,
				})
		});
		names.insert("EXTRABFG", handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("ARM1", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC0", handle.clone());
		doomednums.insert(2018, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("ARM2", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC1", handle.clone());
		doomednums.insert(2019, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("BON1", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC2", handle.clone());
		doomednums.insert(2014, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("BON2", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC3", handle.clone());
		doomednums.insert(2015, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("BKEY", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC4", handle.clone());
		doomednums.insert(5, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("RKEY", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC5", handle.clone());
		doomednums.insert(13, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("YKEY", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC6", handle.clone());
		doomednums.insert(6, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("YSKU", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC7", handle.clone());
		doomednums.insert(39, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("RSKU", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC8", handle.clone());
		doomednums.insert(38, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("BSKU", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC9", handle.clone());
		doomednums.insert(40, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("STIM", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC10", handle.clone());
		doomednums.insert(2011, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("MEDI", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC11", handle.clone());
		doomednums.insert(2012, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("SOUL", &mut *loader),
					frame: 0,
					full_bright: true,
				})
		});
		names.insert("MISC12", handle.clone());
		doomednums.insert(2013, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("PINV", &mut *loader),
					frame: 0,
					full_bright: true,
				})
		});
		names.insert("INV", handle.clone());
		doomednums.insert(2022, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("PSTR", &mut *loader),
					frame: 0,
					full_bright: true,
				})
		});
		names.insert("MISC13", handle.clone());
		doomednums.insert(2023, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("PINS", &mut *loader),
					frame: 0,
					full_bright: true,
				})
		});
		names.insert("INS", handle.clone());
		doomednums.insert(2024, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("SUIT", &mut *loader),
					frame: 0,
					full_bright: true,
				})
		});
		names.insert("MISC14", handle.clone());
		doomednums.insert(2025, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("PMAP", &mut *loader),
					frame: 0,
					full_bright: true,
				})
		});
		names.insert("MISC15", handle.clone());
		doomednums.insert(2026, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("PVIS", &mut *loader),
					frame: 0,
					full_bright: true,
				})
		});
		names.insert("MISC16", handle.clone());
		doomednums.insert(2045, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("MEGA", &mut *loader),
					frame: 0,
					full_bright: true,
				})
		});
		names.insert("MEGA", handle.clone());
		doomednums.insert(83, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("CLIP", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("CLIP", handle.clone());
		doomednums.insert(2007, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("AMMO", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC17", handle.clone());
		doomednums.insert(2048, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("ROCK", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC18", handle.clone());
		doomednums.insert(2010, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("BROK", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC19", handle.clone());
		doomednums.insert(2046, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("CELL", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC20", handle.clone());
		doomednums.insert(2047, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("CELP", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC21", handle.clone());
		doomednums.insert(17, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("SHEL", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC22", handle.clone());
		doomednums.insert(2008, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("SBOX", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC23", handle.clone());
		doomednums.insert(2049, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("BPAK", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC24", handle.clone());
		doomednums.insert(8, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("BFUG", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC25", handle.clone());
		doomednums.insert(2006, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("MGUN", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("CHAINGUN", handle.clone());
		doomednums.insert(2002, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("CSAW", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC26", handle.clone());
		doomednums.insert(2005, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("LAUN", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC27", handle.clone());
		doomednums.insert(2003, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("PLAS", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC28", handle.clone());
		doomednums.insert(2004, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("SHOT", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("SHOTGUN", handle.clone());
		doomednums.insert(2001, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("SGN2", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("SUPERSHOTGUN", handle.clone());
		doomednums.insert(82, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("TLMP", &mut *loader),
					frame: 0,
					full_bright: true,
				})
		});
		names.insert("MISC29", handle.clone());
		doomednums.insert(85, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("TLP2", &mut *loader),
					frame: 0,
					full_bright: true,
				})
		});
		names.insert("MISC30", handle.clone());
		doomednums.insert(86, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("COLU", &mut *loader),
					frame: 0,
					full_bright: true,
				})
		});
		names.insert("MISC31", handle.clone());
		doomednums.insert(2028, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("COL1", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC32", handle.clone());
		doomednums.insert(30, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("COL2", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC33", handle.clone());
		doomednums.insert(31, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("COL3", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC34", handle.clone());
		doomednums.insert(32, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("COL4", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC35", handle.clone());
		doomednums.insert(33, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("COL6", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC36", handle.clone());
		doomednums.insert(37, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("COL5", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC37", handle.clone());
		doomednums.insert(36, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("CEYE", &mut *loader),
					frame: 0,
					full_bright: true,
				})
		});
		names.insert("MISC38", handle.clone());
		doomednums.insert(41, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("FSKU", &mut *loader),
					frame: 0,
					full_bright: true,
				})
		});
		names.insert("MISC39", handle.clone());
		doomednums.insert(42, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("TRE1", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC40", handle.clone());
		doomednums.insert(43, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("TBLU", &mut *loader),
					frame: 0,
					full_bright: true,
				})
		});
		names.insert("MISC41", handle.clone());
		doomednums.insert(44, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("TGRN", &mut *loader),
					frame: 0,
					full_bright: true,
				})
		});
		names.insert("MISC42", handle.clone());
		doomednums.insert(45, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("TRED", &mut *loader),
					frame: 0,
					full_bright: true,
				})
		});
		names.insert("MISC43", handle.clone());
		doomednums.insert(46, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("SMBT", &mut *loader),
					frame: 0,
					full_bright: true,
				})
		});
		names.insert("MISC44", handle.clone());
		doomednums.insert(55, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("SMGT", &mut *loader),
					frame: 0,
					full_bright: true,
				})
		});
		names.insert("MISC45", handle.clone());
		doomednums.insert(56, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("SMRT", &mut *loader),
					frame: 0,
					full_bright: true,
				})
		});
		names.insert("MISC46", handle.clone());
		doomednums.insert(57, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("SMIT", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC47", handle.clone());
		doomednums.insert(47, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("ELEC", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC48", handle.clone());
		doomednums.insert(48, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("CAND", &mut *loader),
					frame: 0,
					full_bright: true,
				})
		});
		names.insert("MISC49", handle.clone());
		doomednums.insert(34, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("CBRA", &mut *loader),
					frame: 0,
					full_bright: true,
				})
		});
		names.insert("MISC50", handle.clone());
		doomednums.insert(35, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpawnOnCeiling {
					offset: 68.0,
				})
				.with_component(SpriteRender {
					sprite: sprite_storage.load("GOR1", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC51", handle.clone());
		doomednums.insert(49, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpawnOnCeiling {
					offset: 84.0,
				})
				.with_component(SpriteRender {
					sprite: sprite_storage.load("GOR2", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC52", handle.clone());
		doomednums.insert(50, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpawnOnCeiling {
					offset: 84.0,
				})
				.with_component(SpriteRender {
					sprite: sprite_storage.load("GOR3", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC53", handle.clone());
		doomednums.insert(51, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpawnOnCeiling {
					offset: 68.0,
				})
				.with_component(SpriteRender {
					sprite: sprite_storage.load("GOR4", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC54", handle.clone());
		doomednums.insert(52, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpawnOnCeiling {
					offset: 52.0,
				})
				.with_component(SpriteRender {
					sprite: sprite_storage.load("GOR5", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC55", handle.clone());
		doomednums.insert(53, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpawnOnCeiling {
					offset: 84.0,
				})
				.with_component(SpriteRender {
					sprite: sprite_storage.load("GOR2", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC56", handle.clone());
		doomednums.insert(59, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpawnOnCeiling {
					offset: 68.0,
				})
				.with_component(SpriteRender {
					sprite: sprite_storage.load("GOR4", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC57", handle.clone());
		doomednums.insert(60, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpawnOnCeiling {
					offset: 52.0,
				})
				.with_component(SpriteRender {
					sprite: sprite_storage.load("GOR3", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC58", handle.clone());
		doomednums.insert(61, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpawnOnCeiling {
					offset: 52.0,
				})
				.with_component(SpriteRender {
					sprite: sprite_storage.load("GOR5", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC59", handle.clone());
		doomednums.insert(62, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpawnOnCeiling {
					offset: 68.0,
				})
				.with_component(SpriteRender {
					sprite: sprite_storage.load("GOR1", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC60", handle.clone());
		doomednums.insert(63, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("HEAD", &mut *loader),
					frame: 11,
					full_bright: false,
				})
		});
		names.insert("MISC61", handle.clone());
		doomednums.insert(22, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("PLAY", &mut *loader),
					frame: 13,
					full_bright: false,
				})
		});
		names.insert("MISC62", handle.clone());
		doomednums.insert(15, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("POSS", &mut *loader),
					frame: 11,
					full_bright: false,
				})
		});
		names.insert("MISC63", handle.clone());
		doomednums.insert(18, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("SARG", &mut *loader),
					frame: 13,
					full_bright: false,
				})
		});
		names.insert("MISC64", handle.clone());
		doomednums.insert(21, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("SKUL", &mut *loader),
					frame: 10,
					full_bright: false,
				})
		});
		names.insert("MISC65", handle.clone());
		doomednums.insert(23, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("TROO", &mut *loader),
					frame: 12,
					full_bright: false,
				})
		});
		names.insert("MISC66", handle.clone());
		doomednums.insert(20, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("SPOS", &mut *loader),
					frame: 11,
					full_bright: false,
				})
		});
		names.insert("MISC67", handle.clone());
		doomednums.insert(19, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("PLAY", &mut *loader),
					frame: 22,
					full_bright: false,
				})
		});
		names.insert("MISC68", handle.clone());
		doomednums.insert(10, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("PLAY", &mut *loader),
					frame: 22,
					full_bright: false,
				})
		});
		names.insert("MISC69", handle.clone());
		doomednums.insert(12, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("POL2", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC70", handle.clone());
		doomednums.insert(28, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("POL5", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC71", handle.clone());
		doomednums.insert(24, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("POL4", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC72", handle.clone());
		doomednums.insert(27, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("POL3", &mut *loader),
					frame: 0,
					full_bright: true,
				})
		});
		names.insert("MISC73", handle.clone());
		doomednums.insert(29, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("POL1", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC74", handle.clone());
		doomednums.insert(25, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("POL6", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC75", handle.clone());
		doomednums.insert(26, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("TRE2", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC76", handle.clone());
		doomednums.insert(54, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("FCAN", &mut *loader),
					frame: 0,
					full_bright: true,
				})
		});
		names.insert("MISC77", handle.clone());
		doomednums.insert(70, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpawnOnCeiling {
					offset: 88.0,
				})
				.with_component(SpriteRender {
					sprite: sprite_storage.load("HDB1", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC78", handle.clone());
		doomednums.insert(73, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpawnOnCeiling {
					offset: 88.0,
				})
				.with_component(SpriteRender {
					sprite: sprite_storage.load("HDB2", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC79", handle.clone());
		doomednums.insert(74, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpawnOnCeiling {
					offset: 64.0,
				})
				.with_component(SpriteRender {
					sprite: sprite_storage.load("HDB3", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC80", handle.clone());
		doomednums.insert(75, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpawnOnCeiling {
					offset: 64.0,
				})
				.with_component(SpriteRender {
					sprite: sprite_storage.load("HDB4", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC81", handle.clone());
		doomednums.insert(76, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpawnOnCeiling {
					offset: 64.0,
				})
				.with_component(SpriteRender {
					sprite: sprite_storage.load("HDB5", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC82", handle.clone());
		doomednums.insert(77, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpawnOnCeiling {
					offset: 64.0,
				})
				.with_component(SpriteRender {
					sprite: sprite_storage.load("HDB6", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC83", handle.clone());
		doomednums.insert(78, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("POB1", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC84", handle.clone());
		doomednums.insert(79, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("POB2", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC85", handle.clone());
		doomednums.insert(80, handle.clone());

		let handle = template_storage.insert({
			EntityTemplate::new()
				.with_component(SpriteRender {
					sprite: sprite_storage.load("BRS1", &mut *loader),
					frame: 0,
					full_bright: false,
				})
		});
		names.insert("MISC86", handle.clone());
		doomednums.insert(81, handle.clone());

		MobjTypes { names, doomednums }
	}
}
