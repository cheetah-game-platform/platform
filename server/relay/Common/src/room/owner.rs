use crate::room::UserId;
use serde::{Deserialize, Serialize};

///
/// владелец - клиент или root
///
#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum ObjectOwner {
	Root,
	User(UserId),
}