use crate::{
	assets::{AssetFormat, AssetHandle, AssetStorage, DataSource},
	doom::{
		components::TransformComponent,
		entities::{DOOMEDNUMS, ENTITIES},
		image::{DoomImageFormat, DoomPaletteFormat},
		wad::WadLoader,
	},
	geometry::BoundingBox2,
	renderer::{
		mesh::MeshBuilder,
		model::{BSPModel, Face, VertexData},
		texture::{Texture, TextureBuilder},
		video::Video,
	},
};
use byteorder::{ReadBytesExt, LE};
use nalgebra::{Matrix, Vector2, Vector3};
use sdl2::{pixels::PixelFormatEnum, rect::Rect, surface::Surface};
use specs::{world::Builder, ReadExpect, SystemData, World, WorldExt, Write};
use std::{
	collections::{
		hash_map::{Entry, HashMap},
		HashSet,
	},
	error::Error,
	io::{Cursor, ErrorKind, Read, Seek, SeekFrom},
	str,
};
use vulkano::{format::Format, image::Dimensions};

pub fn spawn_map_entities(world: &mut World, name: &str) -> Result<(), Box<dyn Error>> {
	let things = {
		let mut loader = world.fetch_mut::<WadLoader>();
		DoomMapThingsFormat.import(name, &mut *loader)?
	};

	for thing in things {
		//println!("{:#?}", thing);
		let entity = world
			.create_entity()
			.with(TransformComponent {
				position: Vector3::new(thing.position[0], thing.position[1], 0.0),
				rotation: Vector3::new(0.0, 0.0, thing.angle),
			})
			.build();

		let name = DOOMEDNUMS
			.get(&thing.doomednum)
			.ok_or(
				Box::from(format!("Doomednum not found: {}", thing.doomednum)) as Box<dyn Error>,
			)?;
		let spawn_function = ENTITIES
			.get(name)
			.ok_or(Box::from(format!("Entity not found: {}", name)) as Box<dyn Error>)?;

		spawn_function(entity, world);
	}

	Ok(())
}

fn group_by_size(
	surfaces: Vec<Surface<'static>>,
	world: &World,
) -> Vec<(AssetHandle<Texture>, usize)> {
	// Group surfaces by size in a HashMap, while keeping track of which goes where
	let mut surfaces_by_size: HashMap<[u32; 2], Vec<Surface<'static>>> = HashMap::new();
	let mut sizes_and_layers: Vec<([u32; 2], usize)> = Vec::with_capacity(surfaces.len());

	for surface in surfaces {
		let size = [surface.width(), surface.height()];
		let entry = match surfaces_by_size.entry(size) {
			Entry::Occupied(item) => item.into_mut(),
			Entry::Vacant(item) => item.insert(Vec::new()),
		};

		sizes_and_layers.push((size, entry.len()));
		entry.push(surface);
	}

	// Turn the grouped surfaces into textures
	let (mut texture_storage, video) =
		<(Write<AssetStorage<Texture>>, ReadExpect<Video>) as SystemData>::fetch(world);
	let textures_by_size = surfaces_by_size
		.into_iter()
		.map(|entry| {
			let surfaces = entry.1;
			let size = Vector3::new(
				surfaces[0].width(),
				surfaces[0].height(),
				surfaces.len() as u32,
			);

			// Find the corresponding Vulkan pixel format
			let format = match surfaces[0].pixel_format_enum() {
				PixelFormatEnum::RGB24 => Format::R8G8B8Unorm,
				PixelFormatEnum::BGR24 => Format::B8G8R8Unorm,
				PixelFormatEnum::RGBA32 => Format::R8G8B8A8Unorm,
				PixelFormatEnum::BGRA32 => Format::B8G8R8A8Unorm,
				_ => unimplemented!(),
			};

			let layer_size = surfaces[0].without_lock().unwrap().len();
			let mut data = vec![0u8; layer_size * surfaces.len()];

			// Copy all the layers into the buffer
			for (chunk, surface) in data.chunks_exact_mut(layer_size).zip(surfaces) {
				chunk.copy_from_slice(surface.without_lock().unwrap());
			}

			// Create the image
			let (texture, future) = TextureBuilder::new()
				.with_data(data, format)
				.with_dimensions(Dimensions::Dim2dArray {
					width: size[0],
					height: size[1],
					array_layers: size[2],
				})
				.build(&video.queues().graphics)
				.unwrap_or_else(|e| panic!("Error building texture: {}", e));

			let handle = texture_storage.insert(texture);
			(entry.0, handle)
		})
		.collect::<HashMap<[u32; 2], AssetHandle<Texture>>>();

	// Now create the final Vec and return
	sizes_and_layers
		.into_iter()
		.map(|entry| (textures_by_size[&entry.0].clone(), entry.1))
		.collect()
}

