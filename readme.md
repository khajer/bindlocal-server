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

# concept
- open tcp 2 port that
- 1 tcp (http) -> receive http request
- 2 tcp -> socket -> communicate to client


step 1 (connect to localhost)

```web browser -> connl server -> connl client -> localhost (dev)```

step 2 (send data to web browser)

```localhost(dev) -> connl client -> connl server -> web browser```

# roadmap
- [✓] test first on react-app
- [✓] test on svelte completely


# official website
[https://connl.io](https://connl.io)
