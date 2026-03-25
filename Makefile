.PHONY: install test run redis

install:
	python3 -m venv .venv
	. .venv/bin/activate && pip install -e ".[dev]"

redis:
	docker compose up -d redis

run:
	. .venv/bin/activate && uvicorn app.main:app --reload

test:
	. .venv/bin/activate && pytest