fn load_textures(map: &DoomMap, world: &World) -> Result<[HashMap<String, (AssetHandle<Texture>, usize)>; 2], Box<dyn Error>> {
	let mut texture_names = HashSet::new();
	for sidedef in &map.sidedefs {
		if let Some(name) = &sidedef.top_texture_name {
			texture_names.insert(name.as_str());
		}

		if let Some(name) = &sidedef.bottom_texture_name {
			texture_names.insert(name.as_str());
		}

		if let Some(name) = &sidedef.middle_texture_name {
			texture_names.insert(name.as_str());
		}
	}

	let mut flat_names = HashSet::new();
	for sector in &map.sectors {
		flat_names.insert(sector.floor_flat_name.as_str());
		flat_names.insert(sector.ceiling_flat_name.as_str());
	}

	let mut loader = world.fetch_mut::<WadLoader>();

	// Load all the surfaces, while storing name-index mapping
	let mut surfaces = Vec::with_capacity(texture_names.len() + flat_names.len());
	let mut texture_names_indices = HashMap::with_capacity(texture_names.len());
	let mut flat_names_indices = HashMap::with_capacity(flat_names.len());

	for name in texture_names {
		let surface = DoomTextureFormat.import(name, &mut *loader)?;
		texture_names_indices.insert(name, surfaces.len());
		surfaces.push(surface);
	}

	for name in flat_names {
		let surface = DoomFlatFormat.import(name, &mut *loader)?;
		flat_names_indices.insert(name, surfaces.len());
		surfaces.push(surface);
	}

	// Convert into textures grouped by size
	let grouped_textures = group_by_size(surfaces, world);

	// Recombine names with textures
	Ok([
		texture_names_indices
			.into_iter()
			.map(|entry| (entry.0.to_owned(), grouped_textures[entry.1].clone()))
			.collect(),
		flat_names_indices
			.into_iter()
			.map(|entry| (entry.0.to_owned(), grouped_textures[entry.1].clone()))
			.collect(),
	])
}

fn generate_lightmaps(world: &World) -> Result<AssetHandle<Texture>, Box<dyn Error>> {
	let (mut texture_storage, video) =
		<(Write<AssetStorage<Texture>>, ReadExpect<Video>)>::fetch(world);
	let mut surfaces = Vec::new();

	for i in 0..=15 {
		let mut surface = Surface::new(1, 1, PixelFormatEnum::RGBA32)?;
		let pixels = surface.without_lock_mut().unwrap();
		pixels[0] = i * 16;
		pixels[1] = i * 16;
		pixels[2] = i * 16;
		pixels[3] = 255;
		surfaces.push(surface);
	}

	let size = Vector3::new(
		surfaces[0].width(),
		surfaces[0].height(),
		surfaces.len() as u32,
	);

	let layer_size = surfaces[0].without_lock().unwrap().len();
	let mut data = vec![0u8; layer_size * surfaces.len()];

	// Copy all the layers into the buffer
	for (chunk, surface) in data.chunks_exact_mut(layer_size).zip(surfaces) {
		chunk.copy_from_slice(surface.without_lock().unwrap());
	}

	// Create the image
	let (texture, future) = TextureBuilder::new()
		.with_data(data, Format::R8G8B8A8Unorm)
		.with_dimensions(Dimensions::Dim2dArray {
			width: size[0],
			height: size[1],
			array_layers: size[2],
		})
		.build(&video.queues().graphics)?;

	Ok(texture_storage.insert(texture))
}

fn push_wall(
	vertices: &mut Vec<VertexData>,
	vert_h: [&Vector2<f32>; 2],
	vert_v: [f32; 2],
	offset: Vector2<f32>,
	peg_factor: [f32; 2],
	dimensions: Dimensions,
	texture_layer: f32,
	light_layer: f32,
) {
	let diff = vert_h[1] - vert_h[0];
	let width = Matrix::norm(&diff);
	let height = vert_v[1] - vert_v[0];

	for (h, v) in [(0, 0), (1, 0), (1, 1), (1, 1), (0, 1), (0, 0)].iter().copied() {
		vertices.push(VertexData {
			in_position: [vert_h[h][0], vert_h[h][1], vert_v[v]],
			in_texture_coord: [
				(offset[0] + width * h as f32) / dimensions.width() as f32,
				(offset[1] + height * peg_factor[v]) / dimensions.height() as f32,
				texture_layer,
			],
			in_lightmap_coord: [
				0.0,
				0.0,
				light_layer,
			],
		});
	}
}

fn push_flat<'a>(
	vertices: &mut Vec<VertexData>,
	mut iter: impl Iterator<Item = &'a Vector2<f32>>,
	vert_z: f32,
	dimensions: Dimensions,
	texture_layer: f32,
	light_layer: f32,
) {
	let first = iter.next().unwrap();
	let mut previous = iter.next().unwrap();

	for vert in iter {
		let vert_h = [first, previous, vert];

		for h in [0, 1, 2].iter().copied() {
			vertices.push(VertexData {
				in_position: [vert_h[h][0], vert_h[h][1], vert_z],
				in_texture_coord: [
					vert_h[h][0] / dimensions.width() as f32,
					vert_h[h][1] / dimensions.height() as f32,
					texture_layer,
				],
				in_lightmap_coord: [0.0, 0.0, light_layer],
			});
		}

		previous = vert;
	}
}

