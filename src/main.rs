use crate::error::app::AppError;
use crate::grpc::Service;
use crate::proto::parapluie::parapluie_db_server::ParapluieDbServer;
use crate::proto::parapluie::FILE_DESCRIPTOR_SET;
use crate::repository::{Processor, Repository};
use rusqlite::Connection;
use tokio::sync::mpsc;
use tokio::{select, task};
use tonic::transport::Server;
use tracing_subscriber::FmtSubscriber;

mod repository;
mod proto;
mod grpc;
mod model;
mod error;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let x =1;
    let subscriber = FmtSubscriber::builder().finish();
    tracing::subscriber::set_global_default(subscriber)?;


    let (sender, receiver) = mpsc::channel(32);
    let repository = Repository::new(sender).await;
    let grpc_service = Service::new(repository);
    let listen_addr = "0.0.0.0:50051".parse()?;
    let server = ParapluieDbServer::new(grpc_service);

    let sqlite_task = task::spawn_blocking(move || -> Result<(), AppError> {
        // NOTE: The connection must be opened in the same thread as the processor.
        let conn = Connection::open("/tmp/db.sqlite")?;

        conn.pragma_update(None, "journal_mode", "WAL")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS item (
                partition_key TEXT NOT NULL,
                sort_key TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                version INTEGER NOT NULL,
                value BLOB NOT NULL,
                PRIMARY KEY (partition_key, sort_key)
            )",
            [],
        )?;

        let processor = Processor::new(conn, receiver);
        processor.blocking_process_tasks()?;

        Ok(())
    });

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .include_reflection_service(true)
        .build_v1()?;

    // NOTE: Once the server stops, the processor will stop as well because the sender will be
    // dropped.
    let grpc_server = Server::builder()
        .add_service(server)
        .add_service(reflection_service)
        .serve(listen_addr);

    select! {
        result = grpc_server => {
            println!("gRPC server stopped: ${:?}", result);
            result?
        }
        result = sqlite_task => {
            println!("SQLite task stopped: ${:?}", result);
            result??
        }
    }

    Ok(())
}