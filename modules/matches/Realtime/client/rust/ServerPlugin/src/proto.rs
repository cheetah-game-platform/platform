pub mod matches {
	pub mod realtime {
		pub mod shared {
			tonic::include_proto!("cheetah.matches.realtime.shared");
		}

		pub mod internal {
			tonic::include_proto!("cheetah.matches.realtime.internal");
		}
	}
}