fn make_meshes(
	map: &DoomMap,
	textures: &HashMap<String, (AssetHandle<Texture>, usize)>,
	flats: &HashMap<String, (AssetHandle<Texture>, usize)>,
	world: &World,
) -> Result<(Vec<VertexData>, Vec<Face>), Box<dyn Error>> {
	let mut vertices = Vec::new();
	let mut faces = Vec::new();
	let texture_storage = <ReadExpect<AssetStorage<Texture>>>::fetch(world);

	for ssect in &map.gl_ssect {
		let segs = &map.gl_segs[ssect.first_seg_index..ssect.first_seg_index + ssect.seg_count];
		let mut sector = None;

		// Walls
		for seg in segs.iter() {
			if let Some(linedef_index) = seg.linedef_index {
				let linedef = &map.linedefs[linedef_index];

				if let Some(front_sidedef_index) = linedef.sidedef_indices[seg.side as usize] {
					let front_sidedef = &map.sidedefs[front_sidedef_index];

					// Assign sector
					if let Some(s) = sector {
						if s as *const _ != &map.sectors[front_sidedef.sector_index] as *const _ {
							return Err(Box::from("Not all the segs belong to the same sector!"));
						}
					} else {
						sector = Some(&map.sectors[front_sidedef.sector_index]);
					}

					let front_sector = sector.unwrap();

					// Add wall
					let start_vertex = if seg.vertex_indices[0].1 {
						&map.gl_vert[seg.vertex_indices[0].0]
					} else {
						&map.vertexes[seg.vertex_indices[0].0]
					};

					let end_vertex = if seg.vertex_indices[1].1 {
						&map.gl_vert[seg.vertex_indices[1].0]
					} else {
						&map.vertexes[seg.vertex_indices[1].0]
					};

					let top_peg_factor = if linedef.flags & 8 != 0 {
						[0.0, -1.0]
					} else {
						[1.0, 0.0]
					};

					let bottom_peg_factor = if linedef.flags & 16 != 0 {
						[1.0, 0.0]
					} else {
						[0.0, -1.0]
					};

					// Two-sided or one-sided sidedef?
					if let Some(back_sidedef_index) = linedef.sidedef_indices[!seg.side as usize] {
						let back_sidedef = &map.sidedefs[back_sidedef_index];
						let back_sector = &map.sectors[back_sidedef.sector_index];
						let spans = [
							front_sector.floor_height,
							f32::max(back_sector.floor_height, front_sector.floor_height),
							f32::min(front_sector.ceiling_height, back_sector.ceiling_height),
							front_sector.ceiling_height,
						];

						// Top section
						if let Some(texture_name) = &front_sidedef.top_texture_name {
							let texture = &textures[texture_name];
							let dimensions = texture_storage.get(&texture.0).unwrap().dimensions();
							faces.push(Face {
								first_vertex_index: vertices.len(),
								vertex_count: 6,
								texture: texture.0.clone(),
							});

							push_wall(
								&mut vertices,
								[start_vertex, end_vertex],
								[spans[2], spans[3]],
								front_sidedef.texture_offset,
								top_peg_factor,
								dimensions,
								texture.1 as f32,
								(front_sector.light_level >> 4) as f32,
							);
						}

						// Bottom section
						if let Some(texture_name) = &front_sidedef.bottom_texture_name {
							let texture = &textures[texture_name];
							let dimensions = texture_storage.get(&texture.0).unwrap().dimensions();
							faces.push(Face {
								first_vertex_index: vertices.len(),
								vertex_count: 6,
								texture: texture.0.clone(),
							});

							push_wall(
								&mut vertices,
								[start_vertex, end_vertex],
								[spans[0], spans[1]],
								front_sidedef.texture_offset,
								bottom_peg_factor,
								dimensions,
								texture.1 as f32,
								(front_sector.light_level >> 4) as f32,
							);
						}

						// Middle section
						if let Some(texture_name) = &front_sidedef.middle_texture_name {
							let texture = &textures[texture_name];
							let dimensions = texture_storage.get(&texture.0).unwrap().dimensions();
							faces.push(Face {
								first_vertex_index: vertices.len(),
								vertex_count: 6,
								texture: texture.0.clone(),
							});

							push_wall(
								&mut vertices,
								[start_vertex, end_vertex],
								[spans[1], spans[2]],
								front_sidedef.texture_offset,
								bottom_peg_factor,
								dimensions,
								texture.1 as f32,
								(front_sector.light_level >> 4) as f32,
							);
						}
					} else {
						if let Some(texture_name) = &front_sidedef.middle_texture_name {
							let texture = &textures[texture_name];
							let dimensions = texture_storage.get(&texture.0).unwrap().dimensions();

							faces.push(Face {
								first_vertex_index: vertices.len(),
								vertex_count: 6,
								texture: texture.0.clone(),
							});

							push_wall(
								&mut vertices,
								[start_vertex, end_vertex],
								[front_sector.floor_height, front_sector.ceiling_height],
								front_sidedef.texture_offset,
								bottom_peg_factor,
								dimensions,
								texture.1 as f32,
								(front_sector.light_level >> 4) as f32,
							);
						}
					}
				}
			}
		}

		let sector = &sector.unwrap();

		// Floor
		let flat = &flats[&sector.floor_flat_name];
		let dimensions = texture_storage.get(&flat.0).unwrap().dimensions();
		faces.push(Face {
			first_vertex_index: vertices.len(),
			vertex_count: segs.len() * 3 - 6,
			texture: flat.0.clone(),
		});

		push_flat(
			&mut vertices,
			segs.iter().rev().map(|seg| if seg.vertex_indices[0].1 {
				&map.gl_vert[seg.vertex_indices[0].0]
			} else {
				&map.vertexes[seg.vertex_indices[0].0]
			}),
			sector.floor_height,
			dimensions,
			flat.1 as f32,
			(sector.light_level >> 4) as f32,
		);

		// Ceiling
		let flat = &flats[&sector.ceiling_flat_name];
		let dimensions = texture_storage.get(&flat.0).unwrap().dimensions();
		faces.push(Face {
			first_vertex_index: vertices.len(),
			vertex_count: segs.len() * 3 - 6,
			texture: flat.0.clone(),
		});

		push_flat(
			&mut vertices,
			segs.iter().map(|seg| if seg.vertex_indices[0].1 {
				&map.gl_vert[seg.vertex_indices[0].0]
			} else {
				&map.vertexes[seg.vertex_indices[0].0]
			}),
			sector.ceiling_height,
			dimensions,
			flat.1 as f32,
			(sector.light_level >> 4) as f32,
		);
	}

	Ok((vertices, faces))
}

