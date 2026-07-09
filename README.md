# Factory Game V2

## Tests

```bash
dotnet watch test tests.csproj --project tests.csproj -- -v quiet --nologo -l:"console;verbosity=normal"
```

## Telemetry

OpenTelemetry credentials are read from the environment at runtime.

For Honeycomb, set `FACTORY_GAME_OTEL_HEADERS` to the OTLP header string, for example:

```bash
FACTORY_GAME_OTEL_HEADERS='x-honeycomb-team=...'
```

To point a local build at a collector without a secret, override the OTLP endpoints instead:

```bash
FACTORY_GAME_OTEL_LOGS_ENDPOINT='http://localhost:4318/v1/logs'
FACTORY_GAME_OTEL_TRACES_ENDPOINT='http://localhost:4318/v1/traces'
```

If the headers variable is unset, the game will start telemetry without auth headers.
