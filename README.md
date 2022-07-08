# Local Environment

This is a cross-platform tool to spinup services for development from a unified configuration file similar to Docker Compose but the services run directly on
the host instead of container. Thus no virtual machine is required on macOS and Windows.

## Usage

### Sample configurations

```yaml
# locenv-services.yml
sample-c:
  repository:
    type: git
    uri: https://github.com/locenv/sample-c.git
```

### Start services

```sh
locenv up
```

### Install a module

```sh
locenv mod install github:locenv/mod-autoconf
```

### Update services to latest version

```sh
locenv pull
```

## Script & module limitations

- Coroutines is not supported due to it use `longjmp`, which causes Rust objects to leak.
  See [here](https://stackoverflow.com/questions/34303507/lua-coroutines-setjmp-longjmp-clobbering) for detailed information.
- Lua error does not handled properly until [this](https://github.com/rust-lang/rust/issues/74990) completed. However we still recommended to use the standard
  Lua error to raise the error.

## License

MIT