pub fn from_wad(name: &str, world: &World) -> Result<BSPModel, Box<dyn Error>> {
	let map = {
		let mut loader = world.fetch_mut::<WadLoader>();
		DoomMapFormat.import(name, &mut *loader)?
	};

	// Load textures and flats
	let [textures, flats] = load_textures(&map, world)?;

	// Generate lightmaps
	let lightmaps = generate_lightmaps(world)?;

	// Process all subsectors, add geometry for each seg
	let (vertices, faces) = make_meshes(&map, &textures, &flats, world)?;

	let video = world.fetch::<Video>();

	let (mesh, future) = MeshBuilder::new()
		.with_data(vertices)
		.build(&video.queues().graphics)?;
	Ok(BSPModel::new(mesh, faces, lightmaps))
}

#[derive(Clone, Debug)]
pub struct DoomMap {
	linedefs: Vec<DoomMapLinedef>,
	sidedefs: Vec<DoomMapSidedef>,
	vertexes: Vec<Vector2<f32>>,
	sectors: Vec<DoomMapSector>,
	gl_vert: Vec<Vector2<f32>>,
	gl_segs: Vec<DoomMapGLSeg>,
	gl_ssect: Vec<DoomMapGLSSect>,
	gl_nodes: Vec<DoomMapGLNode>,
}

pub struct DoomMapFormat;

impl AssetFormat for DoomMapFormat {
	type Asset = DoomMap;

	fn import(
		&self,
		name: &str,
		source: &mut impl DataSource,
	) -> Result<Self::Asset, Box<dyn Error>> {
		let gl_name = format!("GL_{}", name);

		Ok(DoomMap {
			linedefs: DoomMapLinedefsFormat.import(name, source)?,
			sidedefs: DoomMapSidedefsFormat.import(name, source)?,
			vertexes: DoomMapVertexesFormat.import(name, source)?,
			sectors: DoomMapSectorsFormat.import(name, source)?,
			gl_vert: DoomMapGLVertFormat.import(&gl_name, source)?,
			gl_segs: DoomMapGLSegsFormat.import(&gl_name, source)?,
			gl_ssect: DoomMapGLSSectFormat.import(&gl_name, source)?,
			gl_nodes: DoomMapGLNodesFormat.import(&gl_name, source)?,
		})
	}
}

#[derive(Clone, Debug)]
pub struct DoomMapThing {
	pub position: Vector2<f32>,
	pub angle: f32,
	pub doomednum: u16,
	pub flags: u16,
}

pub struct DoomMapThingsFormat;

impl AssetFormat for DoomMapThingsFormat {
	type Asset = Vec<DoomMapThing>;

