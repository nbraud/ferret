mod assets;
mod audio;
mod commands;
mod component;
mod configvars;
mod doom;
mod geometry;
mod input;
mod logger;
mod quadtree;
mod renderer;

use crate::{
	assets::{AssetHandle, AssetStorage, DataSource},
	audio::Sound,
	component::EntityTemplate,
	input::{Axis, Bindings, Button, InputState, MouseAxis},
	quadtree::Quadtree,
	renderer::{AsBytes, RenderContext},
};
use anyhow::{bail, Context};
use clap::{App, Arg, ArgMatches};
use nalgebra::{Matrix4, Vector3};
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;
use shrev::EventChannel;
use specs::{DispatcherBuilder, Entity, ReadExpect, RunNow, World, WorldExt, WriteExpect};
use std::{
	path::PathBuf,
	time::{Duration, Instant},
};
use vulkano::{
	format::Format,
	image::{Dimensions, ImmutableImage},
};
use winit::{
	event::{ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	platform::desktop::EventLoopExtDesktop,
};

fn main() -> anyhow::Result<()> {
	let arg_matches = App::new(clap::crate_name!())
		.about(clap::crate_description!())
		.version(clap::crate_version!())
		.arg(
			Arg::with_name("PWADS")
				.help("PWAD files to add")
				.multiple(true),
		)
		.arg(
			Arg::with_name("iwad")
				.help("IWAD file to use instead of the default")
				.short("i")
				.long("iwad")
				.value_name("FILE"),
		)
		.arg(
			Arg::with_name("map")
				.help("Map to load at startup")
				.short("m")
				.long("map")
				.value_name("NAME"),
		)
		.arg(
			Arg::with_name("log-level")
				.help("Highest log level to display")
				.long("log-level")
				.value_name("LEVEL")
				.possible_values(&["ERROR", "WARN", "INFO", "DEBUG", "TRACE"]),
		)
		.get_matches();

	logger::init(&arg_matches)?;

	let mut loader = doom::wad::WadLoader::new();
	load_wads(&mut loader, &arg_matches)?;

	let (command_sender, command_receiver) = commands::init()?;
	let mut event_loop = EventLoop::new();
	let (render_context, _debug_callback) =
		RenderContext::new(&event_loop).context("Could not create rendering context")?;
	let sound_sender = audio::init()?;
	let bindings = get_bindings();

	// Select map
	let map =
		if let Some(map) = arg_matches.value_of("map") {
			map
		} else {
			let wad = loader.wads().next().unwrap().file_name().unwrap();

			if wad == "doom.wad" || wad == "doom1.wad" || wad == "doomu.wad" {
				"E1M1"
			} else if wad == "doom2.wad" || wad == "tnt.wad" || wad == "plutonia.wad" {
				"MAP01"
			} else {
				bail!("No default map is known for this IWAD. Try specifying one with the \"-m\" option.")
			}
		};
	command_sender.send(format!("map {}", map)).ok();

	// Set up world
	let mut world = World::new();

	// Register components
	world.register::<doom::client::UseAction>();
	world.register::<doom::components::SpawnOnCeiling>();
	world.register::<doom::components::SpawnPoint>();
	world.register::<doom::components::Transform>();
	world.register::<doom::components::Velocity>();
	world.register::<doom::door::DoorActive>();
	world.register::<doom::door::SwitchActive>();
	world.register::<doom::light::LightFlash>();
	world.register::<doom::light::LightGlow>();
	world.register::<doom::map::LinedefRef>();
	world.register::<doom::map::MapDynamic>();
	world.register::<doom::map::SectorRef>();
	world.register::<doom::physics::BoxCollider>();
	world.register::<doom::render::sprite::SpriteRender>();
	world.register::<doom::sound::SoundPlaying>();
	world.register::<doom::update::TextureScroll>();

	// Insert asset storages
	world.insert(AssetStorage::<EntityTemplate>::default());
	world.insert(AssetStorage::<Sound>::default());
	world.insert(AssetStorage::<doom::map::Map>::default());
	world.insert(AssetStorage::<doom::map::textures::Flat>::default());
	world.insert(AssetStorage::<doom::map::textures::Wall>::default());
	world.insert(AssetStorage::<doom::image::Palette>::default());
	world.insert(AssetStorage::<doom::sprite::Sprite>::default());
	world.insert(AssetStorage::<doom::sprite::SpriteImage>::default());

	// Insert other resources
	world.insert(Pcg64Mcg::from_entropy());
	world.insert(render_context);
	world.insert(sound_sender);
	world.insert(loader);
	world.insert(InputState::new());
	world.insert(bindings);
	world.insert(Vec::<(AssetHandle<Sound>, Entity)>::new());
	world.insert(doom::client::Client::default());
	world.insert(doom::data::FRAME_TIME);
	world.insert(EventChannel::<doom::client::UseEvent>::new());

	// Create systems
	let mut render_system =
		doom::render::RenderSystem::new(&world).context("Couldn't create RenderSystem")?;
	let mut sound_system = doom::sound::SoundSystem;
	let mut update_dispatcher = DispatcherBuilder::new()
		.with_thread_local(doom::client::PlayerCommandSystem::default())
		.with_thread_local(doom::client::PlayerMoveSystem::default())
		.with_thread_local(doom::client::PlayerUseSystem::default())
		.with_thread_local(doom::physics::PhysicsSystem::default())
		.with_thread_local(doom::door::DoorUpdateSystem::new(
			world
				.get_mut::<EventChannel<doom::client::UseEvent>>()
				.unwrap()
				.register_reader(),
		))
		.with_thread_local(doom::light::LightUpdateSystem::default())
		.with_thread_local(doom::update::TextureAnimSystem::default())
		.build();

	let mut should_quit = false;
	let mut old_time = Instant::now();
	let mut leftover_time = Duration::default();

	while !should_quit {
		let mut delta;
		let mut new_time;

		// Busy-loop until there is at least a millisecond of delta
		while {
			new_time = Instant::now();
			delta = new_time - old_time;
			delta.as_millis() < 1
		} {}

		old_time = new_time;
		//println!("{} fps", 1.0/delta.as_secs_f32());

		// Process events from the system
		event_loop.run_return(|event, _, control_flow| {
			let (mut input_state, render_context) =
				world.system_data::<(WriteExpect<InputState>, ReadExpect<RenderContext>)>();
			input_state.process_event(&event);

			match event {
				Event::WindowEvent { event, .. } => match event {
					WindowEvent::CloseRequested => {
						command_sender.send("quit".to_owned()).ok();
						*control_flow = ControlFlow::Exit;
					}
					WindowEvent::Resized(_) => {
						if let Err(msg) = render_system.recreate() {
							log::warn!("Error recreating swapchain: {}", msg);
						}
					}
					WindowEvent::MouseInput {
						state: ElementState::Pressed,
						..
					} => {
						let window = render_context.surface().window();
						if let Err(msg) = window.set_cursor_grab(true) {
							log::warn!("Couldn't grab cursor: {}", msg);
						}
						window.set_cursor_visible(false);
						input_state.set_mouse_delta_enabled(true);
					}
					WindowEvent::Focused(false)
					| WindowEvent::KeyboardInput {
						input:
							KeyboardInput {
								state: ElementState::Pressed,
								virtual_keycode: Some(VirtualKeyCode::Escape),
								..
							},
						..
					} => {
						let window = render_context.surface().window();
						if let Err(msg) = window.set_cursor_grab(false) {
							log::warn!("Couldn't release cursor: {}", msg);
						}
						window.set_cursor_visible(true);
						input_state.set_mouse_delta_enabled(false);
					}
					_ => {}
				},
				Event::RedrawEventsCleared => {
					*control_flow = ControlFlow::Exit;
				}
				_ => {}
			}
		});

		// Execute console commands
		while let Some(command) = command_receiver.try_iter().next() {
			// Split into tokens
			let tokens = match commands::tokenize(&command) {
				Ok(tokens) => tokens,
				Err(e) => {
					log::error!("Invalid syntax: {}", e);
					continue;
				}
			};

			// Split further into subcommands
			for args in tokens.split(|tok| tok == ";") {
				match args[0].as_str() {
					"map" => load_map(&args[1], &mut world)?,
					"quit" => should_quit = true,
					_ => log::error!("Unknown command: {}", args[0]),
				}
			}
		}

		if should_quit {
			return Ok(());
		}

		// Run game frames
		leftover_time += delta;

		if leftover_time >= doom::data::FRAME_TIME {
			leftover_time -= doom::data::FRAME_TIME;

			update_dispatcher.dispatch(&world);

			// Reset input delta state
			{
				let mut input_state = world.fetch_mut::<InputState>();
				input_state.reset();
			}
		}

		// Update sound
		sound_system.run_now(&world);

		// Draw frame
		render_system.run_now(&world);
	}

	Ok(())
}

fn load_wads(loader: &mut doom::wad::WadLoader, arg_matches: &ArgMatches) -> anyhow::Result<()> {
	let mut wads = Vec::new();
	const IWADS: [&str; 6] = ["doom2", "plutonia", "tnt", "doomu", "doom", "doom1"];

	let iwad = if let Some(iwad) = arg_matches.value_of("iwad") {
		PathBuf::from(iwad)
	} else if let Some(iwad) = IWADS
		.iter()
		.map(|p| PathBuf::from(format!("{}.wad", p)))
		.find(|p| p.is_file())
	{
		iwad
	} else {
		bail!("No iwad file found. Try specifying one with the \"-i\" command line option.")
	};

	wads.push(iwad);

	if let Some(iter) = arg_matches.values_of("PWADS") {
		wads.extend(iter.map(PathBuf::from));
	}

	for path in wads {
		loader
			.add(&path)
			.context(format!("Couldn't load {}", path.display()))?;

		// Try to load the .gwa file as well if present
		if let Some(extension) = path.extension() {
			if extension == "wad" {
				let path = path.with_extension("gwa");

				if path.is_file() {
					loader
						.add(&path)
						.context(format!("Couldn't load {}", path.display()))?;
				}
			}
		}
	}

	Ok(())
}

fn get_bindings() -> Bindings<doom::input::Action, doom::input::Axis> {
	let mut bindings = Bindings::new();
	bindings.bind_action(
		doom::input::Action::Attack,
		Button::Mouse(MouseButton::Left),
	);
	bindings.bind_action(doom::input::Action::Use, Button::Key(VirtualKeyCode::Space));
	bindings.bind_action(doom::input::Action::Use, Button::Mouse(MouseButton::Middle));
	bindings.bind_action(
		doom::input::Action::Walk,
		Button::Key(VirtualKeyCode::LShift),
	);
	bindings.bind_action(
		doom::input::Action::Walk,
		Button::Key(VirtualKeyCode::RShift),
	);
	bindings.bind_axis(
		doom::input::Axis::Forward,
		Axis::Emulated {
			pos: Button::Key(VirtualKeyCode::W),
			neg: Button::Key(VirtualKeyCode::S),
		},
	);
	bindings.bind_axis(
		doom::input::Axis::Strafe,
		Axis::Emulated {
			pos: Button::Key(VirtualKeyCode::A),
			neg: Button::Key(VirtualKeyCode::D),
		},
	);
	bindings.bind_axis(
		doom::input::Axis::Yaw,
		Axis::Mouse {
			axis: MouseAxis::X,
			scale: 3.0,
		},
	);
	bindings.bind_axis(
		doom::input::Axis::Pitch,
		Axis::Mouse {
			axis: MouseAxis::Y,
			scale: 3.0,
		},
	);
	//println!("{}", serde_json::to_string(&bindings)?);

	bindings
}

fn load_map(name: &str, world: &mut World) -> anyhow::Result<()> {
	log::info!("Starting map {}...", name);
	let start_time = Instant::now();

	// Load palette
	let palette_handle = {
		let (mut loader, mut palette_storage) = world.system_data::<(
			WriteExpect<doom::wad::WadLoader>,
			WriteExpect<AssetStorage<crate::doom::image::Palette>>,
		)>();
		let handle = palette_storage.load("PLAYPAL", &mut *loader);
		palette_storage.build_waiting(Ok);
		handle
	};

	// Load entity type data
	log::info!("Loading entity data...");
	world.insert(doom::data::MobjTypes::new(&world));
	world.insert(doom::data::SectorTypes::new(&world));
	world.insert(doom::data::LinedefTypes::new(&world));

	// Load sprite images
	{
		let (
			palette_storage,
			mut sprite_storage,
			mut sprite_image_storage,
			mut source,
			render_context,
		) = world.system_data::<(
			ReadExpect<AssetStorage<crate::doom::image::Palette>>,
			WriteExpect<AssetStorage<crate::doom::sprite::Sprite>>,
			WriteExpect<AssetStorage<crate::doom::sprite::SpriteImage>>,
			WriteExpect<crate::doom::wad::WadLoader>,
			ReadExpect<crate::renderer::RenderContext>,
		)>();
		let palette = palette_storage.get(&palette_handle).unwrap();
		sprite_storage.build_waiting(|intermediate| {
			Ok(intermediate.build(&mut *sprite_image_storage, &mut *source)?)
		});

		sprite_image_storage.build_waiting(|image| {
			let data: Vec<_> = image
				.data
				.into_iter()
				.map(|pixel| {
					if pixel.a == 0xFF {
						palette[pixel.i as usize]
					} else {
						crate::doom::image::RGBAColor::default()
					}
				})
				.collect();

			// Create the image
			let matrix = Matrix4::new_translation(&Vector3::new(
				0.0,
				image.offset[0] as f32,
				image.offset[1] as f32,
			)) * Matrix4::new_nonuniform_scaling(&Vector3::new(
				0.0,
				image.size[0] as f32,
				image.size[1] as f32,
			));

			let (image, _future) = ImmutableImage::from_iter(
				data.as_bytes().iter().copied(),
				Dimensions::Dim2d {
					width: image.size[0] as u32,
					height: image.size[1] as u32,
				},
				Format::R8G8B8A8Unorm,
				render_context.queues().graphics.clone(),
			)?;

			Ok(crate::doom::sprite::SpriteImage { matrix, image })
		});
	}

	// Load sounds
	{
		let mut sound_storage = world.system_data::<WriteExpect<AssetStorage<Sound>>>();

		sound_storage.build_waiting(|intermediate| doom::sound::build_sound(intermediate));
	}

	// Load map
	log::info!("Loading map...");
	let map_handle = {
		let (mut loader, mut map_storage, mut flat_storage, mut wall_storage) = world
			.system_data::<(
				WriteExpect<doom::wad::WadLoader>,
				WriteExpect<AssetStorage<doom::map::Map>>,
				WriteExpect<AssetStorage<doom::map::textures::Flat>>,
				WriteExpect<AssetStorage<doom::map::textures::Wall>>,
			)>();
		let map_handle = map_storage.load(name, &mut *loader);
		map_storage.build_waiting(|data| {
			doom::map::load::build_map(
				data,
				"SKY1",
				&mut *loader,
				&mut *flat_storage,
				&mut *wall_storage,
			)
		});

		map_handle
	};

	// Build flats and wall textures
	{
		let (palette_storage, mut flat_storage, render_context) = world.system_data::<(
			ReadExpect<AssetStorage<doom::image::Palette>>,
			WriteExpect<AssetStorage<doom::map::textures::Flat>>,
			ReadExpect<RenderContext>,
		)>();
		let palette = palette_storage.get(&palette_handle).unwrap();
		flat_storage.build_waiting(|image| {
			let data: Vec<_> = image
				.data
				.into_iter()
				.map(|pixel| {
					if pixel.a == 0xFF {
						palette[pixel.i as usize]
					} else {
						crate::doom::image::RGBAColor::default()
					}
				})
				.collect();

			// Create the image
			let (image, _future) = ImmutableImage::from_iter(
				data.as_bytes().iter().copied(),
				Dimensions::Dim2d {
					width: image.size[0] as u32,
					height: image.size[1] as u32,
				},
				Format::R8G8B8A8Unorm,
				render_context.queues().graphics.clone(),
			)?;

			Ok(image)
		});
	}

	{
		let (palette_storage, mut wall_storage, render_context) = world.system_data::<(
			ReadExpect<AssetStorage<doom::image::Palette>>,
			WriteExpect<AssetStorage<doom::map::textures::Wall>>,
			ReadExpect<RenderContext>,
		)>();
		let palette = palette_storage.get(&palette_handle).unwrap();
		wall_storage.build_waiting(|image| {
			let data: Vec<_> = image
				.data
				.into_iter()
				.map(|pixel| {
					if pixel.a == 0xFF {
						palette[pixel.i as usize]
					} else {
						crate::doom::image::RGBAColor::default()
					}
				})
				.collect();

			let (image, _future) = ImmutableImage::from_iter(
				data.as_bytes().iter().copied(),
				Dimensions::Dim2d {
					width: image.size[0] as u32,
					height: image.size[1] as u32,
				},
				Format::R8G8B8A8Unorm,
				render_context.queues().graphics.clone(),
			)?;

			Ok(image)
		});
	}

	log::info!("Spawning entities...");

	// Create quadtree
	let bbox = {
		let map_storage = world.system_data::<ReadExpect<AssetStorage<doom::map::Map>>>();
		let map = map_storage.get(&map_handle).unwrap();
		map.bbox.clone()
	};
	world.insert(Quadtree::new(bbox));

	// Spawn map entities and things
	let things = {
		let loader = world.system_data::<WriteExpect<doom::wad::WadLoader>>();
		doom::map::load::build_things(&loader.load(&format!("{}/+{}", name, 1))?)?
	};
	doom::map::spawn_map_entities(&world, &map_handle)?;
	doom::map::spawn_things(things, &world, &map_handle)?;

	// Spawn player
	let entity = doom::map::spawn_player(&world)?;
	world
		.system_data::<WriteExpect<doom::client::Client>>()
		.entity = Some(entity);

	log::debug!(
		"Loading took {} s",
		(Instant::now() - start_time).as_secs_f32()
	);

	Ok(())
}
