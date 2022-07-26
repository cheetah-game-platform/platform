use std::future::Future;

use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::{Channel, Endpoint, Error, Server, Uri};
use tonic::{Request, Response, Status};
use tower::service_fn;

use crate::proto::matches::realtime::internal::internal_server::{Internal, InternalServer};
use crate::proto::matches::realtime::internal::{
	CreateMemberRequest, CreateMemberResponse, CreateSuperMemberRequest, DeleteMemberRequest, DeleteMemberResponse, DeleteRoomRequest,
	DeleteRoomResponse, EmptyRequest, GetRoomInfoRequest, GetRoomInfoResponse, MarkRoomAsReadyRequest, MarkRoomAsReadyResponse, ProbeRequest,
	ProbeResponse, PutForwardedCommandConfigRequest, PutForwardedCommandConfigResponse, RoomIdResponse, RoomLifecycleResponse, RoomTemplate,
	UpdateRoomPermissionsRequest, UpdateRoomPermissionsResponse,
};

pub struct RealtimeStub<CreatedEventStubFunc, Fut>
where
	CreatedEventStubFunc: Fn(Sender<Result<RoomLifecycleResponse, Status>>) -> Fut + Send + Sync + 'static,
	Fut: Future<Output = ()> + 'static + Send + Sync,
{
	pub created_event_stub_function: CreatedEventStubFunc,
}

#[tonic::async_trait]
#[allow(clippy::unreachable)]
impl<CreatedEventStubFunc, Fut> Internal for RealtimeStub<CreatedEventStubFunc, Fut>
where
	CreatedEventStubFunc: Fn(Sender<Result<RoomLifecycleResponse, Status>>) -> Fut + Send + Sync + 'static,
	Fut: Future<Output = ()> + 'static + Send + Sync,
{
	async fn create_room(&self, _request: Request<RoomTemplate>) -> Result<Response<RoomIdResponse>, Status> {
		unreachable!()
	}

	async fn create_member(&self, _request: Request<CreateMemberRequest>) -> Result<Response<CreateMemberResponse>, Status> {
		unreachable!()
	}

	async fn delete_member(&self, _request: Request<DeleteMemberRequest>) -> Result<Response<DeleteMemberResponse>, Status> {
		unreachable!()
	}

	async fn create_super_member(&self, _request: Request<CreateSuperMemberRequest>) -> Result<Response<CreateMemberResponse>, Status> {
		unreachable!()
	}

	async fn probe(&self, _request: Request<ProbeRequest>) -> Result<Response<ProbeResponse>, Status> {
		unreachable!()
	}

	type WatchRoomLifecycleEventStream = ReceiverStream<Result<RoomLifecycleResponse, Status>>;
	async fn watch_room_lifecycle_event(&self, _request: Request<EmptyRequest>) -> Result<Response<Self::WatchRoomLifecycleEventStream>, Status> {
		let (tx, rx) = mpsc::channel(64);
		(self.created_event_stub_function)(tx).await;
		Ok(Response::new(ReceiverStream::new(rx)))
	}

	async fn delete_room(&self, _request: Request<DeleteRoomRequest>) -> Result<Response<DeleteRoomResponse>, Status> {
		unreachable!()
	}

	async fn put_forwarded_command_config(
		&self,
		_request: Request<PutForwardedCommandConfigRequest>,
	) -> Result<Response<PutForwardedCommandConfigResponse>, Status> {
		unreachable!()
	}

	async fn mark_room_as_ready(&self, _request: Request<MarkRoomAsReadyRequest>) -> Result<Response<MarkRoomAsReadyResponse>, Status> {
		unreachable!()
	}

	async fn get_room_info(&self, _request: Request<GetRoomInfoRequest>) -> Result<Response<GetRoomInfoResponse>, Status> {
		unreachable!()
	}

	async fn update_room_permissions(
		&self,
		_request: Request<UpdateRoomPermissionsRequest>,
	) -> Result<Response<UpdateRoomPermissionsResponse>, Status> {
		unreachable!()
	}
}

pub fn create_stub_server<F, Fut>(f: F) -> (Runtime, JoinHandle<Result<(), Error>>, Channel)
where
	F: Fn(Sender<Result<RoomLifecycleResponse, Status>>) -> Fut + Send + Sync + 'static,
	Fut: Future<Output = ()> + 'static + Send + Sync,
{
	let (client, server) = tokio::io::duplex(1024);

	let service = RealtimeStub {
		created_event_stub_function: f,
	};
	let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(2).build().unwrap();

	let server_handler = runtime.spawn(async move {
		Server::builder()
			.add_service(InternalServer::new(service))
			.serve_with_incoming(futures::stream::iter(vec![Ok::<_, std::io::Error>(server)]))
			.await
	});

	let mut client = Some(client);
	let channel = runtime.block_on(async move {
		Endpoint::try_from("http://[::]:50051")
			.unwrap()
			.connect_with_connector(service_fn(move |_: Uri| {
				let client = client.take();
				async move {
					if let Some(client) = client {
						Ok(client)
					} else {
						Err(std::io::Error::new(std::io::ErrorKind::Other, "Client already taken"))
					}
				}
			}))
			.await
	});

	(runtime, server_handler, channel.unwrap())
}
