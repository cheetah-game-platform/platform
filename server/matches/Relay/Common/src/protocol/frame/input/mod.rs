use crate::protocol::frame::headers::{Header, Headers};
use crate::protocol::frame::{CommandVec, FrameId};

#[derive(Debug, PartialEq, Clone)]
pub struct InFrame {
	pub frame_id: FrameId,
	pub headers: Headers,
	pub commands: CommandVec,
}
impl InFrame {
	pub const MAX_COMMAND_COUNT: usize = 64;

	pub fn new(frame_id: FrameId) -> Self {
		Self {
			frame_id,
			headers: Default::default(),
			commands: Default::default(),
		}
	}

	///
	///  Получить оригинальный frame_id
	/// - для повторно отосланных фреймов - id изначального фрейма
	/// - для всех остальных id фрейма
	///
	pub fn get_original_frame_id(&self) -> FrameId {
		match self.headers.first(Header::predicate_retransmit) {
			None => self.frame_id,
			Some(value) => value.original_frame_id,
		}
	}

	///
	/// Фрейм с надежной доставкой?
	///
	pub fn is_reliability(&self) -> bool {
		self.commands.iter().any(|f| f.channel.is_reliable())
	}
}