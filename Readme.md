# Rust admin panel [In process]

TODO:
1) Добавить возможность загружать аватарки пользователям;
2) Исследовать и возможно добавить опциональное хранилище KeyValue написанное на Rust как альтернативу Redis, для ускорения работы в одно-серверных системах;
3) Добавить систему логирования с возможностью доступа через web ui;
4) Написать генерируемую документацию и позаботиться о доступе к ней через web ui;
5) Сформировать CI/CD через файл для одно-серверных систем с запуском без докера;
6) Сформировать CI/CD через файл для одно-серверных систем с запуском через docker-compose;
7) Сформировать CI/CD через файл для k8s систем;
8) Сформировать CI/CD GitLab для одно-серверных систем с запуском без докера;
9) Сформировать CI/CD GitLab для одно-серверных систем с запуском через docker-compose;
10) Сформировать CI/CD GitLab для k8s систем;
11) Провести нагрузочное тестирование;
12) Постараться оптимизировать скорость исполнения ещё сильнее и сократить расход памяти путём уменьшения размеров типов переменных, там где это возможно;
13) Проверить безопасность подключенных библиотек вручную просмотрев их код;
14) Переименовать сборку в что-то уникальное, например взять Laravel за основу и назвать сборку Ralaver;
15) Выпустить первый релиз;
16) Оформить статью на habr, а так же проконсультироваться с безопасниками касательно сборки.

## Среда разработки

Среда разработки состоит из:
1) rust:1.82.0
2) nodejs:22.13.1

### Команды бекенда
Перед запуском проекта создайте .env файл с переменными окружения из файла .env.example:
```shell
cp .env.example .env
```

Команда запуска среды разработки:
```shell
docker compose -f dev.docker-compose.yaml up -d
```

Команда остановки среды разработки:
```shell
docker compose -f dev.docker-compose.yaml down
```

Команда входа в среду разработки:
```shell
docker compose -f dev.docker-compose.yaml exec app bash
```
Эта команда будет полезна, например: для применения миграций.

Команда перезапуска среды разработки вместе со сборкой исходников:
```shell
docker compose -f dev.docker-compose.yaml down && docker compose -f dev.docker-compose.yaml up --build -d
```

Команда запуска приложения в среде разработки:
```shell
docker compose -f dev.docker-compose.yaml exec app cargo run
```

#### Миграции базы данных
Команда запуска миграций:
```shell
cargo run --bin migrate up
```
```shell
docker compose -f dev.docker-compose.yaml exec app cargo run --bin migrate up
```

Команда отката миграций:
```shell
cargo run --bin migrate down
```
```shell
docker compose -f dev.docker-compose.yaml exec app cargo run --bin migrate down
```




### Команды фронтенда
Команда для установки зависимостей фронтенда:
```shell
docker compose -f dev.docker-compose.yaml exec app npm i
```

Команда для сборки фронтенда (перед сборкой не забудьте установить зависимости):
```shell
docker compose -f dev.docker-compose.yaml exec app npm run build
```

Команда для перезаписи root владельца файлов:
```shell
sudo chown -R $UID:$UID .
```
