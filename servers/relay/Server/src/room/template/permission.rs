use std::collections::{HashMap, HashSet};

use fnv::FnvBuildHasher;

use cheetah_relay_common::constants::{FieldIdType, GameObjectTemplateType};
use cheetah_relay_common::room::access::AccessGroups;

use crate::room::template::config::{Permission, PermissionGroup, Permissions};
use crate::room::types::FieldType;

#[derive(Debug)]
pub struct PermissionManager {
	templates: HashMap<GameObjectTemplateType, Vec<PermissionGroup>, FnvBuildHasher>,
	fields: HashMap<PermissionFieldKey, Vec<PermissionGroup>, FnvBuildHasher>,
	cache: HashMap<PermissionCachedFieldKey, Permission, FnvBuildHasher>,
	write_access_template: HashSet<GameObjectTemplateType, FnvBuildHasher>,
	write_access_fields: HashSet<PermissionFieldKey, FnvBuildHasher>,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
struct PermissionFieldKey {
	template: GameObjectTemplateType,
	field_id: FieldIdType,
	field_type: FieldType,
}

#[derive(Hash, Eq, PartialEq, Debug)]
struct PermissionCachedFieldKey {
	field_key: PermissionFieldKey,
	group: AccessGroups,
}

impl PermissionManager {
	pub fn new(permission: &Permissions) -> Self {
		let mut result = Self {
			templates: Default::default(),
			fields: Default::default(),
			cache: Default::default(),
			write_access_template: Default::default(),
			write_access_fields: Default::default(),
		};

		for template in &permission.templates {
			if template.groups.iter().find(|t| t.permission > Permission::Ro).is_some() {
				result.write_access_template.insert(template.template);
			}
			result.templates.insert(template.template.clone(), template.groups.clone());

			for field in &template.fields {
				let key = PermissionFieldKey {
					template: template.template,
					field_id: field.field_id,
					field_type: field.field_type.clone(),
				};

				if field.groups.iter().find(|t| t.permission > Permission::Ro).is_some() {
					result.write_access_fields.insert(key.clone());
				}

				result.fields.insert(key, field.groups.clone());
			}
		}

		result
	}

	///
	/// Доступен ли объект на запись другим пользователем кроме создателя
	///
	pub fn has_write_access(&mut self, template: GameObjectTemplateType, field_id: FieldIdType, field_type: FieldType) -> bool {
		self.write_access_template.contains(&template)
			|| self.write_access_fields.contains(&PermissionFieldKey {
				template,
				field_id,
				field_type,
			})
	}

	pub fn get_permission(
		&mut self,
		template: GameObjectTemplateType,
		field_id: FieldIdType,
		field_type: FieldType,
		user_group: AccessGroups,
	) -> Permission {
		let field_key = PermissionFieldKey {
			template,
			field_id,
			field_type,
		};

		let cached_key = PermissionCachedFieldKey {
			field_key,
			group: user_group,
		};

		match self.cache.get(&cached_key) {
			None => {
				let permission = match self.fields.get(&cached_key.field_key) {
					None => match self.templates.get(&template) {
						None => &Permission::Ro,
						Some(permissions) => PermissionManager::get_permission_by_group(user_group, permissions),
					},
					Some(permissions) => PermissionManager::get_permission_by_group(user_group, permissions),
				};
				self.cache.insert(cached_key, *permission);
				*permission
			}
			Some(permission) => *permission,
		}
	}

	fn get_permission_by_group(user_group: AccessGroups, groups: &Vec<PermissionGroup>) -> &Permission {
		groups
			.iter()
			.find(|p| p.group.contains_any(&user_group))
			.map_or(&Permission::Ro, |p| &p.permission)
	}
}

#[cfg(test)]
mod tests {
	use cheetah_relay_common::room::access::AccessGroups;

	use crate::room::template::config::{Permission, PermissionField, PermissionGroup, Permissions, TemplatePermission};
	use crate::room::template::permission::PermissionManager;
	use crate::room::types::FieldType;

	#[test]
	fn should_default_permission() {
		let mut permissions_manager = PermissionManager::new(&Permissions::default());
		assert_eq!(
			permissions_manager.get_permission(10, 10, FieldType::Long, AccessGroups(0)),
			Permission::Ro
		);
	}