	fn import(
		&self,
		name: &str,
		source: &mut impl DataSource,
	) -> Result<Self::Asset, Box<dyn Error>> {
		let mut data = Cursor::new(source.load(&format!("{}/+{}", name, 1))?);
		let mut things = Vec::new();

		loop {
			let position_x = match data.read_i16::<LE>() {
				Ok(val) => val,
				Err(err) => {
					if err.kind() == ErrorKind::UnexpectedEof {
						break;
					} else {
						return Err(Box::from(err));
					}
				}
			} as f32;
			let position_y = data.read_i16::<LE>()? as f32;
			let angle = data.read_i16::<LE>()? as f32;
			let doomednum = data.read_u16::<LE>()?;
			let flags = data.read_u16::<LE>()?;

			things.push(DoomMapThing {
				position: Vector2::new(position_x, position_y),
				angle,
				doomednum,
				flags,
			});
		}

		Ok(things)
	}
}

#[derive(Clone, Debug)]
pub struct DoomMapLinedef {
	pub start_vertex_index: usize,
	pub end_vertex_index: usize,
	pub flags: u16,
	pub special_type: u16,
	pub sector_tag: u16,
	pub sidedef_indices: [Option<usize>; 2],
}

pub struct DoomMapLinedefsFormat;

impl AssetFormat for DoomMapLinedefsFormat {
	type Asset = Vec<DoomMapLinedef>;

	fn import(
		&self,
		name: &str,
		source: &mut impl DataSource,
	) -> Result<Self::Asset, Box<dyn Error>> {
		let mut data = Cursor::new(source.load(&format!("{}/+{}", name, 2))?);
		let mut linedefs = Vec::new();

		loop {
			let start_vertex_index = match data.read_u16::<LE>() {
				Ok(val) => val,
				Err(err) => {
					if err.kind() == ErrorKind::UnexpectedEof {
						break;
					} else {
						return Err(Box::from(err));
					}
				}
			} as usize;
			let end_vertex_index = data.read_u16::<LE>()? as usize;
			let flags = data.read_u16::<LE>()?;
			let special_type = data.read_u16::<LE>()?;
			let sector_tag = data.read_u16::<LE>()?;
			let right_sidedef_index = data.read_u16::<LE>()? as usize;
			let left_sidedef_index = data.read_u16::<LE>()? as usize;

			linedefs.push(DoomMapLinedef {
				start_vertex_index,
				end_vertex_index,
				flags,
				special_type,
				sector_tag,
				sidedef_indices: [
					if right_sidedef_index == 0xFFFF {
						None
					} else {
						Some(right_sidedef_index)
					},
					if left_sidedef_index == 0xFFFF {
						None
					} else {
						Some(left_sidedef_index)
					},
				],
			});
		}

		Ok(linedefs)
	}
}

#[derive(Clone, Debug)]
pub struct DoomMapSidedef {
	pub texture_offset: Vector2<f32>,
	pub top_texture_name: Option<String>,
	pub bottom_texture_name: Option<String>,
	pub middle_texture_name: Option<String>,
	pub sector_index: usize,
}

pub struct DoomMapSidedefsFormat;

impl AssetFormat for DoomMapSidedefsFormat {
	type Asset = Vec<DoomMapSidedef>;

	fn import(
		&self,
		name: &str,
		source: &mut impl DataSource,
	) -> Result<Self::Asset, Box<dyn Error>> {
		let mut data = Cursor::new(source.load(&format!("{}/+{}", name, 3))?);
		let mut sidedefs = Vec::new();

		loop {
			let texture_offset_x = match data.read_i16::<LE>() {
				Ok(val) => val,
				Err(err) => {
					if err.kind() == ErrorKind::UnexpectedEof {
						break;
					} else {
						return Err(Box::from(err));
					}
				}
			} as f32;
			let texture_offset_y = data.read_i16::<LE>()? as f32;
			let top_texture_name = {
				let mut name = [0u8; 8];
				data.read_exact(&mut name)?;
				String::from(str::from_utf8(&name)?.trim_end_matches('\0'))
			};
			let bottom_texture_name = {
				let mut name = [0u8; 8];
				data.read_exact(&mut name)?;
				String::from(str::from_utf8(&name)?.trim_end_matches('\0'))
			};
			let middle_texture_name = {
				let mut name = [0u8; 8];
				data.read_exact(&mut name)?;
				String::from(str::from_utf8(&name)?.trim_end_matches('\0'))
			};
			let sector_index = data.read_u16::<LE>()? as usize;

			sidedefs.push(DoomMapSidedef {
				texture_offset: Vector2::new(texture_offset_x, texture_offset_y),
				top_texture_name: if top_texture_name == "-" {
					None
				} else {
					Some(top_texture_name)
				},
				bottom_texture_name: if bottom_texture_name == "-" {
					None
				} else {
					Some(bottom_texture_name)
				},
				middle_texture_name: if middle_texture_name == "-" {
					None
				} else {
					Some(middle_texture_name)
				},
				sector_index,
			});
		}

		Ok(sidedefs)
	}
}

pub struct DoomMapVertexesFormat;

impl AssetFormat for DoomMapVertexesFormat {
	type Asset = Vec<Vector2<f32>>;

