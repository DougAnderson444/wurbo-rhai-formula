build:
  cargo component build
  cargo component build --release

prev:
  cd examples/sveltekit && npm run build && npm run preview -- --open

all: build prev  
