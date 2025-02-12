# Rust admin panel [In process]

## Среда разработки

Среда разработки состоит из:
1) rust:1.82.0
2) nodejs:22.13.1
3) diesel_cli - is rust package

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
docker compose -f dev.docker-compose.yaml exec dev_tools bash
```
Эта команда будет полезна, например: для применения миграций.

Команда перезапуска среды разработки вместе со сборкой исходников:
```shell
docker compose -f dev.docker-compose.yaml down && docker compose -f dev.docker-compose.yaml up --build -d
```

#### Миграции базы данных
Команда запуска миграций:
```shell
diesel migration run
```
```shell
docker compose -f dev.docker-compose.yaml exec dev_tools diesel migration run
```

Команда отката миграций:
```shell
diesel migration revert
```
```shell
docker compose -f dev.docker-compose.yaml exec dev_tools diesel migration revert
```

Команда создания миграций:
```shell
diesel migration generate create_users
```
```shell
docker compose -f dev.docker-compose.yaml exec dev_tools diesel migration generate create_users
```




### Команды фронтенда
Команда для установки зависимостей фронтенда:
```shell
docker compose -f dev.docker-compose.yaml exec dev_tools npm i
```

Команда для сборки фронтенда (перед сборкой не забудьте установить зависимости):
```shell
docker compose -f dev.docker-compose.yaml exec dev_tools npm run build
```

Команда для перезаписи root владельца файлов:
```shell
sudo chown -R $UID:$UID .
```

## План разработки:
1) Исследование общепринятых правил построения сайтов на actix (в случае отсутсвия конкретных исчерпывающих подходов будет принят подход Laravel);
2) Формирование базовой архитектуры и структуры каталогов;
3) Реализация базового отображения шаблонов;
4) -> Реализация базовых миграций и посева данных;
5) Реализация базового хранилища;
6) Реализация базовых логов;
7) Реализация базовых конфигураций;
8) Реализация базовых роутов;
9) Реализация базовых языков;
10) Реализация базовых тестов;
11) Реализация CI/CD;
12) Реализация авторизации;
13) Реализация управления пользователями;
14) Формирование отдельного репозитория для будущего переиспользования;
15) Реализация бизнес логики.