	fn import(
		&self,
		name: &str,
		source: &mut impl DataSource,
	) -> Result<Self::Asset, Box<dyn Error>> {
		let mut data = Cursor::new(source.load(&format!("{}/+{}", name, 4))?);
		let mut vertexes = Vec::new();

		loop {
			let x = match data.read_i16::<LE>() {
				Ok(val) => val,
				Err(err) => {
					if err.kind() == ErrorKind::UnexpectedEof {
						break;
					} else {
						return Err(Box::from(err));
					}
				}
			} as f32;
			let y = data.read_i16::<LE>()? as f32;

			vertexes.push(Vector2::new(x, y));
		}

		Ok(vertexes)
	}
}

#[derive(Clone, Debug)]
pub struct DoomMapSector {
	pub floor_height: f32,
	pub ceiling_height: f32,
	pub floor_flat_name: String,
	pub ceiling_flat_name: String,
	pub light_level: u16,
	pub special_type: u16,
	pub sector_tag: u16,
}

pub struct DoomMapSectorsFormat;

impl AssetFormat for DoomMapSectorsFormat {
	type Asset = Vec<DoomMapSector>;

	fn import(
		&self,
		name: &str,
		source: &mut impl DataSource,
	) -> Result<Self::Asset, Box<dyn Error>> {
		let mut data = Cursor::new(source.load(&format!("{}/+{}", name, 8))?);
		let mut sectors = Vec::new();

		loop {
			let floor_height = match data.read_i16::<LE>() {
				Ok(val) => val,
				Err(err) => {
					if err.kind() == ErrorKind::UnexpectedEof {
						break;
					} else {
						return Err(Box::from(err));
					}
				}
			} as f32;
			let ceiling_height = data.read_i16::<LE>()? as f32;
			let floor_flat_name = {
				let mut name = [0u8; 8];
				data.read_exact(&mut name)?;
				String::from(str::from_utf8(&name)?.trim_end_matches('\0'))
			};
			let ceiling_flat_name = {
				let mut name = [0u8; 8];
				data.read_exact(&mut name)?;
				String::from(str::from_utf8(&name)?.trim_end_matches('\0'))
			};
			let light_level = data.read_u16::<LE>()?;
			let special_type = data.read_u16::<LE>()?;
			let sector_tag = data.read_u16::<LE>()?;

			sectors.push(DoomMapSector {
				floor_height,
				ceiling_height,
				floor_flat_name,
				ceiling_flat_name,
				light_level,
				special_type,
				sector_tag,
			});
		}

		Ok(sectors)
	}
}

pub struct DoomMapGLVertFormat;

impl AssetFormat for DoomMapGLVertFormat {
	type Asset = Vec<Vector2<f32>>;

	fn import(
		&self,
		name: &str,
		source: &mut impl DataSource,
	) -> Result<Self::Asset, Box<dyn Error>> {
		let mut data = Cursor::new(source.load(&format!("{}/+{}", name, 1))?);
		let mut gl_vert = Vec::new();

		let mut signature = [0u8; 4];
		data.read_exact(&mut signature)?;

		if &signature != b"gNd2" {
			return Err(Box::from("No gNd2 signature found"));
		}

		loop {
			let x = match data.read_i32::<LE>() {
				Ok(val) => val,
				Err(err) => {
					if err.kind() == ErrorKind::UnexpectedEof {
						break;
					} else {
						return Err(Box::from(err));
					}
				}
			} as f32;
			let y = data.read_i32::<LE>()? as f32;

			gl_vert.push(Vector2::new(x / 65536.0, y / 65536.0));
		}

		Ok(gl_vert)
	}
}

#[derive(Clone, Debug)]
pub struct DoomMapGLSeg {
	pub vertex_indices: [(usize, bool); 2],
	pub linedef_index: Option<usize>,
	pub side: bool,
	pub partner_seg_index: Option<usize>,
}

pub struct DoomMapGLSegsFormat;

impl AssetFormat for DoomMapGLSegsFormat {
	type Asset = Vec<DoomMapGLSeg>;

	fn import(
		&self,
		name: &str,
		source: &mut impl DataSource,
	) -> Result<Self::Asset, Box<dyn Error>> {
		let mut data = Cursor::new(source.load(&format!("{}/+{}", name, 2))?);
		let mut gl_segs = Vec::new();

		loop {
			let start_vertex_index = match data.read_u16::<LE>() {
				Ok(val) => val,
				Err(err) => {
					if err.kind() == ErrorKind::UnexpectedEof {
						break;
					} else {
						return Err(Box::from(err));
					}
				}
			} as usize;
			let end_vertex_index = data.read_u16::<LE>()? as usize;
			let linedef_index = data.read_u16::<LE>()? as usize;
			let side = data.read_u16::<LE>()? != 0;
			let partner_seg_index = data.read_u16::<LE>()? as usize;

			gl_segs.push(DoomMapGLSeg {
				vertex_indices: [
					if (start_vertex_index & 0x8000) != 0 {
						(start_vertex_index & 0x7FFF, true)
					} else {
						(start_vertex_index, false)
					},
					if (end_vertex_index & 0x8000) != 0 {
						(end_vertex_index & 0x7FFF, true)
					} else {
						(end_vertex_index, false)
					}
				],
				linedef_index: {
					if linedef_index == 0xFFFF {
						None
					} else {
						Some(linedef_index)
					}
				},
				side,
				partner_seg_index: {
					if partner_seg_index == 0xFFFF {
						None
					} else {
						Some(partner_seg_index)
					}
				},
			});
		}

		Ok(gl_segs)
	}
}

