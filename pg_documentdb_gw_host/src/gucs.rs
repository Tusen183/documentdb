use pgrx::ffi::CString;
use pgrx::{GucContext, GucFlags, GucRegistry, GucSetting};

pub(crate) static PG_DOCUMENTDB_GATEWAY_DATABASE: GucSetting<Option<CString>> =
    GucSetting::<Option<CString>>::new(None);

pub(crate) static PG_DOCUMENTDB_SETUP_CONFIGURATION: GucSetting<Option<CString>> =
    GucSetting::<Option<CString>>::new(None);

pub(crate) static PG_DOCUMENTDB_GATEWAY_LOG_LEVEL: GucSetting<Option<CString>> =
    GucSetting::<Option<CString>>::new(None);

pub(crate) static PG_DOCUMENTDB_GATEWAY_LOG_PRETTY_PRINT: GucSetting<bool> =
    GucSetting::<bool>::new(true);

pub fn init() {
    GucRegistry::define_string_guc(
        c"documentdb_gateway.database",
        c"The database that the pg_documentdb_gateway BGWorker will connect to",
        c"This should be the database that you ran `CREATE EXTENSION pg_documentdb_gateway` in",
        &PG_DOCUMENTDB_GATEWAY_DATABASE,
        GucContext::Postmaster,
        GucFlags::SUPERUSER_ONLY,
    );
    GucRegistry::define_string_guc(
        c"documentdb_gateway.setup_configuration_file",
        c"The setup configuration file for the pg_documentdb_gateway BGWorker",
        c"This should be the path to the setup configuration file",
        &PG_DOCUMENTDB_SETUP_CONFIGURATION,
        GucContext::Postmaster,
        GucFlags::SUPERUSER_ONLY,
    );
    GucRegistry::define_string_guc(
        c"documentdb_gateway.log_level",
        c"Log level for the gateway (trace, debug, info, warn, error)",
        c"Controls the verbosity of gateway logs. Default: info",
        &PG_DOCUMENTDB_GATEWAY_LOG_LEVEL,
        GucContext::Sighup,  // Can be changed with pg_reload_conf()
        GucFlags::default(),
    );
    GucRegistry::define_bool_guc(
        c"documentdb_gateway.log_pretty_print",
        c"Pretty print BSON documents in logs",
        c"When enabled, BSON documents are displayed as readable JSON instead of hex. Default: true",
        &PG_DOCUMENTDB_GATEWAY_LOG_PRETTY_PRINT,
        GucContext::Sighup,  // Can be changed with pg_reload_conf()
        GucFlags::default(),
    );
}
