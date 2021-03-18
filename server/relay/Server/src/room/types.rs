use serde::{Deserialize, Serialize};
///
/// Тип данных поля
///
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Hash, Eq, Copy)]
pub enum FieldType {
	#[serde(rename = "long")]
	Long,
	#[serde(rename = "float")]
	Float,
	#[serde(rename = "structure")]
	Structure,
	#[serde(rename = "event")]
	Event,
}