#[derive(Clone, Debug)]
pub struct DoomMapGLSSect {
	pub seg_count: usize,
	pub first_seg_index: usize,
}

pub struct DoomMapGLSSectFormat;

impl AssetFormat for DoomMapGLSSectFormat {
	type Asset = Vec<DoomMapGLSSect>;

	fn import(
		&self,
		name: &str,
		source: &mut impl DataSource,
	) -> Result<Self::Asset, Box<dyn Error>> {
		let mut data = Cursor::new(source.load(&format!("{}/+{}", name, 3))?);
		let mut gl_ssect = Vec::new();

		loop {
			let seg_count = match data.read_u16::<LE>() {
				Ok(val) => val,
				Err(err) => {
					if err.kind() == ErrorKind::UnexpectedEof {
						break;
					} else {
						return Err(Box::from(err));
					}
				}
			} as usize;
			let first_seg_index = data.read_u16::<LE>()? as usize;

			gl_ssect.push(DoomMapGLSSect {
				seg_count,
				first_seg_index,
			});
		}

		Ok(gl_ssect)
	}
}

#[derive(Clone, Debug)]
pub struct DoomMapGLNode {
	pub partition_point: Vector2<f32>,
	pub partition_dir: Vector2<f32>,
	pub right_bbox: BoundingBox2,
	pub left_bbox: BoundingBox2,
	pub right_child_index: BSPChildNode,
	pub left_child_index: BSPChildNode,
}

#[derive(Copy, Clone, Debug)]
pub enum BSPChildNode {
	Leaf(usize),
	Branch(usize),
}

pub struct DoomMapGLNodesFormat;

impl AssetFormat for DoomMapGLNodesFormat {
	type Asset = Vec<DoomMapGLNode>;

	fn import(
		&self,
		name: &str,
		source: &mut impl DataSource,
	) -> Result<Self::Asset, Box<dyn Error>> {
		let mut data = Cursor::new(source.load(&format!("{}/+{}", name, 4))?);
		let mut gl_nodes = Vec::new();

		loop {
			let partition_point_x = match data.read_i16::<LE>() {
				Ok(val) => val,
				Err(err) => {
					if err.kind() == ErrorKind::UnexpectedEof {
						break;
					} else {
						return Err(Box::from(err));
					}
				}
			} as f32;
			let partition_point_y = data.read_i16::<LE>()? as f32;
			let partition_dir_x = data.read_i16::<LE>()? as f32;
			let partition_dir_y = data.read_i16::<LE>()? as f32;
			let right_bbox_top = data.read_i16::<LE>()? as f32;
			let right_bbox_bottom = data.read_i16::<LE>()? as f32;
			let right_bbox_left = data.read_i16::<LE>()? as f32;
			let right_bbox_right = data.read_i16::<LE>()? as f32;
			let left_bbox_top = data.read_i16::<LE>()? as f32;
			let left_bbox_bottom = data.read_i16::<LE>()? as f32;
			let left_bbox_left = data.read_i16::<LE>()? as f32;
			let left_bbox_right = data.read_i16::<LE>()? as f32;
			let right_child_index = data.read_u16::<LE>()? as usize;
			let left_child_index = data.read_u16::<LE>()? as usize;

			gl_nodes.push(DoomMapGLNode {
				partition_point: Vector2::new(partition_point_x, partition_point_y),
				partition_dir: Vector2::new(partition_dir_x, partition_dir_y),
				right_bbox: BoundingBox2::from_extents(
					right_bbox_top,
					right_bbox_bottom,
					right_bbox_left,
					right_bbox_right,
				),
				left_bbox: BoundingBox2::from_extents(
					left_bbox_top,
					left_bbox_bottom,
					left_bbox_left,
					left_bbox_right,
				),
				right_child_index: {
					if (right_child_index & 0x8000) != 0 {
						BSPChildNode::Leaf(right_child_index & 0x7FFF)
					} else {
						BSPChildNode::Branch(right_child_index)
					}
				},
				left_child_index: {
					if (left_child_index & 0x8000) != 0 {
						BSPChildNode::Leaf(left_child_index & 0x7FFF)
					} else {
						BSPChildNode::Branch(left_child_index)
					}
				},
			});
		}

		Ok(gl_nodes)
	}
}

pub struct DoomFlatFormat;

impl AssetFormat for DoomFlatFormat {
	type Asset = Surface<'static>;

