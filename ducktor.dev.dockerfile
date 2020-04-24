FROM mcr.microsoft.com/dotnet/core/sdk:3.1 AS ducktor

WORKDIR /ducktor
ENTRYPOINT dotnet run --project Debugger --urls "http://0.0.0.0:5000"