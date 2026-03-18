.PHONY: dev build test lint lint-spec clean format

dev:
	@echo "Starting development servers..."
	@trap 'kill 0' EXIT; \
	cd backend && cargo run -p crm-demo & \
	cd frontend && npm run dev & \
	wait

build:
	cd backend && cargo build --release
	cd frontend && npm run build

test:
	cd backend && cargo test
	cd frontend && npm test -- --run

lint:
	cd backend && cargo fmt --check
	cd backend && cargo clippy -- -D warnings
	cd frontend && npm run lint
	cd frontend && npm run check
	cd spec && npm run lint

lint-spec:
	cd spec && npm run lint

format:
	cd backend && cargo fmt
	cd frontend && npm run format

clean:
	cd backend && cargo clean
	cd frontend && rm -rf .svelte-kit build node_modules/.vite
