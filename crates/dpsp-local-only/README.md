# dpsp-local-only

A DPRun service provider that only handles connections on the local machine, useful for testing.

## Usage

This crate should be used together with the [dprun](../dprun) crate.

Create a LocalOnlyServer:

```rust
use dpsp_local_only::LocalOnlyServer;

let server = Arc::new(Mutex::new(
    LocalOnlyServer::new());
```

Create service provider instances with this `Arc<Mutex<LocalOnlyServer>>`:

```rust
use dpsp_local_only::LocalOnlySP;

dprun_options.service_provider_handler(
    Box::new(LocalOnlySP::new(Arc::clone(&server))));
```

## License

[GPL-3.0](../../LICENSE.md)
