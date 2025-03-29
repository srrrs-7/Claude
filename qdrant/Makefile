.PHONY: qdrant restart logs clean

qdrant:
	docker-compose up -d

restart:
	docker-compose down && docker-compose up -d

logs:
	docker-compose logs -f

clean:
	docker-compose down -v
