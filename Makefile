.PHONY: dev dev-backend dev-frontend

# Hot-reload backend (requires: cargo install cargo-watch)
dev-backend:
	cargo watch -x 'run --bin docx-jats-backend'

# Start SvelteKit dev server
dev-frontend:
	cd frontend && npm run dev

# Run both in parallel
dev:
	$(MAKE) -j2 dev-backend dev-frontend
