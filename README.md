# Local Environment

This is a cross-platform tool to spinup services for development from a unified configuration file similar to Docker Compose but the services run directly on the host instead of container. Thus no virtual machine is required on macOS and Windows.

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

### Update services to latest version

```sh
locenv pull
```

## License

MIT
