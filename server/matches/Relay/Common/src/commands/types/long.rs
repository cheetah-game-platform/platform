use std::io::Cursor;

use crate::constants::FieldId;
use crate::protocol::codec::cursor::VariableInt;
use crate::room::object::GameObjectId;

///
/// Обновление счетчика
/// - C->S
///
#[derive(Debug, PartialEq, Clone)]
pub struct IncrementLongC2SCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldId,
	pub increment: i64,
}

///
/// Установка счетчика
/// - C->S, S->C
///
#[derive(Debug, Clone, PartialEq)]
pub struct SetLongCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldId,
	pub value: i64,
}

///
/// Установка значения new если текущее равно current
/// reset - значение после выхода пользователя
///  
#[derive(Debug, Clone, PartialEq)]
pub struct CompareAndSetLongCommand {
	pub object_id: GameObjectId,
	pub field_id: FieldId,
	pub current: i64,
	pub new: i64,
	pub reset: i64,
}

impl SetLongCommand {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_i64(self.value)
	}

	pub fn decode(object_id: GameObjectId, field_id: FieldId, input: &mut Cursor<&mut [u8]>) -> std::io::Result<Self> {
		let value = input.read_variable_i64()?;
		Ok(SetLongCommand {
			object_id,
			field_id,
			value,
		})
	}
}

impl IncrementLongC2SCommand {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_i64(self.increment)
	}
	pub fn decode(object_id: GameObjectId, field_id: FieldId, input: &mut Cursor<&mut [u8]>) -> std::io::Result<Self> {
		let increment = input.read_variable_i64()?;
		Ok(Self {
			object_id,
			field_id,
			increment,
		})
	}
}

impl CompareAndSetLongCommand {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_i64(self.current)?;
		out.write_variable_i64(self.new)?;
		out.write_variable_i64(self.reset)
	}
	pub fn decode(object_id: GameObjectId, field_id: FieldId, input: &mut Cursor<&mut [u8]>) -> std::io::Result<Self> {
		let current = input.read_variable_i64()?;
		let new = input.read_variable_i64()?;
		let reset = input.read_variable_i64()?;
		Ok(Self {
			object_id,
			field_id,
			current,
			new,
			reset,
		})
	}
}