	#[test]
	fn should_permission_for_template_by_group() {
		let mut permissions = Permissions::default();
		let mut template_permission = TemplatePermission {
			template: 10,
			groups: Default::default(),
			fields: Default::default(),
		};
		template_permission.groups.push(PermissionGroup {
			group: AccessGroups(0b11),
			permission: Permission::Rw,
		});

		template_permission.groups.push(PermissionGroup {
			group: AccessGroups(0b1000),
			permission: Permission::Deny,
		});
		permissions.templates.push(template_permission);

		let mut permissions_manager = PermissionManager::new(&permissions);

		assert_eq!(
			permissions_manager.get_permission(10, 10, FieldType::Long, AccessGroups(0b01)),
			Permission::Rw
		);
		assert_eq!(
			permissions_manager.get_permission(10, 10, FieldType::Long, AccessGroups(0b1000)),
			Permission::Deny
		);
	}

	#[test]
	fn should_permission_for_fields() {
		let mut permissions = Permissions::default();
		let mut template_permission = TemplatePermission {
			template: 10,
			groups: Default::default(),
			fields: Default::default(),
		};
		template_permission.groups.push(PermissionGroup {
			group: AccessGroups(0b11),
			permission: Permission::Deny,
		});

		template_permission.fields.push(PermissionField {
			field_id: 15,
			field_type: FieldType::Long,
			groups: vec![PermissionGroup {
				group: AccessGroups(0b11),
				permission: Permission::Rw,
			}],
		});
		permissions.templates.push(template_permission);

		let mut permissions_manager = PermissionManager::new(&permissions);

		assert_eq!(
			permissions_manager.get_permission(10, 10, FieldType::Long, AccessGroups(0b01)),
			Permission::Deny
		);
		assert_eq!(
			permissions_manager.get_permission(10, 15, FieldType::Long, AccessGroups(0b01)),
			Permission::Rw
		);
	}

	#[test]
	fn should_cache_permission_for_fields() {
		let mut permissions = Permissions::default();
		let mut template_permission = TemplatePermission {
			template: 10,
			groups: Default::default(),
			fields: Default::default(),
		};
		template_permission.groups.push(PermissionGroup {
			group: AccessGroups(0b11),
			permission: Permission::Deny,
		});

		template_permission.fields.push(PermissionField {
			field_id: 15,
			field_type: FieldType::Long,
			groups: vec![PermissionGroup {
				group: AccessGroups(0b11),
				permission: Permission::Rw,
			}],
		});
		permissions.templates.push(template_permission);

		let mut permissions_manager = PermissionManager::new(&permissions);
		// прогреваем кеш
		permissions_manager.get_permission(10, 10, FieldType::Long, AccessGroups(0b01));
		permissions_manager.get_permission(10, 15, FieldType::Long, AccessGroups(0b01));
		// удаляем исходные данные
		permissions_manager.fields.clear();
		permissions_manager.templates.clear();

		assert_eq!(
			permissions_manager.get_permission(10, 10, FieldType::Long, AccessGroups(0b01)),
			Permission::Deny
		);
		assert_eq!(
			permissions_manager.get_permission(10, 15, FieldType::Long, AccessGroups(0b01)),
			Permission::Rw
		);
	}

	#[test]
	fn should_not_has_write_access_by_default() {
		let permissions = Permissions::default();
		let mut permissions_manager = PermissionManager::new(&permissions);
		assert!(!permissions_manager.has_write_access(10, 100, FieldType::Long));
	}

	#[test]
	fn should_has_write_access_if_object_has_write_permission() {
		let mut permissions = Permissions::default();
		permissions.templates.push(TemplatePermission {
			template: 10,
			groups: vec![PermissionGroup {
				group: Default::default(),
				permission: Permission::Rw,
			}],
			fields: vec![],
		});
		let mut permissions_manager = PermissionManager::new(&permissions);
		assert!(permissions_manager.has_write_access(10, 100, FieldType::Long));
	}

	#[test]
	fn should_not_has_write_access_if_object_has_read_permission() {
		let mut permissions = Permissions::default();
		permissions.templates.push(TemplatePermission {
			template: 10,
			groups: vec![PermissionGroup {
				group: Default::default(),
				permission: Permission::Ro,
			}],
			fields: vec![],
		});
		let mut permissions_manager = PermissionManager::new(&permissions);
		assert!(!permissions_manager.has_write_access(10, 100, FieldType::Long));
	}

	#[test]
	fn should_has_write_access_if_object_has_field_with_write_permission() {
		let mut permissions = Permissions::default();
		permissions.templates.push(TemplatePermission {
			template: 10,
			groups: vec![],
			fields: vec![PermissionField {
				field_id: 100,
				field_type: FieldType::Long,
				groups: vec![PermissionGroup {
					group: Default::default(),
					permission: Permission::Rw,
				}],
			}],
		});
		let mut permissions_manager = PermissionManager::new(&permissions);
		assert!(permissions_manager.has_write_access(10, 100, FieldType::Long));
	}
}
