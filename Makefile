.PHONY: dev build test lint lint-spec clean format e2e gallery-dev tutorial-people-app

dev:
	@echo "Starting development servers..."
	@trap 'kill 0' EXIT; \
	cd backend && cargo run -p crm-demo & \
	cd frontend && npm run dev & \
	wait

gallery-dev:
	@echo "Starting gallery-demo on :3002..."
	@trap 'kill 0' EXIT; \
	cd backend && cargo run -p gallery-demo & \
	wait

tutorial-people-app:
	@echo "Starting tutorial-people-app on :3003 (frontend dev on :5173)..."
	@trap 'kill 0' EXIT; \
	cd backend && cargo run -p tutorial-people-app & \
	cd frontend && npm run dev & \
	wait

build:
	cd frontend && npm run build
	cd backend && cargo build --release

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

e2e:
	cd frontend && npx playwright test --config playwright.e2e.config.ts tests/e2e/

clean:
	cd backend && cargo clean
	cd frontend && rm -rf .svelte-kit build node_modules/.vite