	fn import(
		&self,
		name: &str,
		source: &mut impl DataSource,
	) -> Result<Self::Asset, Box<dyn Error>> {
		let palette = DoomPaletteFormat.import("PLAYPAL", source)?;
		let mut data = Cursor::new(source.load(name)?);
		let mut surface = Surface::new(64, 64, PixelFormatEnum::RGBA32)?;

		{
			let pixels = surface.without_lock_mut().unwrap();
			let mut flat_pixels = [0u8; 64 * 64];

			data.read_exact(&mut flat_pixels)?;

			for i in 0..flat_pixels.len() {
				let color = palette[flat_pixels[i] as usize];
				pixels[4 * i + 0] = color.r;
				pixels[4 * i + 1] = color.g;
				pixels[4 * i + 2] = color.b;
				pixels[4 * i + 3] = color.a;
			}
		}

		Ok(surface)
	}
}

pub struct DoomPNamesFormat;

impl AssetFormat for DoomPNamesFormat {
	type Asset = Vec<String>;

	fn import(
		&self,
		name: &str,
		source: &mut impl DataSource,
	) -> Result<Self::Asset, Box<dyn Error>> {
		let mut data = Cursor::new(source.load(name)?);
		let count = data.read_u32::<LE>()? as usize;
		let mut pnames = Vec::with_capacity(count);

		for _ in 0..count {
			let mut name = [0u8; 8];
			data.read_exact(&mut name)?;
			let name = String::from(str::from_utf8(&name)?.trim_end_matches('\0'));
			pnames.push(name);
		}

		Ok(pnames)
	}
}

pub struct DoomTextureFormat;

impl AssetFormat for DoomTextureFormat {
	type Asset = Surface<'static>;

	fn import(
		&self,
		name: &str,
		source: &mut impl DataSource,
	) -> Result<Self::Asset, Box<dyn Error>> {
		let pnames = DoomPNamesFormat.import("PNAMES", source)?;
		let mut texture_info = DoomTexturesFormat.import("TEXTURE1", source)?;
		texture_info.extend(DoomTexturesFormat.import("TEXTURE2", source)?);

		let name = name.to_ascii_uppercase();
		let texture_info = texture_info
			.get(&name)
			.ok_or(format!("Texture {} does not exist", name))?;

		let mut surface = Surface::new(
			texture_info.size[0] as u32,
			texture_info.size[1] as u32,
			PixelFormatEnum::RGBA32,
		)?;

		for patch_info in &texture_info.patches {
			let name = &pnames[patch_info.index];

			// Use to_surface because the offsets of patches are ignored anyway
			let mut patch = DoomImageFormat.import(&name, source)?;
			let surface2 = Surface::from_data(
				&mut patch.data,
				patch.size[0] as u32,
				patch.size[1] as u32,
				patch.size[0] as u32 * 4,
				PixelFormatEnum::RGBA32,
			)?;
			surface2.blit(
				None,
				&mut surface,
				Rect::new(
					patch_info.offset[0] as i32,
					patch_info.offset[1] as i32,
					0,
					0,
				),
			)?;
		}

		Ok(surface)
	}
}

pub struct DoomPatchInfo {
	pub offset: Vector2<i32>,
	pub index: usize,
}

pub struct DoomTextureInfo {
	pub size: Vector2<u32>,
	pub patches: Vec<DoomPatchInfo>,
}

pub struct DoomTexturesFormat;

impl AssetFormat for DoomTexturesFormat {
	type Asset = HashMap<String, DoomTextureInfo>;

	fn import(
		&self,
		name: &str,
		source: &mut impl DataSource,
	) -> Result<Self::Asset, Box<dyn Error>> {
		let mut data = Cursor::new(source.load(name)?);
		let mut texture_info = HashMap::new();

		let count = data.read_u32::<LE>()? as usize;
		let mut offsets = vec![0u32; count];
		data.read_u32_into::<LE>(&mut offsets)?;

		for i in 0..count {
			data.seek(SeekFrom::Start(offsets[i] as u64))?;

			let mut name = [0u8; 8];
			data.read_exact(&mut name)?;
			let mut name = String::from(str::from_utf8(&name)?.trim_end_matches('\0'));
			name.make_ascii_uppercase();

			data.read_u32::<LE>()?; // unused bytes

			let size_x = data.read_u16::<LE>()? as u32;
			let size_y = data.read_u16::<LE>()? as u32;

			data.read_u32::<LE>()?; // unused bytes

			let patch_count = data.read_u16::<LE>()? as usize;
			let mut patches = Vec::with_capacity(patch_count);

			for _j in 0..patch_count {
				let offset_x = data.read_i16::<LE>()? as i32;
				let offset_y = data.read_i16::<LE>()? as i32;
				let patch_index = data.read_u16::<LE>()? as usize;

				data.read_u32::<LE>()?; // unused bytes

				patches.push(DoomPatchInfo {
					offset: Vector2::new(offset_x, offset_y),
					index: patch_index,
				});
			}

			texture_info.insert(
				name,
				DoomTextureInfo {
					size: Vector2::new(size_x, size_y),
					patches: patches,
				},
			);
		}

		Ok(texture_info)
	}
}
