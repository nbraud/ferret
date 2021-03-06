use anyhow::ensure;
use byteorder::{NetworkEndian as NE, ReadBytesExt, WriteBytesExt};
use std::{
	convert::TryFrom,
	error::Error,
	io::{Cursor, Read, Write},
	str,
};
use crate::commands;


#[derive(Debug)]
pub enum Packet<T> {
	Unsequenced(Vec<T>),
	Sequenced(SequencedPacket),
}

impl<T> From<SequencedPacket> for Packet<T> {
	fn from(packet: SequencedPacket) -> Packet<T> {
		Packet::Sequenced(packet)
	}
}

impl<T: TryRead<T>> TryFrom<Vec<u8>> for Packet<T> {
	type Error = anyhow::Error;

	fn try_from(data: Vec<u8>) -> anyhow::Result<Packet<T>> {
		let mut reader = Cursor::new(data);
		let sequence = reader.read_u32::<NE>()?;

		if sequence == 0xFFFFFFFF {
			let mut messages = Vec::new();

			while reader.position() < reader.get_ref().len() as u64 {
				messages.push(T::try_read(&mut reader)?)
			}

			Ok(Packet::Unsequenced(messages))
		} else {
			Ok(SequencedPacket::try_from(reader.into_inner())?.into())
		}
	}
}

impl<T: Into<Vec<u8>>> From<Packet<T>> for Vec<u8> {
	fn from(packet: Packet<T>) -> Vec<u8> {
		match packet {
			Packet::Unsequenced(messages) => {
				let mut writer = Cursor::new(Vec::new());
				writer.write_u32::<NE>(0xFFFFFFFF).unwrap();

				for message in messages {
					writer.write(&message.into()).unwrap();
				}

				writer.into_inner()
			},
			Packet::Sequenced(p) => p.into(),
		}
	}
}

#[derive(Debug)]
pub struct SequencedPacket {
	pub sequence: u32,
	pub data: Vec<u8>,
}

impl TryFrom<Vec<u8>> for SequencedPacket {
	type Error = anyhow::Error;

	fn try_from(buf: Vec<u8>) -> anyhow::Result<SequencedPacket> {
		let mut reader = Cursor::new(buf);
		let sequence = reader.read_u32::<NE>()?;

		ensure!(sequence != 0xFFFFFFFF, "not a sequenced packet");

		Ok(SequencedPacket {
			sequence,
			data: reader.into_inner()[4..].to_owned(),
		})
	}
}

impl From<SequencedPacket> for Vec<u8> {
	fn from(packet: SequencedPacket) -> Vec<u8> {
		let mut writer = Cursor::new(Vec::new());
		writer.write_u32::<NE>(packet.sequence).unwrap();
		writer.write(&packet.data).unwrap();
		writer.into_inner()
	}
}

pub trait TryRead<T> {
	fn try_read(reader: &mut Cursor<Vec<u8>>) -> anyhow::Result<T>;
}


/*
	Client-to-server protocol
*/

#[derive(Debug)]
pub enum ClientMessage {
	Connect,
	RCon(String),
}

impl TryRead<ClientMessage> for ClientMessage {
	fn try_read(reader: &mut Cursor<Vec<u8>>) -> anyhow::Result<ClientMessage> {
		let message_type = reader.read_u8()?;

		Ok(match message_type {
			1 => {
				ClientMessage::Connect
			},
			2 => {
				let length = reader.read_u32::<NE>()?;
				let mut data = vec![0u8; length as usize];
				reader.read_exact(data.as_mut_slice())?;
				ClientMessage::RCon(String::from_utf8(data)?)
			},
			_ => unreachable!(),
		})
	}
}

impl From<ClientMessage> for Vec<u8> {
	fn from(message: ClientMessage) -> Vec<u8> {
		let mut writer = Cursor::new(Vec::new());

		match message {
			ClientMessage::Connect => {
				writer.write_u8(1).unwrap();
			}
			ClientMessage::RCon(text) => {
				writer.write_u8(2).unwrap();
				writer.write_u32::<NE>(text.len() as u32).unwrap();
				writer.write(text.as_bytes()).unwrap();
			}
		}

		writer.into_inner()
	}
}


/*
	Server-to-client protocol
*/

#[derive(Debug)]
pub enum ServerMessage {
	//ConfigVariable(String, String),
	ConnectResponse,
	ComponentDelete(u32, u8),
	ComponentDelta(u32, u8, Vec<u8>),
	ComponentNew(u32, u8),
	Disconnect,
	EntityDelete(u32),
	EntityNew(u32),
}

impl TryRead<ServerMessage> for ServerMessage {
	fn try_read(reader: &mut Cursor<Vec<u8>>) -> anyhow::Result<ServerMessage> {
		let message_type = reader.read_u8()?;

		Ok(match message_type {
			1 => {
				ServerMessage::ConnectResponse
			},
			2 => {
				let entity_id = reader.read_u32::<NE>()?;
				let component_id = reader.read_u8()?;
				ServerMessage::ComponentDelete(entity_id, component_id)
			},
			3 => {
				let entity_id = reader.read_u32::<NE>()?;
				let component_id = reader.read_u8()?;
				let length = reader.read_u32::<NE>()?;
				let mut data = vec![0u8; length as usize];
				reader.read_exact(data.as_mut_slice())?;
				ServerMessage::ComponentDelta(entity_id, component_id, data)
			},
			4 => {
				let entity_id = reader.read_u32::<NE>()?;
				let component_id = reader.read_u8()?;
				ServerMessage::ComponentNew(entity_id, component_id)
			},
			5 => {
				ServerMessage::Disconnect
			},
			6 => {
				let entity_id = reader.read_u32::<NE>()?;
				ServerMessage::EntityDelete(entity_id)
			},
			7 => {
				let entity_id = reader.read_u32::<NE>()?;
				ServerMessage::EntityNew(entity_id)
			},
			_ => unreachable!(),
		})
	}
}

impl From<ServerMessage> for Vec<u8> {
	fn from(message: ServerMessage) -> Vec<u8> {
		let mut writer = Cursor::new(Vec::new());

		match message {
			ServerMessage::ConnectResponse => {
				writer.write_u8(1).unwrap();
			},
			ServerMessage::ComponentDelete(entity_id, component_id) => {
				writer.write_u8(2).unwrap();
				writer.write_u32::<NE>(entity_id).unwrap();
				writer.write_u8(component_id).unwrap();
			},
			ServerMessage::ComponentDelta(entity_id, component_id, data) => {
				writer.write_u8(3).unwrap();
				writer.write_u32::<NE>(entity_id).unwrap();
				writer.write_u8(component_id).unwrap();
				writer.write_u32::<NE>(data.len() as u32).unwrap();
				writer.write(&data).unwrap();
			},
			ServerMessage::ComponentNew(entity_id, component_id) => {
				writer.write_u8(4).unwrap();
				writer.write_u32::<NE>(entity_id).unwrap();
				writer.write_u8(component_id).unwrap();
			},
			ServerMessage::Disconnect => {
				writer.write_u8(5).unwrap();
			},
			ServerMessage::EntityDelete(entity_id) => {
				writer.write_u8(6).unwrap();
				writer.write_u32::<NE>(entity_id).unwrap();
			},
			ServerMessage::EntityNew(entity_id) => {
				writer.write_u8(7).unwrap();
				writer.write_u32::<NE>(entity_id).unwrap();
			},
		}

		writer.into_inner()
	}
}
