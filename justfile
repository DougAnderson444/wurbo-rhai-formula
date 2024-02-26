build:
  cargo component build

prev:
  cd examples/sveltekit && npm run build && npm run preview -- --open

all: build prev  
