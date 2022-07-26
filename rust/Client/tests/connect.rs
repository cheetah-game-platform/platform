use std::thread;
use std::time::Duration;

use cheetah_client::ffi::execute_with_client;
use cheetah_common::network::client::{ConnectionStatus, DisconnectedReason};
use cheetah_common::protocol::disconnect::timeout::DisconnectByTimeout;

use crate::helpers::helper::IntegrationTestHelper;
use crate::helpers::server::IntegrationTestServerBuilder;

pub mod helpers;

#[test]
fn should_connect_to_server() {
	let builder = IntegrationTestServerBuilder::default();
	let mut helper = IntegrationTestHelper::new(builder);
	let (member_id, private_key) = helper.create_member();
	let client = helper.create_client(member_id, &private_key);
	helper.wait_udp();
	execute_with_client(client, |api| {
		let status = api.get_connection_status().unwrap();
		assert_eq!(status, ConnectionStatus::Connected);
		Ok(())
	});
}

#[test]
fn should_disconnect_when_server_closed() {
	let builder = IntegrationTestServerBuilder::default();

	let mut helper = IntegrationTestHelper::new(builder);
	let (member_id, private_key) = helper.create_member();
	let client = helper.create_client(member_id, &private_key);
	helper.wait_udp();

	execute_with_client(client, |api| {
		let status = api.get_connection_status().unwrap();
		assert_eq!(status, ConnectionStatus::Connected);
		Ok(())
	});

	drop(helper);

	execute_with_client(client, |api| {
		api.set_protocol_time_offset(DisconnectByTimeout::TIMEOUT).unwrap();
		Ok(())
	});
	thread::sleep(Duration::from_millis(100));

	execute_with_client(client, |api| {
		let status = api.get_connection_status().unwrap();
		assert!(matches!(status, ConnectionStatus::Disconnected(DisconnectedReason::ByTimeout)));
		Ok(())
	});
}
