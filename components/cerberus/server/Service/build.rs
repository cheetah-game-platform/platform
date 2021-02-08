use std::io::Error;

fn main() -> Result<(), Error> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile(
            &[
                "../../proto/service.external.proto",
                "../../proto/service.internal.proto",
            ],
            &["../../proto/"],
        )?;

    Result::Ok(())
}