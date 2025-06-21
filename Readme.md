# Rust admin panel [In process]

TODO:
1) Добавить возможность загружать аватарки пользователям;
2) Решить проблему с выводом пользователя на странице ошибки 404 (уходит в панику при обращении к request.extensions);
3) Добавить возможность работы авторизации без обращения или с минимальным обращением к KeyValueService;
4) Исследовать и возможно добавить опциональное хранилище KeyValue написанное на Rust как альтернативу Redis, для ускорения работы в одно-серверных системах;
5) Вынести 500 тексты ошибок как константы в отдельный файл и добавить переводы к ним;
6) Добавить систему логирования с возможностью доступа через web ui;
7) Написать генерируемую документацию и позаботиться о доступе к ней через web ui;
8) Сформировать CI/CD через файл для одно-серверных систем с запуском без докера;
9) Сформировать CI/CD через файл для одно-серверных систем с запуском через docker-compose;
10) Сформировать CI/CD через файл для k8s систем;
11) Сформировать CI/CD GitLab для одно-серверных систем с запуском без докера;
12) Сформировать CI/CD GitLab для одно-серверных систем с запуском через docker-compose;
13) Сформировать CI/CD GitLab для k8s систем;
14) Провести нагрузочное тестирование;
15) Постараться оптимизировать скорость исполнения ещё сильнее и сократить расход памяти путём уменьшения размеров типов переменных, там где это возможно;
16) Проверить безопасность подключенных библиотек вручную просмотрев их код;
17) Переименовать сборку в что-то уникальное, например взять Laravel за основу и назвать сборку Ralaver;
18) Выпустить первый релиз;
19) Оформить статью на habr, а так же проконсультироваться с безопасниками касательно сборки.

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
diesel migration run
```
```shell
docker compose -f dev.docker-compose.yaml exec app diesel migration run
```

Команда отката миграций:
```shell
diesel migration revert
```
```shell
docker compose -f dev.docker-compose.yaml exec app diesel migration revert
```

Команда создания миграций:
```shell
diesel migration generate create_users
```
```shell
docker compose -f dev.docker-compose.yaml exec app diesel migration generate create_users
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

## Заметки:

Если возникает ошибка diesel: 
`No function or associated item "as_select" found in the current scope for struct "User"` и подобные, 
то нужно подключить методы `use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};`. Например:
```rust
use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};

fn run(){
    let mut connection = db_pool.get()
        .map_err(|_| error::ErrorInternalServerError(""))?;

    let results: Vec<User> = crate::schema::users::dsl::users
        .select(User::as_select())
        .limit(1)
        .load::<User>(&mut connection)
        .map_err(|_| error::ErrorInternalServerError(""))?;

    let result: Option<&User> = results.get(0);
}
```