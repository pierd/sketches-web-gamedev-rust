# sketches-web-gamedev-rust
Assorted pieces that together should allow for Web game development in Rust

## rust-webgl

Most of the code from [rustwasm webgl example](https://rustwasm.github.io/wasm-bindgen/examples/webgl.html). Using `web-sys` to render something in WebGL + some basic event handling. Running:

```
$ npm install
$ npm run serve
$ open http://localhost:8080
```

To get deployable build run:
```
$ npx webpack
```

## TODO

- https://stackoverflow.com/questions/4037212/html-canvas-full-screen
- https://www.html5rocks.com/en/mobile/touch/
- https://coderwall.com/p/iygcpa/gameloop-the-correct-way
