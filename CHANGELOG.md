# 0.0.6
- Duplicate query keys ARE detected and forbidden (see 0.0.4 version)
- Update README.md and docker-compose for latest version pull

# 0.0.5
- Add README.md
- Add the license
- Pushing Docker images of cthulhu 0.0.5 to GitHub
- Add proxy_test.sh script for the demo

# 0.0.4
- Add request forward.
- Add docker compose infra and fastapi demo.

- Declared but absent JSON body params are enforced
- Declared but absent QUERY params are not enforced
- Duplicate query keys are not detected keeps the last value (to be modified, as the application might pick the first one).

# 0.0.3
- Add docker with multi stage build with locked version.
- Add Cargo.lock to lock the exact versions of the dependencies.

# 0.0.2
- Remove the concept of data types since we receive only string data, to restrict input to numbers, we simply need to allow only numeric characters.
- Add request validation system.

# 0.0.1
- Add project structure.
- Add read conf functionality.
- Add debug HTTP requests from the port specified in the conf.