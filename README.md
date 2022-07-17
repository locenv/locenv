# Local Environment
[![CI](https://github.com/locenv/locenv/actions/workflows/ci.yml/badge.svg)](https://github.com/locenv/locenv/actions/workflows/ci.yml)
![GitHub commit activity](https://img.shields.io/github/commit-activity/m/locenv/locenv)
![GitHub repo size](https://img.shields.io/github/repo-size/locenv/locenv)

This is a cross-platform tool to spinup services for development from a unified configuration file similar to Docker Compose but the services run directly on
the host instead of container. Thus no virtual machine is required on macOS and Windows.

**This project is not fully functional yet and it is under development.**

## Usage

### Sample configurations

```yaml
# locenv-services.yml
configurations:
  sample-c:
    repository:
      uri: https://github.com/locenv/sample-c.git
      type: git
instances:
  sample-c:
    configuration: sample-c
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
