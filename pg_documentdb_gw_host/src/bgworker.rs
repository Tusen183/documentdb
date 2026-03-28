use pgrx::{bgworkers::*, prelude::*};
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use documentdb_gateway_core::{
    configuration::{DocumentDBSetupConfiguration, PgConfiguration, SetupConfiguration},
    postgres::{
        conn_mgmt::create_connection_pool_manager, create_query_catalog, DocumentDBDataClient,
    },
    run_gateway,
    service::TlsProvider,
    shutdown_controller::SHUTDOWN_CONTROLLER,
    startup::{create_postgres_object, get_service_context},
};

use crate::gucs::{
    PG_DOCUMENTDB_GATEWAY_DATABASE,
    PG_DOCUMENTDB_SETUP_CONFIGURATION,
    PG_DOCUMENTDB_GATEWAY_LOG_LEVEL,
};

pub fn init() {
    BackgroundWorkerBuilder::new("DocumentDB Gateway Host")
        .set_function("documentdb_gw_worker_main")
        .set_library("pg_documentdb_gw_host")
        .set_restart_time(Some(Duration::from_secs(1)))
        .set_start_time(BgWorkerStartTime::ConsistentState)
        .enable_spi_access()
        .load();
}

#[pg_guard]
#[no_mangle]
pub extern "C-unwind" fn documentdb_gw_worker_main(_arg: pg_sys::Datum) {
    BackgroundWorker::attach_signal_handlers(SignalWakeFlags::SIGHUP | SignalWakeFlags::SIGTERM);

    let database_name = String::from(
        PG_DOCUMENTDB_GATEWAY_DATABASE
            .get()
            .expect("GUC database not set")
            .to_str()
            .unwrap(),
    );

    let setup_configuration_file = String::from(
        PG_DOCUMENTDB_SETUP_CONFIGURATION
            .get()
            .expect("GUC setup configuration not set")
            .to_str()
            .unwrap(),
    );

    // Get log level from GUC before entering async runtime
    let log_level = PG_DOCUMENTDB_GATEWAY_LOG_LEVEL
        .get()
        .and_then(|s| s.to_str().ok().map(|s| s.to_string()))
        .unwrap_or_else(|| "info".to_string());

    BackgroundWorker::connect_worker_to_spi(Some(database_name.as_str()), None);

    let shutdown_token = SHUTDOWN_CONTROLLER.token();
    let worker_name = BackgroundWorker::get_name();

    // now start the gw on a worker thread.
    let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();

    tokio_runtime.spawn(async move {
        run_docdb_gateway(setup_configuration_file.as_str(), log_level).await;
        SHUTDOWN_CONTROLLER.shutdown();
    });

    // wake up every second or if we received a SIGTERM
    while BackgroundWorker::wait_latch(Some(Duration::from_secs(1))) {
        if shutdown_token.is_cancelled() {
            break;
        }
    }

    SHUTDOWN_CONTROLLER.shutdown();
    tokio_runtime.shutdown_timeout(Duration::from_secs(1));
    log!("{} stopped", worker_name);
}

async fn run_docdb_gateway(setup_configuration_file: &str, log_level: String) {
    let cfg_file = std::path::PathBuf::from(setup_configuration_file);

    let shutdown_token = SHUTDOWN_CONTROLLER.token();

    let setup_configuration =
        DocumentDBSetupConfiguration::new(&cfg_file).expect("Failed to load configuration.");

    let tls_provider = TlsProvider::new(
        SetupConfiguration::certificate_options(&setup_configuration),
        None,
        None,
    )
    .await
    .expect("Failed to create TLS provider.");

    let connection_pool_manager = create_connection_pool_manager(
        create_query_catalog(),
        Box::new(setup_configuration.clone()),
    )
    .await;
    // Initialize tracing subscriber to write gateway logs to a separate file
    // Use log level passed from main thread (already retrieved from GUC)
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&log_level));

    // Use PGDATA/log directory for gateway logs
    let pgdata = std::env::var("PGDATA").unwrap_or_else(|_| "/tmp".to_string());
    let log_dir_path = std::path::PathBuf::from(&pgdata).join("log");

    // Ensure log directory exists
    if let Err(e) = std::fs::create_dir_all(&log_dir_path) {
        eprintln!("Warning: Failed to create log directory {:?}: {}", log_dir_path, e);

    }

    let log_file_path = log_dir_path.join("gateway.log");

    // Create file appender that writes to $PGDATA/log/gateway.log
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)
        .expect("Failed to open gateway log file");

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::sync::Mutex::new(file))
                .with_ansi(false)
                .with_target(true)
                .with_thread_ids(true)
        )
        .init();

    tracing::info!("Gateway logs writing to: {:?}", log_file_path);
    tracing::info!("Gateway log level: {}", log_level);
    tracing::info!("Starting server with configuration: {setup_configuration:?}");

    let dynamic_configuration = create_postgres_object(
        || async {
            PgConfiguration::new(
                &setup_configuration,
                Arc::clone(&connection_pool_manager),
                vec!["documentdb.".to_string()],
            )
            .await
        },
        &setup_configuration,
    )
    .await;

    let service_context = get_service_context(
        Box::new(setup_configuration),
        dynamic_configuration,
        connection_pool_manager,
        tls_provider,
    );

    run_gateway::<DocumentDBDataClient>(service_context, None, shutdown_token)
        .await
        .unwrap();
}
