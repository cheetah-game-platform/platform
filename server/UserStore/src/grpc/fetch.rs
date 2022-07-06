use crate::grpc::unwrap_request;
use crate::grpc::userstore::{
	fetch_server::Fetch, GetDoubleReply, GetDoubleRequest, GetLongReply, GetLongRequest,
	GetStringReply, GetStringRequest, Status as UserStoreStatus,
};
use crate::ydb::YDBFetch;
use cheetah_libraries_microservice::trace::trace;
use tonic::{Request, Response, Status};
use ydb::TableClient;

pub struct FetchService {
	fetch: YDBFetch,
	jwt_public_key: String,
}

impl FetchService {
	pub fn new(client: TableClient, jwt_public_key: String) -> Self {
		Self {
			fetch: YDBFetch::new(client),
			jwt_public_key,
		}
	}
}

#[tonic::async_trait]
impl Fetch for FetchService {
	async fn get_long(
		&self,
		request: Request<GetLongRequest>,
	) -> Result<Response<GetLongReply>, Status> {
		match unwrap_request(request, self.jwt_public_key.clone()) {
			Ok((user, args)) => match self.fetch.get(&user, &args.field_name).await {
				Ok(value) => Ok(Response::new(GetLongReply {
					value,
					status: UserStoreStatus::Ok as i32,
				})),
				Err(e) => e.lift(|s| GetLongReply {
					status: s as i32,
					value: 0,
				}),
			},
			Err(e) => Err(e),
		}
	}

	async fn get_double(
		&self,
		request: Request<GetDoubleRequest>,
	) -> Result<Response<GetDoubleReply>, Status> {
		match unwrap_request(request, self.jwt_public_key.clone()) {
			Ok((user, args)) => match self.fetch.get(&user, &args.field_name).await {
				Ok(value) => Ok(Response::new(GetDoubleReply {
					value,
					status: UserStoreStatus::Ok as i32,
				})),
				Err(e) => {
					trace("Fetch::get_double failed", &e);
					e.lift(|s| GetDoubleReply {
						status: s as i32,
						value: 0.0,
					})
				}
			},
			Err(e) => Err(e),
		}
	}

	async fn get_string(
		&self,
		request: Request<GetStringRequest>,
	) -> Result<Response<GetStringReply>, Status> {
		match unwrap_request(request, self.jwt_public_key.clone()) {
			Ok((user, args)) => match self.fetch.get(&user, &args.field_name).await {
				Ok(value) => Ok(Response::new(GetStringReply {
					value,
					status: UserStoreStatus::Ok as i32,
				})),
				Err(e) => {
					trace("Fetch::get_string failed", &e);
					e.lift(|s| GetStringReply {
						status: s as i32,
						value: "".into(),
					})
				}
			},
			Err(s) => Err(s),
		}
	}
}
