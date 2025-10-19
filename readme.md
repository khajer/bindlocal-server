# bindlocal-server
proxy server app.

# docker build
```sh
docker build . -t connl:latest
```

# how to build with cmd
```sh
cargo build --release
```

# how to run the server
```sh
./connl
```

# stack
- tcp 2 port
- 1 tcp (http) -> receive http request
- 2 tcp -> socket -> communicate to client


# roadmap
- [✓] test first on react-app
- [✓] test on svelte completely
